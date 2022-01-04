pub mod default_engine;

pub use default_engine::DefaultEngine;

use crate::error::Error;

pub trait Engine {
    fn init(self) -> Result<(), Error>;
}
