use crate::{assets::Light, components::Camera, ecs::Entity, Vector4};
use std::sync::{Arc, Mutex};

pub struct Scene {
    pub root: Arc<Entity>,
    pub camera: Mutex<Arc<Camera>>,
    pub lights: Mutex<Vec<Arc<Light>>>,
    pub bg: Mutex<Vector4<f32>>,
}

impl Scene {
    pub fn new(
        root: &Arc<Entity>,
        camera: Arc<Camera>,
        lights: Vec<Arc<Light>>,
        bg: Vector4<f32>,
    ) -> Arc<Self> {
        Arc::new(Self {
            root: root.clone(),
            camera: Mutex::new(camera),
            lights: Mutex::new(lights),
            bg: Mutex::new(bg),
        })
    }
}
