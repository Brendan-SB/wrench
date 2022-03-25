use crate::error::Error;
use cgmath::{InnerSpace, Vector3};
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

impl Normal {
    pub fn from_vertices(vertices: &[&Vertex]) -> Option<Self> {
        match (vertices.get(0), vertices.get(1), vertices.get(2)) {
            (Some(v1), Some(v2), Some(v3)) => {
                let v1 = Vector3::from(v1.position);
                let v2 = Vector3::from(v2.position);
                let v3 = Vector3::from(v3.position);
                let a = v1 - v2;
                let b = v1 - v3;
                let n = a.cross(b).normalize();
                let normal = Self { normal: n.into() };

                Some(normal)
            }
            _ => None,
        }
    }
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
        vertices: &[Vertex],
        indices: &[u32],
        normals: &[Normal],
    ) -> Result<Arc<Self>, Error> {
        let normals = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            normals.iter().cloned(),
        )?;
        let vertices = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            vertices.iter().cloned(),
        )?;
        let indices = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            indices.iter().cloned(),
        )?;

        Ok(Arc::new(Self {
            vertices,
            indices,
            normals,
        }))
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

        Self::new(device, &vertices, &obj.indices, &normals)
    }
}
