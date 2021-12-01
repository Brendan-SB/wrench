use crate::{assets, ecs::reexports::*};

#[derive(Component)]
pub struct Model {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub asset: Arc<assets::Mesh>,
    pub texture: Arc<assets::Texture>
}

impl Model {
    pub fn new(id: Arc<String>, asset: Arc<assets::Mesh>, texture: Arc<assets::Texture>) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: Arc::new("model".to_string()),
            asset,
            texture,
        })
    }
}
