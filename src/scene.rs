use crate::{assets::Light, components::Camera, ecs::World};
use std::sync::{Arc, Mutex};

pub const MAX_LIGHTS: usize = 255;

pub struct Scene {
    pub world: Arc<World>,
    pub camera: Mutex<Arc<Camera>>,
    pub lights: Arc<Mutex<Vec<Arc<Light>>>>,
}

impl Scene {
    pub fn new(
        world: Arc<World>,
        camera: Arc<Camera>,
        lights: Arc<Mutex<Vec<Arc<Light>>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            world,
            camera: Mutex::new(camera),
            lights,
        })
    }
}
