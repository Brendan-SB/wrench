use crate::{assets::Light, components::Camera, ecs::World, Vector4};
use std::sync::{Arc, Mutex};

pub struct Scene {
    pub world: Arc<World>,
    pub camera: Mutex<Arc<Camera>>,
    pub lights: Mutex<Vec<Arc<Light>>>,
    pub bg: Mutex<Vector4<f32>>,
}

impl Scene {
    pub fn new(
        world: Arc<World>,
        camera: Arc<Camera>,
        lights: Vec<Arc<Light>>,
        bg: Vector4<f32>,
    ) -> Arc<Self> {
        Arc::new(Self {
            world,
            camera: Mutex::new(camera),
            lights: Mutex::new(lights),
            bg: Mutex::new(bg),
        })
    }
}
