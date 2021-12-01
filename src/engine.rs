use crate::{
    assets::mesh::{Normal, Vertex},
    components::{Model, Transform},
    ecs::World,
    error::Error,
    shaders::{vertex, Shaders},
};
use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};
use std::sync::{Arc, Mutex};
use vulkano::{
    buffer::{cpu_pool::CpuBufferPool, BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    descriptor_set::persistent::PersistentDescriptorSet,
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{
        attachment::AttachmentImage, view::ImageView, ImageUsage, SwapchainImage,
    },
    instance::Instance,
    pipeline::{
        depth_stencil::DepthStencil, vertex::BuffersDefinition, viewport::Viewport,
        GraphicsPipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    swapchain::{
        self, AcquireError, ColorSpace, SurfaceTransform, Swapchain, SwapchainCreationError,
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

pub type Surface = swapchain::Surface<Window>;

pub struct Engine {
    pub physical_index: usize,
    pub event_loop: EventLoop<()>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
    pub shaders: Arc<Shaders>,
    pub render_pass: Arc<RenderPass>,
    pub pipeline: Mutex<Arc<GraphicsPipeline>>,
    pub swapchain: Mutex<Arc<Swapchain<Window>>>,
    pub framebuffers: Mutex<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    pub world: Mutex<Arc<World>>,
}

impl Engine {
    pub fn new(physical_index: usize, world: Arc<World>) -> Result<Self, Error> {
        let req_exts = vulkano_win::required_extensions();
        let instance = Instance::new(None, Version::V1_1, &req_exts, None)?;
        let physical = match PhysicalDevice::from_index(&instance, physical_index) {
            Some(physical) => physical,
            None => return Err(Error::NoPhysicalDevice),
        };
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone())?;
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
        let shaders = Arc::new(Shaders::new(device.clone())?);
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
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: Format::D16_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
        )?);
        let (pipeline, framebuffers) = Self::window_size_dependent_setup(
            &images,
            render_pass.clone(),
            device.clone(),
            shaders.clone(),
        )?;

        Ok(Self {
            physical_index,
            event_loop,
            device,
            queue,
            surface,
            shaders,
            render_pass,
            pipeline: Mutex::new(pipeline),
            swapchain: Mutex::new(swapchain),
            framebuffers: Mutex::new(framebuffers),
            world: Mutex::new(world),
        })
    }

    pub fn first(world: Arc<World>) -> Result<Self, Error> {
        Self::new(0, world)
    }

    pub fn run(self) -> Result<(), Error> {
        let sampler = Sampler::new(
            self.device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();
        let uniform_buffer =
            CpuBufferPool::<vertex::ty::Data>::new(self.device.clone(), BufferUsage::all());
        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.event_loop.run(move |event, _, control_flow| match event {
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
                
                let entities = self.world.lock().unwrap().entities();
                let entities = entities.lock().unwrap();

                for entity in &*entities {
                    for (_, v) in &*entity.components().lock().unwrap() {
                        for component in &*v.lock().unwrap() {
                            component.on_update();
                        }
                    }
                }

                let dimensions: [u32; 2] = self.surface.window().inner_size().into();
                let mut pipeline = self.pipeline.lock().unwrap();
                let mut swapchain = self.swapchain.lock().unwrap();
                let mut framebuffers = self.framebuffers.lock().unwrap();

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
                    )
                    .unwrap();

                    *pipeline = new_pipeline;
                    *framebuffers = new_framebuffers;

                    recreate_swapchain = false;
                }

                for entity in &*entities {
                    if let Some(models) = entity.get_type::<Model>(Arc::new("model".to_string())) {
                        if let Some(transform) =
                            entity.get_first::<Transform>(Arc::new("transform".to_string()))
                        {
                            for model in &*models {
                                let uniform_buffer_subbuffer = {
                                    let rotation = Matrix3::from_angle_x(Rad(transform.rotation.x))
                                        * Matrix3::from_angle_y(Rad(transform.rotation.y))
                                        * Matrix3::from_angle_z(Rad(transform.rotation.z));

                                    let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                                    let proj = cgmath::perspective(
                                        Rad(std::f32::consts::FRAC_PI_2),
                                        aspect_ratio,
                                        0.01,
                                        100.0,
                                    );
                                    let view = Matrix4::look_at_rh(
                                        Point3::new(0.3, 0.3, 1.0),
                                        Point3::new(0.0, 0.0, 0.0),
                                        Vector3::new(0.0, -1.0, 0.0),
                                    );
                                    let scale = Matrix4::from_scale(0.01);
                                    let uniform_data = vertex::ty::Data {
                                        world: Matrix4::from(rotation).into(),
                                        view: (view * scale).into(),
                                        proj: proj.into(),
                                        position: transform.position.into(),
                                    };

                                    Arc::new(uniform_buffer.next(uniform_data).unwrap())
                                };
                                let set_layouts = pipeline.layout().descriptor_set_layouts();
                                let set_layout = set_layouts.get(0).unwrap();
                                let mut set_builder =
                                    PersistentDescriptorSet::start(set_layout.clone());

                                set_builder
                                    .add_buffer(uniform_buffer_subbuffer)
                                    .unwrap()
                                    .add_sampled_image(model.texture.image.clone(), sampler.clone())
                                    .unwrap();

                                let set = Arc::new(set_builder.build().unwrap());
                                let (image_num, suboptimal, acquire_future) =
                                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                                        Ok(r) => r,

                                        Err(AcquireError::OutOfDate) => {
                                            recreate_swapchain = true;

                                            return;
                                        }

                                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                                    };

                                if suboptimal {
                                    recreate_swapchain = true;
                                }

                                let normal_buffer = CpuAccessibleBuffer::from_iter(
                                    self.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    model.asset.normals.iter().cloned(),
                                )
                                .unwrap();
                                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                                    self.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    model.asset.vertices.iter().cloned(),
                                )
                                .unwrap();
                                let index_buffer = CpuAccessibleBuffer::from_iter(
                                    self.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    model.asset.indices.iter().cloned(),
                                )
                                .unwrap();
                                let mut builder = AutoCommandBufferBuilder::primary(
                                    self.device.clone(),
                                    self.queue.family(),
                                    CommandBufferUsage::OneTimeSubmit,
                                )
                                .unwrap();

                                builder
                                    .begin_render_pass(
                                        framebuffers[image_num].clone(),
                                        SubpassContents::Inline,
                                        vec![[0.0, 0.0, 1.0, 1.0].into(), 1_f32.into()],
                                    )
                                    .unwrap()
                                    .bind_pipeline_graphics(pipeline.clone())
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
                                    .unwrap()
                                    .end_render_pass()
                                    .unwrap();

                                let command_buffer = builder.build().unwrap();
                                let future = previous_frame_end
                                    .take()
                                    .unwrap()
                                    .join(acquire_future)
                                    .then_execute(self.queue.clone(), command_buffer)
                                    .unwrap()
                                    .then_swapchain_present(
                                        self.queue.clone(),
                                        swapchain.clone(),
                                        image_num,
                                    )
                                    .then_signal_fence_and_flush();

                                match future {
                                    Ok(future) => {
                                        previous_frame_end = Some(future.boxed());
                                    }
                                    Err(FlushError::OutOfDate) => {
                                        recreate_swapchain = true;
                                        previous_frame_end =
                                            Some(sync::now(self.device.clone()).boxed());
                                    }
                                    Err(e) => {
                                        println!("Failed to flush future: {:?}", e);

                                        previous_frame_end =
                                            Some(sync::now(self.device.clone()).boxed());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        });
    }

    fn window_size_dependent_setup(
        images: &Vec<Arc<SwapchainImage<Window>>>,
        render_pass: Arc<RenderPass>,
        device: Arc<Device>,
        shaders: Arc<Shaders>,
    ) -> Result<
        (
            Arc<GraphicsPipeline>,
            Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
        ),
        Error,
    > {
        let dimensions = images[0].dimensions();
        let depth_buffer = ImageView::new(
            AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap(),
        )
        .unwrap();
        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new(image.clone()).unwrap();
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(view)
                        .unwrap()
                        .add(depth_buffer.clone())
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

    pub fn physical_index(&self) -> usize {
        self.physical_index
    }
}
