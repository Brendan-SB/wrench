use std::sync::{Arc, Mutex};

pub struct Material {
    pub ambient: Mutex<f32>,
    pub diff_strength: Mutex<f32>,
    pub spec_strength: Mutex<f32>,
}

impl Material {
    pub fn new(ambient: f32, diff_strength: f32, spec_strength: f32) -> Arc<Self> {
        Arc::new(Self {
            ambient: Mutex::new(ambient),
            diff_strength: Mutex::new(diff_strength),
            spec_strength: Mutex::new(spec_strength),
        })
    }
}
