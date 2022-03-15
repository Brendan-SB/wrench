use crate::error::Error;
use obj::TexturedVertex;
use std::{io::BufRead, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
};

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
    pub vertices: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u32]>>,
    pub normals: Arc<CpuAccessibleBuffer<[Normal]>>,
}

impl Mesh {
    pub fn new(
        device: Arc<Device>,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        normals: Vec<Normal>,
    ) -> Arc<Self> {
        let normals = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            normals.iter().cloned(),
        )
        .unwrap();
        let vertices = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            vertices.iter().cloned(),
        )
        .unwrap();
        let indices = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            indices.iter().cloned(),
        )
        .unwrap();
        Arc::new(Self {
            vertices,
            indices,
            normals,
        })
    }

    pub fn from_obj<R>(device: Arc<Device>, reader: R) -> Result<Arc<Self>, Error>
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

        Ok(Self::new(device, vertices, obj.indices, normals))
    }
}
