use crate::{
    assets::{Material, Mesh, Texture},
    components::{Camera, Light, Transform},
    ecs::{self, reexports::*, Component, Entity},
    shaders::{fragment, vertex},
    Matrix4, Rad, Vector4,
};
use std::sync::{Arc, Mutex};
use vulkano::{
    buffer::{cpu_pool::CpuBufferPool, BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::PrimaryAutoCommandBuffer,
    command_buffer::{pool::standard::StandardCommandPoolBuilder, AutoCommandBufferBuilder},
    descriptor_set::persistent::PersistentDescriptorSet,
    device::Device,
    pipeline::{GraphicsPipeline, PipelineBindPoint},
};

#[derive(Component)]
pub struct Model {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub mesh: Mutex<Arc<Mesh>>,
    pub texture: Mutex<Arc<Texture>>,
    pub material: Mutex<Arc<Material>>,
    pub color: Mutex<Vector4<f32>>,
    pub shadowed: Mutex<bool>,
}

impl Model {
    pub fn new(
        id: Arc<String>,
        mesh: Arc<Mesh>,
        texture: Arc<Texture>,
        material: Arc<Material>,
        color: Vector4<f32>,
        shadowed: bool,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id("model"),
            entity: ecs::entity(None),
            mesh: Mutex::new(mesh),
            texture: Mutex::new(texture),
            material: Mutex::new(material),
            color: Mutex::new(color),
            shadowed: Mutex::new(shadowed),
        })
    }

    pub fn draw(
        &self,
        camera: Arc<Camera>,
        device: Arc<Device>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandPoolBuilder,
        >,
        pipeline: &GraphicsPipeline,
        suboptimal: bool,
        recreate_swapchain: &mut bool,
        lights_array: &mut [fragment::ty::Light; 1024],
        lights: &Option<Vec<Arc<Light>>>,
        uniform_buffer: &CpuBufferPool<vertex::ty::Data>,
        frag_uniform_buffer: &CpuBufferPool<fragment::ty::Data>,
        dimensions: &[u32; 2],
    ) {
        let entity = { self.entity.lock().unwrap().clone() };

        if let Some(entity) = entity {
            let camera_entity = { camera.entity.lock().unwrap().clone() };

            if let Some(camera_entity) = camera_entity {
                if let (Some(transform), Some(camera_transform)) = (
                    entity.get_first::<Transform>(ecs::id("transform")),
                    camera_entity.get_first::<Transform>(ecs::id("transform")),
                ) {
                    let transform_data = transform.calculate();
                    let camera_transform_data = camera_transform.calculate();
                    let uniform_buffer_subbuffer = {
                        let rotation = Matrix4::from_angle_x(Rad(transform_data.rotation.x))
                            * Matrix4::from_angle_y(Rad(transform_data.rotation.y))
                            * Matrix4::from_angle_z(Rad(transform_data.rotation.z));
                        let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                        let proj = cgmath::perspective(
                            Rad(*camera.fov.lock().unwrap()),
                            aspect_ratio,
                            *camera.near.lock().unwrap(),
                            *camera.far.lock().unwrap(),
                        );
                        let cam_rotation =
                            Matrix4::from_angle_x(Rad(camera_transform_data.rotation.x))
                                * Matrix4::from_angle_y(Rad(camera_transform_data.rotation.y))
                                * Matrix4::from_angle_z(Rad(camera_transform_data.rotation.z));
                        let translation = Matrix4::from_translation(transform_data.position);
                        let cam_translation =
                            Matrix4::from_translation(camera_transform_data.position);
                        let scale = Matrix4::from_nonuniform_scale(
                            transform_data.scale.x,
                            transform_data.scale.y,
                            transform_data.scale.z,
                        );
                        let uniform_data = vertex::ty::Data {
                            proj: proj.into(),
                            scale: scale.into(),
                            translation: translation.into(),
                            rotation: rotation.into(),
                            cam_translation: cam_translation.into(),
                            cam_rotation: cam_rotation.into(),
                        };

                        Arc::new(uniform_buffer.next(uniform_data).unwrap())
                    };

                    let frag_uniform_buffer_subbuffer = {
                        let material = { self.material.lock().unwrap().clone() };
                        let lights = {
                            match lights {
                                Some(lights) => {
                                    for (i, light) in lights.iter().enumerate() {
                                        let light_entity = { light.entity.lock().unwrap().clone() };

                                        if let Some(light_entity) = light_entity {
                                            if let Some(light_transform) = light_entity
                                                .get_first::<Transform>(ecs::id("transform"))
                                            {
                                                let light_transform_data =
                                                    light_transform.calculate();
                                                let position = Matrix4::from_translation(
                                                    light_transform_data.position,
                                                );
                                                let rotation = Matrix4::from_angle_x(Rad(
                                                    light_transform_data.rotation.x,
                                                )) * Matrix4::from_angle_y(Rad(
                                                    light_transform_data.rotation.y,
                                                )) * Matrix4::from_angle_z(Rad(
                                                    light_transform_data.rotation.z,
                                                ));

                                                lights_array[i] = fragment::ty::Light {
                                                    position: position.into(),
                                                    rotation: rotation.into(),
                                                    color: (*light.color.lock().unwrap()).into(),
                                                    directional: *light.directional.lock().unwrap()
                                                        as u32,
                                                    intensity: *light.intensity.lock().unwrap(),
                                                    cutoff: *light.cutoff.lock().unwrap(),
                                                    outer_cutoff: *light
                                                        .outer_cutoff
                                                        .lock()
                                                        .unwrap(),
                                                    attenuation: *light.attenuation.lock().unwrap(),
                                                };
                                            }
                                        }
                                    }

                                    fragment::ty::LightArray {
                                        len: lights.len() as u32,
                                        array: *lights_array,
                                        _dummy0: [0; 12],
                                    }
                                }

                                None => fragment::ty::LightArray {
                                    len: 0 as u32,
                                    array: *lights_array,
                                    _dummy0: [0; 12],
                                },
                            }
                        };

                        let uniform_data = fragment::ty::Data {
                            color: (*self.color.lock().unwrap()).into(),
                            ambient: *material.ambient.lock().unwrap(),
                            diff_strength: *material.diff_strength.lock().unwrap(),
                            spec_strength: *material.spec_strength.lock().unwrap(),
                            spec_power: *material.spec_power.lock().unwrap(),
                            lights,
                        };

                        Arc::new(frag_uniform_buffer.next(uniform_data).unwrap())
                    };
                    let set_layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();
                    let mut set_builder = PersistentDescriptorSet::start(set_layout.clone());
                    let texture = self.texture.lock().unwrap();

                    set_builder
                        .add_buffer(uniform_buffer_subbuffer)
                        .unwrap()
                        .add_sampled_image(texture.image.clone(), texture.sampler.clone())
                        .unwrap()
                        .add_buffer(frag_uniform_buffer_subbuffer)
                        .unwrap();

                    let set = Arc::new(set_builder.build().unwrap());

                    if suboptimal {
                        *recreate_swapchain = true;
                    }

                    let mesh = self.mesh.lock().unwrap();
                    let normal_buffer = CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        mesh.normals.iter().cloned(),
                    )
                    .unwrap();
                    let vertex_buffer = CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        mesh.vertices.iter().cloned(),
                    )
                    .unwrap();
                    let index_buffer = CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        mesh.indices.iter().cloned(),
                    )
                    .unwrap();

                    builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            set.clone(),
                        )
                        .bind_vertex_buffers(0, (vertex_buffer.clone(), normal_buffer.clone()))
                        .bind_index_buffer(index_buffer.clone())
                        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                        .unwrap();
                }
            }
        }
    }
}
