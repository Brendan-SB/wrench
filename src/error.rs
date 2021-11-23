use std::io;
use vulkano::{device::DeviceCreationError, instance::InstanceCreationError};
use vulkano_win::CreationError;

pub enum Error {
    NoQueueFamily,
    NoQueue,
    NoPhysicalDevice,
    InstanceCreationError(InstanceCreationError),
    CreationError(CreationError),
    DeviceCreationError(DeviceCreationError),
    IoError(io::Error),
    ShaderCError(shaderc::Error),
    NoShaderCompiler,
    NoShaderCompilerOptions,
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

impl From<shaderc::Error> for Error {
    fn from(e: shaderc::Error) -> Self {
        Self::ShaderCError(e)
    }
}
