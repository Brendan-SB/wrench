use crate::ecs::{self, reexports::*};

#[derive(Component)]
pub struct Camera {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub fov: Mutex<f32>,
    pub near: Mutex<f32>,
    pub far: Mutex<f32>,
}

impl Camera {
    pub fn new(id: Arc<String>, fov: f32, near: f32, far: f32) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id("camera"),
            entity: ecs::entity(None),
            fov: Mutex::new(fov),
            near: Mutex::new(near),
            far: Mutex::new(far),
        })
    }
}
