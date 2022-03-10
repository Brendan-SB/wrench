use crate::ecs::{self, reexports::*};

pub struct CameraData {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraData {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self { fov, near, far }
    }
}

#[derive(Component)]
pub struct Camera {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub data: Mutex<CameraData>,
}

impl Camera {
    pub fn new(id: Arc<String>, fov: f32, near: f32, far: f32) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id("camera"),
            entity: ecs::entity(None),
            data: Mutex::new(CameraData::new(fov, near, far)),
        })
    }
}
