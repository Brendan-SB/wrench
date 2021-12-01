use crate::{
    assets::{Mesh, Texture, Transform},
    ecs::reexports::*,
};

#[derive(Component)]
pub struct Model {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub asset: Arc<Mesh>,
    pub texture: Arc<Texture>,
    pub transform: Arc<Transform>,
}

impl Model {
    pub fn new(
        id: Arc<String>,
        asset: Arc<Mesh>,
        texture: Arc<Texture>,
        transform: Arc<Transform>,
    ) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: Arc::new("model".to_string()),
            asset,
            texture,
            transform,
        })
    }
}
