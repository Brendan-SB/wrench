use crate::{Vector4, Vector3};
use std::sync::{Arc, Mutex};

pub struct Light {
    pub position: Mutex<Vector3<f32>>,
    pub color: Mutex<Vector4<f32>>,
    pub intensity: Mutex<f32>,
}

impl Light {
    pub fn new(position: Vector3<f32>, color: Vector4<f32>, intensity: f32) -> Arc<Self> {
        Arc::new(Self {
            position: Mutex::new(position),
            color: Mutex::new(color),
            intensity: Mutex::new(intensity),
        })
    }
}
