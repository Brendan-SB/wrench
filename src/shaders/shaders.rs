use super::{depth, fragment, vertex};
use crate::error::Error;
use std::sync::Arc;
use vulkano::device::Device;

pub struct Shaders {
    pub vertex: vertex::Shader,
    pub fragment: fragment::Shader,
    pub depth_vertex: depth::vertex::Shader,
    pub depth_fragment: depth::fragment::Shader,
}

impl Shaders {
    pub fn new(device: Arc<Device>) -> Result<Arc<Self>, Error> {
        Ok(Arc::new(Self {
            vertex: vertex::Shader::load(device.clone())?,
            fragment: fragment::Shader::load(device.clone())?,
            depth_vertex: depth::vertex::Shader::load(device.clone())?,
            depth_fragment: depth::fragment::Shader::load(device.clone())?,
        }))
    }
}
