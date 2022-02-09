use obj::ObjError;
use png::DecodingError;
use std::io;
use vulkano::{
    device::DeviceCreationError,
    image::{sys::ImageCreationError, view::ImageViewCreationError},
    instance::InstanceCreationError,
    memory::DeviceMemoryAllocError,
    pipeline::GraphicsPipelineCreationError,
    render_pass::RenderPassCreationError,
    swapchain::SwapchainCreationError,
    OomError,
};
use vulkano_win::CreationError;

#[derive(Debug)]
pub enum Error {
    NoQueueFamily,
    NoQueue,
    NoPhysicalDevice,
    NoShaderCompiler,
    NoShaderCompileOptions,
    NoSubpass,
    InstanceCreationError(InstanceCreationError),
    CreationError(CreationError),
    DeviceCreationError(DeviceCreationError),
    SwapchainCreationError(SwapchainCreationError),
    IoError(io::Error),
    OomError(OomError),
    RenderPassCreationError(RenderPassCreationError),
    GraphicsPipelineCreationError(GraphicsPipelineCreationError),
    DeviceMemoryAllocError(DeviceMemoryAllocError),
    ObjError(ObjError),
    ImageCreationError(ImageCreationError),
    ImageViewCreationError(ImageViewCreationError),
    DecodingError(DecodingError),
}

impl From<InstanceCreationError> for Error {
    fn from(e: InstanceCreationError) -> Self {
        Self::InstanceCreationError(e)
    }
}

impl From<CreationError> for Error {
    fn from(e: CreationError) -> Self {
        Self::CreationError(e)
    }
}

impl From<DeviceCreationError> for Error {
    fn from(e: DeviceCreationError) -> Self {
        Self::DeviceCreationError(e)
    }
}

impl From<SwapchainCreationError> for Error {
    fn from(e: SwapchainCreationError) -> Self {
        Self::SwapchainCreationError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<OomError> for Error {
    fn from(e: OomError) -> Self {
        Self::OomError(e)
    }
}

impl From<RenderPassCreationError> for Error {
    fn from(e: RenderPassCreationError) -> Self {
        Self::RenderPassCreationError(e)
    }
}

impl From<GraphicsPipelineCreationError> for Error {
    fn from(e: GraphicsPipelineCreationError) -> Self {
        Self::GraphicsPipelineCreationError(e)
    }
}

impl From<DeviceMemoryAllocError> for Error {
    fn from(e: DeviceMemoryAllocError) -> Self {
        Self::DeviceMemoryAllocError(e)
    }
}

impl From<ObjError> for Error {
    fn from(e: ObjError) -> Self {
        Self::ObjError(e)
    }
}

impl From<ImageCreationError> for Error {
    fn from(e: ImageCreationError) -> Self {
        Self::ImageCreationError(e)
    }
}

impl From<ImageViewCreationError> for Error {
    fn from(e: ImageViewCreationError) -> Self {
        Self::ImageViewCreationError(e)
    }
}

impl From<DecodingError> for Error {
    fn from(e: DecodingError) -> Self {
        Self::DecodingError(e)
    }
}
