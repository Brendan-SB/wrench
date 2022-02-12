use super::Engine;
use crate::{
    assets::mesh::{Normal, Vertex},
    components::{EventHandler, Light, Model},
    ecs::{self, Component, Entity},
    error::Error,
    scene::Scene,
    shaders::{fragment, vertex, Shaders},
    Matrix4, Vector3, Zero,
};
use std::sync::{Arc, Mutex};
use vulkano::{
    buffer::{cpu_pool::CpuBufferPool, BufferUsage},
    command_buffer::{
        pool::standard::StandardCommandPoolBuilder, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, SubpassContents,
    },
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{
        attachment::AttachmentImage, view::ImageView, ImageUsage, SampleCount, SwapchainImage,
    },
    instance::Instance,
    pipeline::{
        depth_stencil::DepthStencil, vertex::BuffersDefinition, viewport::Viewport,
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass},
    swapchain::{
        self, AcquireError, ColorSpace, Surface, SurfaceTransform, Swapchain,
        SwapchainCreationError,
    },
    sync::{self, FlushError, GpuFuture},
    Version,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct DefaultEngine {
    pub physical_index: usize,
    pub sample_count: SampleCount,
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

impl DefaultEngine {
    pub fn instance() -> Result<Arc<Instance>, Error> {
        let req_exts = vulkano_win::required_extensions();
        let instance = Instance::new(None, Version::V1_1, &req_exts, None)?;

        Ok(instance)
    }

    pub fn new(
        physical_index: usize,
        surface: Arc<Surface<Window>>,
        instance: Arc<Instance>,
        event_loop: EventLoop<()>,
        scene: Arc<Scene>,
        sample_count: SampleCount,
    ) -> Result<Self, Error> {
        let physical = match PhysicalDevice::from_index(&instance, physical_index) {
            Some(physical) => physical,
            None => return Err(Error::NoPhysicalDevice),
        };
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
                .build()?
        };
        let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                intermediary: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: sample_count,
                },
                depth: {
                    load: Clear,
                    store: Store,
                    format: Format::D16_UNORM,
                    samples: sample_count,
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
            sample_count,
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
        surface: Arc<Surface<Window>>,
        instance: Arc<Instance>,
        event_loop: EventLoop<()>,
        scene: Arc<Scene>,
        sample_count: SampleCount,
    ) -> Result<Self, Error> {
        Self::new(0, surface, instance, event_loop, scene, sample_count)
    }

    fn window_size_dependent_setup(
        images: &Vec<Arc<SwapchainImage<Window>>>,
        render_pass: Arc<RenderPass>,
        device: Arc<Device>,
        shaders: Arc<Shaders>,
        swapchain: Arc<Swapchain<Window>>,
        sample_count: SampleCount,
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
            sample_count,
            swapchain.format(),
        )?)?;
        let depth_buffer = ImageView::new(AttachmentImage::transient_multisampled(
            device.clone(),
            dimensions,
            sample_count,
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
                .blend_alpha_blending()
                .build(device.clone())?,
        );

        Ok((pipeline, framebuffers))
    }

    fn handle_events(entities: Option<Arc<Vec<Arc<Entity>>>>, event: &Event<()>) {
        if let Some(entities) = entities {
            for entity in &*entities {
                if let Some(event_handlers) =
                    entity.get_type::<EventHandler>(ecs::id("event handler"))
                {
                    for event_handler in &*event_handlers {
                        event_handler.handle(event);
                    }
                }

                Self::handle_events(entity.get_type(ecs::id("entity")), event);
            }
        }
    }

    fn draw_entities(
        entities: Option<Arc<Vec<Arc<Entity>>>>,
        device: Arc<Device>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandPoolBuilder,
        >,
        pipeline: &GraphicsPipeline,
        suboptimal: bool,
        recreate_swapchain: &mut bool,
        light_count: &mut usize,
        lights_array: &mut [fragment::ty::Light; 1024],
        lights: &Option<Vec<Arc<Light>>>,
        uniform_buffer: &CpuBufferPool<vertex::ty::Data>,
        frag_uniform_buffer: &CpuBufferPool<fragment::ty::Data>,
        scene: &Scene,
        dimensions: &[u32; 2],
    ) {
        if let Some(entities) = entities {
            for entity in &*entities {
                if let Some(models) = entity.get_type::<Model>(ecs::id("model")) {
                    for model in &*models {
                        model.draw(
                            device.clone(),
                            builder,
                            pipeline,
                            suboptimal,
                            recreate_swapchain,
                            light_count,
                            lights_array,
                            lights,
                            uniform_buffer,
                            frag_uniform_buffer,
                            scene,
                            dimensions,
                        );
                    }
                }

                Self::draw_entities(
                    entity.get_type(ecs::id("entity")),
                    device.clone(),
                    builder,
                    pipeline,
                    suboptimal,
                    recreate_swapchain,
                    light_count,
                    lights_array,
                    lights,
                    uniform_buffer,
                    frag_uniform_buffer,
                    scene,
                    dimensions,
                );
            }
        }
    }
}

impl Engine for DefaultEngine {
    fn init(self) -> Result<(), Error> {
        self.scene.lock().unwrap().root.init();

        let uniform_buffer =
            CpuBufferPool::<vertex::ty::Data>::new(self.device.clone(), BufferUsage::all());
        let frag_uniform_buffer =
            CpuBufferPool::<fragment::ty::Data>::new(self.device.clone(), BufferUsage::all());
        let mut lights_array = [fragment::ty::Light {
            position: Vector3::zero().into(),
            rotation: Matrix4::zero().into(),
            color: Vector3::zero().into(),
            directional: 0,
            cutoff: 0.0,
            outer_cutoff: 0.0,
            intensity: 0.0,
            attenuation: 0.0,
            _dummy0: [0; 4],
        }; 1024];
        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.event_loop.run(move |event, _, control_flow| {
            {
                let scene = self.scene.lock().unwrap();

                scene.root.update();

                Self::handle_events(scene.root.get_type::<Entity>(ecs::id("entity")), &event);
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
                            vec![bg.into(), 1.0_f32.into(), [0.0_f32; 4].into()],
                        )
                        .unwrap();

                    Self::draw_entities(
                        scene.root.get_type::<Entity>(ecs::id("entity")),
                        self.device.clone(),
                        &mut builder,
                        &*pipeline,
                        suboptimal,
                        &mut recreate_swapchain,
                        &mut 0,
                        &mut lights_array,
                        &scene.get_lights(),
                        &uniform_buffer,
                        &frag_uniform_buffer,
                        &scene,
                        &dimensions,
                    );

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
}
