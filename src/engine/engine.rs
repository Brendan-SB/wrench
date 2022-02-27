use crate::error::Error;

pub trait Engine {
    fn init(self) -> Result<(), Error>;
}
