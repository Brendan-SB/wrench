use crate::{
    ecs::{self, reexports::*},
    Vector3,
};

pub const LIGHT_ID: &str = "light";

pub struct LightData {
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub cutoff: f32,
    pub outer_cutoff: f32,
    pub attenuation: f32,
    pub directional: bool,
}

impl LightData {
    pub fn new(
        color: Vector3<f32>,
        intensity: f32,
        cutoff: f32,
        outer_cutoff: f32,
        attenuation: f32,
        directional: bool,
    ) -> Self {
        Self {
            color,
            intensity,
            cutoff,
            outer_cutoff,
            attenuation,
            directional,
        }
    }
}

#[derive(Component)]
pub struct Light {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<RwLock<Option<Arc<Entity>>>>,
    pub data: RwLock<LightData>,
}

impl Light {
    pub fn new(
        id: Arc<String>,
        color: Vector3<f32>,
        intensity: f32,
        cutoff: f32,
        outer_cutoff: f32,
        attenuation: f32,
        directional: bool,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id(LIGHT_ID),
            entity: ecs::entity(None),
            data: RwLock::new(LightData::new(
                color,
                intensity,
                cutoff,
                outer_cutoff,
                attenuation,
                directional,
            )),
        })
    }
}
