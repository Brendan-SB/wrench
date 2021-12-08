use std::sync::Arc;

pub struct Material {
    pub ambient: f32,
}

impl Material {
    pub fn new(ambient: f32) -> Arc<Self> {
        Arc::new(Self {
            ambient,
        })
    }
}
