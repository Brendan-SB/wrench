use crate::{
    ecs::{self, reexports::*},
    Vector4,
};
use std::sync::{Arc, Mutex};

#[derive(Component)]
pub struct Light {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub color: Mutex<Vector4<f32>>,
    pub directional: Mutex<bool>,
    pub intensity: Mutex<f32>,
    pub cutoff: Mutex<f32>,
    pub outer_cutoff: Mutex<f32>,
    pub attenuation: Mutex<f32>,
}

impl Light {
    pub fn new(
        id: Arc<String>,
        color: Vector4<f32>,
        directional: bool,
        intensity: f32,
        cutoff: f32,
        outer_cutoff: f32,
        attenuation: f32,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id("light"),
            entity: ecs::entity(None),
            color: Mutex::new(color),
            directional: Mutex::new(directional),
            intensity: Mutex::new(intensity),
            cutoff: Mutex::new(cutoff),
            outer_cutoff: Mutex::new(outer_cutoff),
            attenuation: Mutex::new(attenuation),
        })
    }
}
