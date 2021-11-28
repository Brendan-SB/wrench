use crate::{ecs::reexports::*, Vector3};

#[derive(Component)]
pub struct Transform {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub up: Vector3<f32>,
    pub front: Vector3<f32>,
    pub right: Vector3<f32>,
    pub position: Vector3<f32>,
}
