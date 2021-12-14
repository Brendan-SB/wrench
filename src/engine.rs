use crate::{
    assets::mesh::{Normal, Vertex},
    components::{EventHandler, Model},
    ecs,
    error::Error,
    scene::Scene,
    shaders::{fragment, vertex, Shaders},
    Matrix4, Rad, Vector3, Vector4, Zero,
};
use std::sync::{Arc, Mutex};
use vulkano::{
    buffer::{cpu_pool::CpuBufferPool, BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    descriptor_set::persistent::PersistentDescriptorSet,
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{
        attachment::AttachmentImage, view::ImageView, ImageUsage, SampleCount, SwapchainImage,
    },
    instance::Instance,
    pipeline::{
        depth_stencil::DepthStencil, vertex::BuffersDefinition, viewport::Viewport,
        GraphicsPipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass},
    swapchain::{
        self, AcquireError, ColorSpace, Surface, SurfaceTransform, Swapchain,
        SwapchainCreationError,
    },
    sync::{self, FlushError, GpuFuture},
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub const MAX_EVENTS: usize = 50;

pub struct Engine {
    pub physical_index: usize,
    pub sample_count: Arc<SampleCount>,
    pub event_loop: EventLoop<()>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<Window>>,
    pub shaders: Arc<Shaders>,
    pub render_pass: Arc<RenderPass>,
    pub pipeline: Mutex<Arc<GraphicsPipeline>>,
    pub swapchain: Mutex<Arc<Swapchain<Window>>>,
    pub framebuffers: Mutex<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    pub scene: Mutex<Arc<Scene>>,
}

impl Engine {
    pub fn new(
        physical_index: usize,
        window_title: Arc<String>,
        scene: Arc<Scene>,
        sample_count: Arc<SampleCount>,
    ) -> Result<Self, Error> {
        let req_exts = vulkano_win::required_extensions();
        let instance = Instance::new(None, Version::V1_1, &req_exts, None)?;
        let physical = match PhysicalDevice::from_index(&instance, physical_index) {
            Some(physical) => physical,
            None => return Err(Error::NoPhysicalDevice),
        };
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title((*window_title).clone())
            .build_vk_surface(&event_loop, instance.clone())?;
        let queue_family = match physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        {
            Some(queue_family) => queue_family,
            None => return Err(Error::NoQueueFamily),
        };
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, 0.5)].iter().cloned(),
        )?;
        let queue = match queues.next() {
            Some(queue) => queue,
            None => return Err(Error::NoQueue),
        };
        let shaders = Shaders::new(device.clone())?;
        let (swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let dimensions: [u32; 2] = surface.window().inner_size().into();

            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .composite_alpha(alpha)
                .format(format)
                .dimensions(dimensions)
                .layers(1)
                .usage(ImageUsage::color_attachment())
                .transform(SurfaceTransform::Identity)
                .clipped(true)
                .color_space(ColorSpace::SrgbNonLinear)
                .build()
                .unwrap()
        };
        let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                intermediary: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: *sample_count,
                },
                depth: {
                    load: Clear,
                    store: Store,
                    format: Format::D16_UNORM,
                    samples: 1,
                },
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [intermediary],
                depth_stencil: {depth},
                resolve: [color],
            }
        )?);
        let (pipeline, framebuffers) = Self::window_size_dependent_setup(
            &images,
            render_pass.clone(),
            device.clone(),
            shaders.clone(),
            swapchain.clone(),
            sample_count.clone(),
        )?;

        Ok(Self {
            physical_index,
            sample_count,
            event_loop,
            device,
            queue,
            surface,
            shaders,
            render_pass,
            pipeline: Mutex::new(pipeline),
            swapchain: Mutex::new(swapchain),
            framebuffers: Mutex::new(framebuffers),
            scene: Mutex::new(scene),
        })
    }

    pub fn first(
        window_title: Arc<String>,
        scene: Arc<Scene>,
        sample_count: Arc<SampleCount>,
    ) -> Result<Self, Error> {
        Self::new(0, window_title, scene, sample_count)
    }

    pub fn init(self) -> Result<(), Error> {
        for entity in self.scene.lock().unwrap().world.entities() {
            for (_, v) in entity.components() {
                for component in v {
                    component.init();
                }
            }
        }

        let uniform_buffer =
            CpuBufferPool::<vertex::ty::Data>::new(self.device.clone(), BufferUsage::all());
        let frag_uniform_buffer =
            CpuBufferPool::<fragment::ty::Data>::new(self.device.clone(), BufferUsage::all());
        let mut lights_array = [fragment::ty::Light {
            position: Vector3::zero().into(),
            color: Vector4::zero().into(),
            intensity: 0.0,
            _dummy0: [0; 4],
            _dummy1: [0; 12],
        }; 1024];
        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.event_loop.run(move |event, _, control_flow| {
            for entity in self.scene.lock().unwrap().world.entities() {
                for (_, v) in entity.components() {
                    for component in v {
                        component.update();
                    }
                }

                if let Some(event_handlers) =
                    entity.get_type::<EventHandler>(ecs::id("event handler"))
                {
                    for event_handler in &*event_handlers {
                        event_handler.handle(&event);
                    }
                }
            }

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,

                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    recreate_swapchain = true;
                }

                Event::RedrawEventsCleared => {
                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    let dimensions: [u32; 2] = self.surface.window().inner_size().into();
                    let mut pipeline = self.pipeline.lock().unwrap();
                    let mut swapchain = self.swapchain.lock().unwrap();
                    let mut framebuffers = self.framebuffers.lock().unwrap();
                    let scene = self.scene.lock().unwrap();

                    if recreate_swapchain {
                        let (new_swapchain, new_images) =
                            match swapchain.recreate().dimensions(dimensions).build() {
                                Ok(r) => r,
                                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                            };

                        *swapchain = new_swapchain;

                        let (new_pipeline, new_framebuffers) = Self::window_size_dependent_setup(
                            &new_images,
                            self.render_pass.clone(),
                            self.device.clone(),
                            self.shaders.clone(),
                            swapchain.clone(),
                            self.sample_count.clone(),
                        )
                        .unwrap();

                        *pipeline = new_pipeline;
                        *framebuffers = new_framebuffers;

                        recreate_swapchain = false;
                    }

                    let (image_num, suboptimal, acquire_future) =
                        match swapchain::acquire_next_image(swapchain.clone(), None) {
                            Ok(r) => r,

                            Err(AcquireError::OutOfDate) => {
                                recreate_swapchain = true;

                                return;
                            }

                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };

                    let mut builder = AutoCommandBufferBuilder::primary(
                        self.device.clone(),
                        self.queue.family(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();
                    let bg: [f32; 4] = (*scene.bg.lock().unwrap()).into();

                    builder
                        .bind_pipeline_graphics(pipeline.clone())
                        .begin_render_pass(
                            framebuffers[image_num].clone(),
                            SubpassContents::Inline,
                            vec![bg.into(), 1_f32.into(), [0.0, 0.0, 0.0, 0.0].into()],
                        )
                        .unwrap();

                    for entity in scene.world.entities() {
                        if let Some(models) = entity.get_type::<Model>(ecs::id("model")) {
                            for model in &*models {
                                let uniform_buffer_subbuffer = {
                                    let rotation = {
                                        let rotation = model.transform.rotation.lock().unwrap();

                                        Matrix4::from_angle_x(Rad(rotation.x))
                                            * Matrix4::from_angle_y(Rad(rotation.y))
                                            * Matrix4::from_angle_z(Rad(rotation.z))
                                    };
                                    let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                                    let camera = scene.camera.lock().unwrap();
                                    let proj = cgmath::perspective(
                                        Rad(*camera.fov.lock().unwrap()),
                                        aspect_ratio,
                                        *camera.near.lock().unwrap(),
                                        *camera.far.lock().unwrap(),
                                    );
                                    let cam_rotation = {
                                        let rotation = camera.transform.rotation.lock().unwrap();

                                        Matrix4::from_angle_x(Rad(rotation.x))
                                            * Matrix4::from_angle_y(Rad(rotation.y))
                                            * Matrix4::from_angle_z(Rad(rotation.z))
                                    };
                                    let translation = Matrix4::from_translation(
                                        *model.transform.position.lock().unwrap(),
                                    );
                                    let cam_translation = Matrix4::from_translation(
                                        *camera.transform.position.lock().unwrap(),
                                    );
                                    let transform = rotation * translation;
                                    let cam_transform = cam_rotation * cam_translation;
                                    let uniform_data = vertex::ty::Data {
                                        rotation: rotation.into(),
                                        cam_rotation: cam_rotation.into(),
                                        proj: proj.into(),
                                        translation: translation.into(),
                                        cam_translation: cam_translation.into(),
                                        scale: {
                                            let scale = model.transform.scale.lock().unwrap();

                                            Matrix4::from_nonuniform_scale(
                                                scale.x, scale.y, scale.z,
                                            )
                                        }
                                        .into(),
                                        transform: transform.into(),
                                        cam_transform: cam_transform.into(),
                                    };

                                    Arc::new(uniform_buffer.next(uniform_data).unwrap())
                                };
                                let frag_uniform_buffer_subbuffer = {
                                    let lights = scene.lights.lock().unwrap();

                                    for (i, light) in lights.iter().enumerate() {
                                        lights_array[i] = fragment::ty::Light {
                                            position: (*light.transform.position.lock().unwrap())
                                                .into(),
                                            color: (*light.color.lock().unwrap()).into(),
                                            intensity: *light.intensity.lock().unwrap(),
                                            _dummy0: [0; 4],
                                            _dummy1: [0; 12],
                                        };
                                    }

                                    let material = model.material.lock().unwrap();
                                    let uniform_data = fragment::ty::Data {
                                        color: (*model.color.lock().unwrap()).into(),
                                        ambient: *material.ambient.lock().unwrap(),
                                        diff_strength: *material.diff_strength.lock().unwrap(),
                                        spec_strength: *material.spec_strength.lock().unwrap(),
                                        spec_power: *material.spec_power.lock().unwrap(),
                                        lights: fragment::ty::LightArray {
                                            len: lights.len() as u32,
                                            array: lights_array,
                                            _dummy0: [0; 12],
                                        },
                                    };

                                    Arc::new(frag_uniform_buffer.next(uniform_data).unwrap())
                                };
                                let set_layout =
                                    pipeline.layout().descriptor_set_layouts().get(0).unwrap();
                                let mut set_builder =
                                    PersistentDescriptorSet::start(set_layout.clone());
                                let texture = model.texture.lock().unwrap();

                                set_builder
                                    .add_buffer(uniform_buffer_subbuffer)
                                    .unwrap()
                                    .add_sampled_image(
                                        texture.image.clone(),
                                        texture.sampler.clone(),
                                    )
                                    .unwrap()
                                    .add_buffer(frag_uniform_buffer_subbuffer)
                                    .unwrap();

                                let set = Arc::new(set_builder.build().unwrap());

                                if suboptimal {
                                    recreate_swapchain = true;
                                }

                                let mesh = model.mesh.lock().unwrap();
                                let normal_buffer = CpuAccessibleBuffer::from_iter(
                                    self.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    mesh.normals.iter().cloned(),
                                )
                                .unwrap();
                                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                                    self.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    mesh.vertices.iter().cloned(),
                                )
                                .unwrap();
                                let index_buffer = CpuAccessibleBuffer::from_iter(
                                    self.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    mesh.indices.iter().cloned(),
                                )
                                .unwrap();

                                builder
                                    .bind_descriptor_sets(
                                        PipelineBindPoint::Graphics,
                                        pipeline.layout().clone(),
                                        0,
                                        set.clone(),
                                    )
                                    .bind_vertex_buffers(
                                        0,
                                        (vertex_buffer.clone(), normal_buffer.clone()),
                                    )
                                    .bind_index_buffer(index_buffer.clone())
                                    .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                                    .unwrap();
                            }
                        }
                    }

                    builder.end_render_pass().unwrap();

                    let command_buffer = builder.build().unwrap();
                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(self.queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(self.queue.clone(), swapchain.clone(), image_num)
                        .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => {
                            previous_frame_end = Some(future.boxed());
                        }
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                        }
                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);

                            previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn window_size_dependent_setup<W>(
        images: &Vec<Arc<SwapchainImage<Window>>>,
        render_pass: Arc<RenderPass>,
        device: Arc<Device>,
        shaders: Arc<Shaders>,
        swapchain: Arc<Swapchain<W>>,
        sample_count: Arc<SampleCount>,
    ) -> Result<
        (
            Arc<GraphicsPipeline>,
            Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
        ),
        Error,
    > {
        let dimensions = images[0].dimensions();
        let intermediary = ImageView::new(AttachmentImage::transient_multisampled(
            device.clone(),
            dimensions,
            (*sample_count).clone(),
            swapchain.format(),
        )?)?;
        let depth_buffer = ImageView::new(AttachmentImage::transient(
            device.clone(),
            dimensions,
            Format::D16_UNORM,
        )?)?;
        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new(image.clone()).unwrap();

                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(intermediary.clone())
                        .unwrap()
                        .add(depth_buffer.clone())
                        .unwrap()
                        .add(view)
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>();
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input(
                    BuffersDefinition::new()
                        .vertex::<Vertex>()
                        .vertex::<Normal>(),
                )
                .vertex_shader(shaders.vertex.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(shaders.fragment.main_entry_point(), ())
                .render_pass(match Subpass::from(render_pass.clone(), 0) {
                    Some(subpass) => subpass,
                    None => return Err(Error::NoSubpass),
                })
                .viewports(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }])
                .depth_stencil(DepthStencil::simple_depth_test())
                .build(device.clone())?,
        );

        Ok((pipeline, framebuffers))
    }
}
