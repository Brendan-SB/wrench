use std::sync::Arc;

pub struct Material {
    pub ambient: f32,
    pub reflectivity: f32,
}

impl Material {
    pub fn new(ambient: f32, reflectivity: f32) -> Arc<Self> {
        Arc::new(Self {
            ambient,
            reflectivity,
        })
    }
}
