use crate::{
    assets::{Mesh, Texture, Transform},
    ecs::{self, reexports::*},
};

#[derive(Component)]
pub struct Model {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub mesh: Arc<Mesh>,
    pub texture: Arc<Texture>,
    pub transform: Arc<Transform>,
}

impl Model {
    pub fn new(
        id: Arc<String>,
        mesh: Arc<Mesh>,
        texture: Arc<Texture>,
        transform: Arc<Transform>,
    ) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: ecs::id("model"),
            mesh,
            texture,
            transform,
        })
    }
}
