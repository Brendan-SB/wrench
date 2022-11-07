use crate::{
    assets::mesh::{Normal, Vertex},
    components::{Camera, EventHandler, Light, Model, EVENT_HANDLER_ID, MODEL_ID},
    ecs::{self, Component, Entity, ENTITY_ID},
    error::Error,
    scene::Scene,
    shaders::{
        fragment::{self, MAX_LIGHTS},
        vertex, Shaders,
    },
};
use cgmath::{Matrix4, SquareMatrix, Vector3, Zero};
use std::sync::{Arc, RwLock};
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

pub struct InitializedEngine {
    pub lights_array: [fragment::ty::Light; MAX_LIGHTS],
    pub uniform_buffer: CpuBufferPool<vertex::ty::Data>,
    pub frag_uniform_buffer: CpuBufferPool<fragment::ty::Data>,
}

impl InitializedEngine {
    pub fn new(
        lights_array: [fragment::ty::Light; MAX_LIGHTS],
        uniform_buffer: CpuBufferPool<vertex::ty::Data>,
        frag_uniform_buffer: CpuBufferPool<fragment::ty::Data>,
    ) -> Self {
        Self {
            lights_array,
            uniform_buffer,
            frag_uniform_buffer,
        }
    }
}

pub struct Engine {
    pub physical_index: usize,
    pub sample_count: SampleCount,
    pub event_loop: EventLoop<()>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<Window>>,
    pub render_pass: Arc<RenderPass>,
    pub swapchain: RwLock<Arc<Swapchain<Window>>>,
    #[allow(clippy::type_complexity)]
    pub images: RwLock<Vec<Arc<ImageView<Arc<SwapchainImage<Window>>>>>>,
    pub scene: RwLock<Arc<Scene>>,
}

impl Engine {
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
        let physical = PhysicalDevice::from_index(&instance, physical_index).unwrap();
        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .unwrap();
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
        let queue = queues.next().unwrap();
        let (swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let dimensions: [u32; 2] = surface.window().inner_size().into();
            let (swapchain, images) = Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .composite_alpha(alpha)
                .format(format)
                .dimensions(dimensions)
                .layers(1)
                .usage(ImageUsage::color_attachment())
                .transform(SurfaceTransform::Identity)
                .clipped(true)
                .color_space(ColorSpace::SrgbNonLinear)
                .build()?;
            let images = images
                .into_iter()
                .map(|i| ImageView::new(i).unwrap())
                .collect::<Vec<_>>();

            (swapchain, images)
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
            pass:
            {
                color: [intermediary],
                depth_stencil: {depth},
                resolve: [color]
            }
        )?);

        Ok(Self {
            physical_index,
            sample_count,
            event_loop,
            device,
            queue,
            surface,
            render_pass,
            swapchain: RwLock::new(swapchain),
            images: RwLock::new(images),
            scene: RwLock::new(scene),
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
        render_pass: Arc<RenderPass>,
        device: Arc<Device>,
        shaders: Arc<Shaders>,
        swapchain: Arc<Swapchain<Window>>,
    ) -> Result<Arc<GraphicsPipeline>, Error> {
        let dimensions = swapchain.dimensions();
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
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .viewports(vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }])
                .depth_stencil(DepthStencil::simple_depth_test())
                .blend_alpha_blending()
                .build(device)?,
        );

        Ok(pipeline)
    }

    fn create_framebuffers(
        device: Arc<Device>,
        swapchain: Arc<Swapchain<Window>>,
        render_pass: Arc<RenderPass>,
        color: Arc<ImageView<Arc<SwapchainImage<Window>>>>,
        sample_count: SampleCount,
        dimensions: &[u32; 2],
    ) -> Result<Arc<dyn FramebufferAbstract + Send + Sync>, Error> {
        let usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            color_attachment: true,
            ..ImageUsage::none()
        };
        let intermediary = ImageView::new(AttachmentImage::multisampled_with_usage(
            device.clone(),
            *dimensions,
            sample_count,
            swapchain.format(),
            usage,
        )?)?;
        let depth = ImageView::new(AttachmentImage::multisampled_with_usage(
            device,
            *dimensions,
            sample_count,
            Format::D16_UNORM,
            usage,
        )?)?;
        let framebuffer = Arc::new(
            Framebuffer::start(render_pass)
                .add(intermediary)?
                .add(depth)?
                .add(color)?
                .build()?,
        );

        Ok(framebuffer)
    }

    fn handle_events(entity: Arc<Entity>, event: &Event<()>) {
        for event_handler in &*entity.get_type::<EventHandler>(ecs::id(EVENT_HANDLER_ID)) {
            event_handler.handle(event);
        }

        for entity in &*entity.get_type::<Entity>(ecs::id(ENTITY_ID)) {
            Self::handle_events(entity.clone(), event);
        }
    }

    fn draw_entities(
        initialized_engine: &mut InitializedEngine,
        entities: Vec<Arc<Entity>>,
        camera: Arc<Camera>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandPoolBuilder,
        >,
        pipeline: &GraphicsPipeline,
        lights: &Vec<Arc<Light>>,
        dimensions: &[u32; 2],
    ) {
        for entity in &*entities {
            for model in &*entity.get_type::<Model>(ecs::id(MODEL_ID)) {
                if model.data.read().unwrap().visible {
                    model.draw(
                        initialized_engine,
                        camera.clone(),
                        builder,
                        pipeline,
                        lights,
                        dimensions,
                    );
                }
            }

            Self::draw_entities(
                initialized_engine,
                entity.get_type(ecs::id(ENTITY_ID)),
                camera.clone(),
                builder,
                pipeline,
                lights,
                dimensions,
            );
        }
    }

    pub fn init(self, shaders: Arc<Shaders>) -> Result<(), Error> {
        self.scene.read().unwrap().root.on_init();

        let mut pipeline = Self::window_size_dependent_setup(
            self.render_pass.clone(),
            self.device.clone(),
            shaders.clone(),
            self.swapchain.read().unwrap().clone(),
        )?;
        let uniform_buffer = CpuBufferPool::<vertex::ty::Data>::new(
            self.device.clone(),
            BufferUsage::uniform_buffer(),
        );
        let frag_uniform_buffer = CpuBufferPool::<fragment::ty::Data>::new(
            self.device.clone(),
            BufferUsage::uniform_buffer(),
        );
        let lights_array = [fragment::ty::Light {
            position: Matrix4::identity().into(),
            rotation: Matrix4::identity().into(),
            proj: Matrix4::identity().into(),
            color: Vector3::zero().into(),
            directional: 0,
            cutoff: 0.0,
            outer_cutoff: 0.0,
            intensity: 0.0,
            attenuation: 0.0,
        }; MAX_LIGHTS];
        let mut initialized_engine =
            InitializedEngine::new(lights_array, uniform_buffer, frag_uniform_buffer);
        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.event_loop.run(move |event, _, control_flow| {
            {
                let scene = self.scene.read().unwrap();

                Self::handle_events(scene.root.clone(), &event);
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
                    let mut swapchain = self.swapchain.write().unwrap();
                    let mut images = self.images.write().unwrap();
                    let scene = self.scene.read().unwrap();

                    scene.root.on_update();

                    if recreate_swapchain {
                        let (new_swapchain, new_images) =
                            match swapchain.recreate().dimensions(dimensions).build() {
                                Ok(r) => r,
                                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                            };

                        *swapchain = new_swapchain;
                        *images = new_images
                            .into_iter()
                            .map(|i| ImageView::new(i).unwrap())
                            .collect::<Vec<_>>();

                        pipeline = Self::window_size_dependent_setup(
                            self.render_pass.clone(),
                            self.device.clone(),
                            shaders.clone(),
                            swapchain.clone(),
                        )
                        .unwrap();

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

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    let lights = scene.get_lights();
                    let entities = scene.root.get_type::<Entity>(ecs::id(ENTITY_ID));
                    let camera = { scene.camera.read().unwrap().clone() };
                    let bg: [f32; 4] = (*scene.bg.read().unwrap()).into();
                    let framebuffer = Self::create_framebuffers(
                        self.device.clone(),
                        swapchain.clone(),
                        self.render_pass.clone(),
                        images[image_num].clone(),
                        self.sample_count,
                        &dimensions,
                    )
                    .unwrap();
                    let mut builder = AutoCommandBufferBuilder::primary(
                        self.device.clone(),
                        self.queue.family(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();

                    builder
                        .bind_pipeline_graphics(pipeline.clone())
                        .begin_render_pass(
                            framebuffer.clone(),
                            SubpassContents::Inline,
                            vec![bg.into(), 1.0_f32.into(), [0.0_f32; 4].into()],
                        )
                        .unwrap();

                    Self::draw_entities(
                        &mut initialized_engine,
                        entities,
                        camera,
                        &mut builder,
                        &pipeline,
                        &lights,
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
