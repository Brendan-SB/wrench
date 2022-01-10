use crate::{error::Error, InnerSpace, Vector3, Zero};
use obj::TexturedVertex;
use std::{io::BufRead, ops::Deref, sync::Arc};

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position, uv);

#[derive(Default, Copy, Clone)]
pub struct Normal {
    pub normal: [f32; 3],
}

impl<V> From<V> for Normal
where
    V: Deref<Target = Vec<Vertex>>,
{
    fn from(vertices: V) -> Self {
        let mut normal = Vector3::zero();

        for (i, current) in vertices.iter().enumerate() {
            let next = vertices[(i + 1) % (vertices.len() - 1)];

            normal += Vector3::new(
                (current.position[1] - next.position[1]) * (current.position[2] + next.position[2]),
                (current.position[2] - next.position[2]) * (current.position[0] + next.position[0]),
                (current.position[0] - next.position[0]) * (current.position[1] + next.position[1]),
            );
        }

        Normal {
            normal: normal.normalize().into(),
        }
    }
}

vulkano::impl_vertex!(Normal, normal);

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub normals: Vec<Normal>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, normals: Vec<Normal>) -> Arc<Self> {
        Arc::new(Self {
            vertices,
            indices,
            normals,
        })
    }

    pub fn from_obj<R>(reader: R) -> Result<Arc<Self>, Error>
    where
        R: BufRead,
    {
        let obj = obj::load_obj(reader)?;
        let mut vertices = Vec::new();
        let mut normals = Vec::new();

        for vertex in obj.vertices as Vec<TexturedVertex> {
            vertices.push(Vertex {
                position: vertex.position,
                uv: [vertex.texture[0], vertex.texture[1]],
            });
            normals.push(Normal {
                normal: vertex.normal,
            });
        }

        Ok(Self::new(vertices, obj.indices, normals))
    }
}
