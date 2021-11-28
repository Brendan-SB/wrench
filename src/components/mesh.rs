use crate::{
    ecs::reexports::*,
    types::{Normal, Vertex},
};

#[derive(Component)]
pub struct Mesh {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, normals: Vec<Normal>, indices: Vec<u32>) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id: Arc::new("mesh".to_string()),
            tid: Arc::new("mesh".to_string()),
            vertices,
            normals,
            indices,
        })
    }
}
