use obj::ObjError;
use std::{error, io};
use vulkano::{
    device::DeviceCreationError, image::view::ImageViewCreationError,
    instance::InstanceCreationError, memory::DeviceMemoryAllocError,
    pipeline::GraphicsPipelineCreationError, render_pass::RenderPassCreationError, OomError,
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
    IoError(io::Error),
    OomError(OomError),
    RenderPassCreationError(RenderPassCreationError),
    GraphicsPipelineCreationError(GraphicsPipelineCreationError),
    DeviceMemoryAllocError(DeviceMemoryAllocError),
    ObjError(ObjError),
    ImageViewCreationError(ImageViewCreationError),
    Error(Box<dyn error::Error>),
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

impl From<ImageViewCreationError> for Error {
    fn from(e: ImageViewCreationError) -> Self {
        Self::ImageViewCreationError(e)
    }
}

impl From<Box<dyn error::Error>> for Error {
    fn from(e: Box<dyn error::Error>) -> Self {
        Self::Error(e)
    }
}
