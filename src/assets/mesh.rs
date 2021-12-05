use crate::{InnerSpace, Vector3, Zero};
use std::sync::Arc;

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

vulkano::impl_vertex!(Normal, normal);

pub fn surface_normal(vertices: &Vec<Vertex>) -> Normal {
    let mut normal = Vector3::zero();

    for (i, current) in vertices.iter().enumerate() {
        let next = vertices[(i + 1) % vertices.len()];

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
}
