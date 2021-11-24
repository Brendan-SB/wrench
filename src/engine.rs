use crate::{error::Error, shaders::Shaders, vertex::Vertex};
use cgmath::Vector3;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, SubpassContents},
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{ImageUsage, SwapchainImage},
    instance::Instance,
    pipeline::{viewport::Viewport, DynamicState, GraphicsPipeline},
    render_pass::{Framebuffer, FramebufferAbstract, Subpass},
    spirv::Capability,
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

vulkano::impl_vertex!(Vertex, position);

pub type Surface = swapchain::Surface<Window>;

pub struct Engine {
    physical_index: usize,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    shaders: Arc<Shaders>,
    pipeline: Arc<GraphicsPipeline>,
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
                    format: Format::R8G8B8A8_UNORM,
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
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(shaders.fragment.main_entry_point(), ())
                .render_pass(match Subpass::from(render_pass.clone(), 0) {
                    Some(subpass) => subpass,
                    None => return Err(Error::NoSubpass),
                })
                .build(device.clone())?,
        );

        Ok(Self {
            physical_index,
            device,
            queue,
            surface,
            shaders,
            pipeline,
        })
    }

    pub fn default_device() -> Result<Self, Error> {
        Self::new(0)
    }

    pub fn run(&mut self) -> Result<(), Error> {
        Ok(())
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

    pub fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipeline.clone()
    }
}
