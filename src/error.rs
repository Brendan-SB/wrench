use vulkano::instance::InstanceCreationError;
use vulkano_win::CreationError;

pub enum Error {
    NoQueueFamily,
    NoQueue,
    NoPhysicalDevice,
    InstanceCreationError(InstanceCreationError),
    CreationError(CreationError),
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
