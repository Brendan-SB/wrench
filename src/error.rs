use std::io;
use vulkano::{
    device::DeviceCreationError, instance::InstanceCreationError,
    pipeline::GraphicsPipelineCreationError, render_pass::RenderPassCreationError, OomError,
};
use vulkano_win::CreationError;

pub enum Error {
    NoQueueFamily,
    NoQueue,
    NoPhysicalDevice,
    InstanceCreationError(InstanceCreationError),
    CreationError(CreationError),
    DeviceCreationError(DeviceCreationError),
    IoError(io::Error),
    OomError(OomError),
    RenderPassCreationError(RenderPassCreationError),
    GraphicsPipelineCreationError(GraphicsPipelineCreationError),
    NoShaderCompiler,
    NoShaderCompileOptions,
    NoSubpass,
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
