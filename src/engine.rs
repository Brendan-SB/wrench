use crate::{error::Error, shaders::Shaders, vertex::Vertex};
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::Instance,
    pipeline::{viewport::Viewport, GraphicsPipeline},
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass},
    swapchain::{
        self, AcquireError, ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform,
        Swapchain, SwapchainCreationError,
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
    physical_index: usize,
    event_loop: EventLoop<()>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    shaders: Arc<Shaders>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

impl Engine {
    pub fn new(physical_index: usize) -> Result<Self, Error> {
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
        let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::B8G8R8A8_SRGB,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )?);
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
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
                    dimensions: [100 as f32, 100 as f32],
                    depth_range: 0.0..1.0,
                }])
                .build(device.clone())?,
        );
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
        let framebuffers = Self::window_size_dependent_setup(&images, render_pass.clone());

        Ok(Self {
            physical_index,
            event_loop,
            device,
            queue,
            surface,
            shaders,
            render_pass,
            pipeline,
            swapchain,
            images,
            framebuffers,
        })
    }

    pub fn first_device() -> Result<Self, Error> {
        Self::new(0)
    }

    pub fn run(mut self) {
        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.event_loop
            .run(move |event, _, control_flow| match event {
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

                    if recreate_swapchain {
                        let (new_swapchain, new_images) =
                            self.swapchain.recreate().build().unwrap();

                        self.swapchain = new_swapchain;
                        self.images = new_images;

                        self.framebuffers = Self::window_size_dependent_setup(
                            &self.images,
                            self.render_pass.clone(),
                        );

                        recreate_swapchain = false;
                    }
                    let (image_num, suboptimal, acquire_future) =
                        match swapchain::acquire_next_image(self.swapchain.clone(), None) {
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

                    let vertices = [
                        Vertex {
                            position: [-0.5, -0.25, 0.0],
                        },
                        Vertex {
                            position: [0.0, 0.5, 0.0],
                        },
                        Vertex {
                            position: [0.25, -0.1, 0.0],
                        },
                    ];
                    let mut builder = AutoCommandBufferBuilder::primary(
                        self.device.clone(),
                        self.queue.family(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();

                    builder
                        .bind_pipeline_graphics(self.pipeline.clone())
                        .begin_render_pass(
                            self.framebuffers[image_num].clone(),
                            SubpassContents::Inline,
                            vec![[0.0, 0.0, 1.0, 1.0].into()],
                        )
                        .unwrap()
                        .bind_vertex_buffers(0, vertices.iter().map(|v| v.position).collect::Vec<[f32; 3]>())
                        .draw(vertices.len() as u32, (vertices.len() * 3) as u32, 0, 0)
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
                            self.swapchain.clone(),
                            image_num,
                        )
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
            });
    }

    fn window_size_dependent_setup(
        images: &Vec<Arc<SwapchainImage<Window>>>,
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(ImageView::new(image.clone()).unwrap())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>()
    }

    pub fn physical_index(&self) -> usize {
        self.physical_index
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    pub fn shaders(&self) -> Arc<Shaders> {
        self.shaders.clone()
    }

    pub fn render_pass(&self) -> Arc<RenderPass> {
        self.render_pass.clone()
    }

    pub fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipeline.clone()
    }
}
