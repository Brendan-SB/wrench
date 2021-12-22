use crate::{assets::Transform, Vector4};
use std::sync::{Arc, Mutex};

pub struct Light {
    pub transform: Arc<Transform>,
    pub color: Mutex<Vector4<f32>>,
    pub directional: Mutex<bool>,
    pub intensity: Mutex<f32>,
    pub cutoff: Mutex<f32>,
    pub attenuation: Mutex<f32>,
}

impl Light {
    pub fn new(
        transform: Arc<Transform>,
        color: Vector4<f32>,
        directional: bool,
        intensity: f32,
        cutoff: f32,
        attenuation: f32,
    ) -> Arc<Self> {
        Arc::new(Self {
            transform: transform,
            color: Mutex::new(color),
            directional: Mutex::new(directional),
            intensity: Mutex::new(intensity),
            cutoff: Mutex::new(cutoff),
            attenuation: Mutex::new(attenuation),
        })
    }
}
