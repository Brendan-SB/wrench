use crate::{assets::Transform, ecs::reexports::*};

#[derive(Component)]
pub struct Camera {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub transform: Arc<Transform>,
    pub fov: Mutex<f32>,
    pub near: Mutex<f32>,
    pub far: Mutex<f32>,
}

impl Camera {
    pub fn new(
        id: Arc<String>,
        transform: Arc<Transform>,
        fov: f32,
        near: f32,
        far: f32,
    ) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: Arc::new("camera".to_string()),
            transform,
            fov: Mutex::new(fov),
            near: Mutex::new(near),
            far: Mutex::new(far),
        })
    }
}
