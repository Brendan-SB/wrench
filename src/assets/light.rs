use crate::{assets::Transform, Vector4};
use std::sync::{Arc, Mutex};

pub struct Light {
    pub transform: Arc<Transform>,
    pub color: Mutex<Vector4<f32>>,
    pub intensity: Mutex<f32>,
}

impl Light {
    pub fn new(transform: Arc<Transform>, color: Vector4<f32>, intensity: f32) -> Arc<Self> {
        Arc::new(Self {
            transform: transform,
            color: Mutex::new(color),
            intensity: Mutex::new(intensity),
        })
    }
}
