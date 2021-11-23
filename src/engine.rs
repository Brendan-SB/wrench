use crate::error::Error;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, SubpassContents},
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    image::{ImageUsage, SwapchainImage},
    instance::Instance,
    pipeline::{viewport::Viewport, DynamicState, GraphicsPipeline},
    render_pass::{Framebuffer, FramebufferAbstract, Subpass},
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
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
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
        )
        .unwrap();
        let queue = match queues.next() {
            Some(queue) => queue,
            None => return Err(Error::NoQueue),
        };

        Ok(Self {
            physical_index,
            device,
            queue,
            surface,
        })
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
}
