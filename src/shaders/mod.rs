use crate::error::Error;
use std::sync::Arc;
use vulkano::device::Device;

pub mod fragment;
pub mod vertex;

pub struct Shaders {
    pub vertex: vertex::Shader,
    pub fragment: fragment::Shader,
}

impl Shaders {
    pub fn new(device: Arc<Device>) -> Result<Self, Error> {
        Ok(Self {
            vertex: vertex::Shader::load(device.clone())?,
            fragment: fragment::Shader::load(device.clone())?,
        })
    }
}
