use crate::{components::Camera, ecs::World};
use std::sync::{Arc, Mutex};

pub const MAX_LIGHTS: usize = 255;

pub struct Scene {
    pub world: Arc<World>,
    pub camera: Mutex<Arc<Camera>>,
}

impl Scene {
    pub fn new(world: Arc<World>, camera: Arc<Camera>) -> Arc<Self> {
        Arc::new(Self {
            world,
            camera: Mutex::new(camera),
        })
    }
}
