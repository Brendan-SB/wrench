use std::sync::Arc;

pub struct Material {
    pub ambient: f32,
    pub diff_strength: f32,
    pub spec_strength: f32,
    pub spec_power: u32,
}

impl Material {
    pub fn new(ambient: f32, diff_strength: f32, spec_strength: f32, spec_power: u32) -> Arc<Self> {
        Arc::new(Self {
            ambient,
            diff_strength,
            spec_strength,
            spec_power,
        })
    }
}
