use crate::{assets, ecs::reexports::*};

#[derive(Component)]
pub struct Mesh {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub asset: Arc<assets::Mesh>,
}

impl Mesh {
    pub fn new(id: Arc<String>, asset: Arc<assets::Mesh>) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: Arc::new("mesh".to_string()),
            asset,
        })
    }
}
