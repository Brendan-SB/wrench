use crate::assets::Transform;
use std::sync::{Arc, Mutex};

pub struct Light {
    pub transform: Arc<Transform>,
    pub intensity: Mutex<f32>,
}

impl Light {
    pub fn new(transform: Arc<Transform>, intensity: f32) -> Arc<Self> {
        Arc::new(Self {
            transform,
            intensity: Mutex::new(intensity),
        })
    }
}
