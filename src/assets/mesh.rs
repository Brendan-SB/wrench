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
