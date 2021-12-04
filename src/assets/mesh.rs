use crate::{InnerSpace, Vector3};
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

pub fn gen_normals(vertices: &Vec<Vertex>) -> Vec<Normal> {
    let mut normals = Vec::new();

    for window in vertices.windows(2) {
        let normal = Normal {
            normal: Vector3::new(
                (window[0].position[1] - window[1].position[1])
                    * (window[0].position[2] + window[1].position[2]),
                (window[0].position[2] - window[1].position[2])
                    * (window[0].position[0] + window[1].position[0]),
                (window[0].position[0] - window[1].position[0])
                    * (window[0].position[1] + window[1].position[1]),
            )
            .normalize()
            .into(),
        };

        normals.push(normal);
    }

    normals
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

    pub fn auto(vertices: Vec<Vertex>, indices: Vec<u32>) -> Arc<Self> {
        let normals = gen_normals(&vertices);

        Self::new(vertices, indices, normals)
    }
}
