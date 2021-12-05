use crate::{
    assets::{
        mesh::{Normal, Vertex},
        Mesh, Texture, Transform,
    },
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

        let mut vertices = Vec::new();
        let mut normals = Vec::new();

        for vertex in obj.vertices as Vec<TexturedVertex> {
            let mut position = [0.0; 3];

            for i in 0..3 {
                position[i] = vertex.position[i];
            }

            let mut uv = [0.0; 2];

            for i in 0..2 {
                uv[i] = vertex.texture[i];
            }

            vertices.push(Vertex { position, uv });

            let mut normal = [0.0; 3];

            for i in 0..3 {
                normal[i] = vertex.normal[i];
            }

            normals.push(Normal { normal });
        }

        let mesh = Mesh::new(vertices, obj.indices, normals);

        Ok(Self::new(id, mesh, texture, transform))
    }
}
