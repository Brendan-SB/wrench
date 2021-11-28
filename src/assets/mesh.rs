use std::sync::Arc;

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: (f32, f32, f32),
}

vulkano::impl_vertex!(Vertex, position);

#[derive(Default, Copy, Clone)]
pub struct Normal {
    pub normal: (f32, f32, f32),
}

vulkano::impl_vertex!(Normal, normal);

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, normals: Vec<Normal>, indices: Vec<u32>) -> Arc<Self> {
        Arc::new(Self {
            vertices,
            normals,
            indices,
        })
    }
}
