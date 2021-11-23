use crate::{error::Error, Shaders};
use shaderc::{CompileOptions, Compiler, ShaderKind};
use std::{fs, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, SubpassContents},
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
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

pub type Surface = swapchain::Surface<Window>;

pub struct Engine {
    physical_index: usize,
    shaders: Arc<Shaders>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
}

impl Engine {
    pub fn new(
        physical_index: usize,
        vertex_path: &String,
        fragment_path: &String,
    ) -> Result<Self, Error> {
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
        let shaders = Self::load_shaders(vertex_path, fragment_path)?;

        Ok(Self {
            physical_index,
            shaders,
            device,
            queue,
            surface,
        })
    }

    fn load_shaders(vertex_path: &String, fragment_path: &String) -> Result<Arc<Shaders>, Error> {
        let mut compiler = match Compiler::new() {
            Some(compiler) => compiler,
            None => return Err(Error::NoShaderCompiler),
        };
        let options = match CompileOptions::new() {
            Some(options) => options,
            None => return Err(Error::NoShaderCompilerOptions),
        };

        let vertex_buffer = fs::read_to_string(vertex_path)?;
        let vertex = compiler
            .compile_into_spirv(
                vertex_buffer.as_str(),
                ShaderKind::Vertex,
                vertex_path.as_str(),
                "main",
                Some(&options),
            )?
            .as_binary()
            .to_vec();

        let fragment_buffer = fs::read_to_string(fragment_path)?;
        let fragment = compiler
            .compile_into_spirv(
                fragment_buffer.as_str(),
                ShaderKind::Fragment,
                fragment_path.as_str(),
                "main",
                Some(&options),
            )?
            .as_binary()
            .to_vec();

        Ok(Arc::new(Shaders::new(vertex, fragment)))
    }

    pub fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn physical_index(&self) -> usize {
        self.physical_index
    }

    pub fn shaders(&self) -> Arc<Shaders> {
        self.shaders.clone()
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
