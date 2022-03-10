use crate::{
    ecs::{self, reexports::*},
    Vector3,
};
use std::sync::{Arc, Mutex};

pub struct LightData {
    pub color: Vector3<f32>,
    pub directional: bool,
    pub intensity: f32,
    pub cutoff: f32,
    pub outer_cutoff: f32,
    pub attenuation: f32,
}

impl LightData {
    pub fn new(
        color: Vector3<f32>,
        directional: bool,
        intensity: f32,
        cutoff: f32,
        outer_cutoff: f32,
        attenuation: f32,
    ) -> Self {
        Self {
            color,
            directional,
            intensity,
            cutoff,
            outer_cutoff,
            attenuation,
        }
    }
}

#[derive(Component)]
pub struct Light {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub data: Mutex<LightData>,
}

impl Light {
    pub fn new(
        id: Arc<String>,
        color: Vector3<f32>,
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
            data: Mutex::new(LightData::new(
                color,
                directional,
                intensity,
                cutoff,
                outer_cutoff,
                attenuation,
            )),
        })
    }
}
