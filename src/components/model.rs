use crate::{
    assets::{mesh::Vertex, Mesh, Texture, Transform},
    ecs::{self, reexports::*},
    error::Error,
};
use obj::TexturedVertex;
use std::io::BufRead;

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

    pub fn from_obj<R>(
        id: Arc<String>,
        texture: Arc<Texture>,
        transform: Arc<Transform>,
        reader: R,
    ) -> Result<Arc<Self>, Error>
    where
        R: BufRead,
    {
        let obj = obj::load_obj(reader)?;
        let mesh = Mesh::auto(
            obj.vertices
                .into_iter()
                .map(|v: TexturedVertex| Vertex {
                    position: [v.position[0], v.position[1], v.position[2]],
                    uv: [v.texture[0], v.texture[1]],
                })
                .collect::<Vec<Vertex>>(),
            obj.indices,
        );

        Ok(Self::new(id, mesh, texture, transform))
    }
}
