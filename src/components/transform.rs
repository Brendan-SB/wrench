use crate::{ecs::reexports::*, Vector3};

#[derive(Component)]
pub struct Transform {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
}

impl Transform {
    pub fn new(id: Arc<String>, position: Vector3<f32>, rotation: Vector3<f32>) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: Arc::new("transform".to_string()),
            position,
            rotation,
        })
    }
}
