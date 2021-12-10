use crate::Vector3;
use std::sync::{Arc, Mutex};

pub struct Transform {
    pub position: Mutex<Vector3<f32>>,
    pub rotation: Mutex<Vector3<f32>>,
    pub scale: Mutex<Vector3<f32>>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>) -> Arc<Self> {
        Arc::new(Self {
            position: Mutex::new(position),
            rotation: Mutex::new(rotation),
            scale: Mutex::new(scale),
        })
    }

    pub fn scale_1(position: Vector3<f32>, rotation: Vector3<f32>) -> Arc<Self> {
        Self::new(position, rotation, Vector3::new(1.0, 1.0, 1.0))
    }
}
