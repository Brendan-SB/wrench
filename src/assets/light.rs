use crate::{assets::Transform, Vector4};
use std::sync::{Arc, Mutex};

pub struct Light {
    pub transform: Arc<Transform>,
    pub color: Mutex<Vector4<f32>>,
    pub intensity: Mutex<f32>,
    pub spec_power: Mutex<u32>,
}

impl Light {
    pub fn new(
        transform: Arc<Transform>,
        color: Vector4<f32>,
        intensity: f32,
        spec_power: u32,
    ) -> Arc<Self> {
        Arc::new(Self {
            transform: transform,
            color: Mutex::new(color),
            intensity: Mutex::new(intensity),
            spec_power: Mutex::new(spec_power),
        })
    }
}
