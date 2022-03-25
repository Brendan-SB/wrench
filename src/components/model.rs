use crate::{
    assets::{Material, Mesh, Texture},
    components::{Camera, Light, Transform},
    ecs::{self, reexports::*, Component, Entity},
    engine::default_engine::InitializedDefaultEngine,
    shaders::{depth, fragment, vertex},
    EuclideanSpace, Matrix4, Point3, Rad, Vector3, Vector4, Zero,
};
use std::sync::{Arc, Mutex};
use vulkano::{
    buffer::TypedBufferAccess,
    command_buffer::PrimaryAutoCommandBuffer,
    command_buffer::{pool::standard::StandardCommandPoolBuilder, AutoCommandBufferBuilder},
    descriptor_set::persistent::PersistentDescriptorSet,
    device::Device,
    image::{view::ImageView, AttachmentImage},
    pipeline::{GraphicsPipeline, PipelineBindPoint},
    sampler::{BorderColor, Filter, MipmapMode, Sampler, SamplerAddressMode},
};

pub struct ModelData {
    pub mesh: Arc<Mesh>,
    pub texture: Arc<Texture>,
    pub material: Arc<Material>,
    pub color: Vector4<f32>,
    pub shadowed: bool,
}

impl ModelData {
    pub fn new(
        mesh: Arc<Mesh>,
        texture: Arc<Texture>,
        material: Arc<Material>,
        color: Vector4<f32>,
        shadowed: bool,
    ) -> Self {
        Self {
            mesh,
            texture,
            material,
            color,
            shadowed,
        }
    }
}

#[derive(Component)]
pub struct Model {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub data: Mutex<ModelData>,
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
            data: Mutex::new(ModelData::new(mesh, texture, material, color, shadowed)),
        })
    }

    pub fn draw_shadows(
        &self,
        initialized_engine: &InitializedDefaultEngine,
        light: Arc<Light>,
        camera: Arc<Camera>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandPoolBuilder,
        >,
        pipeline: &GraphicsPipeline,
    ) {
        let data = self.data.lock().unwrap();

        if data.shadowed {
            let entity = { self.entity.lock().unwrap().clone() };

            if let Some(entity) = entity {
                let light_entity = { light.entity.lock().unwrap().clone() };

                if let Some(light_entity) = light_entity {
                    if let (Some(transform), Some(light_transform)) = (
                        entity.get_first::<Transform>(ecs::id("transform")),
                        light_entity.get_first::<Transform>(ecs::id("transform")),
                    ) {
                        let transform_data = transform.calculate();
                        let light_transform_data = light_transform.calculate();
                        let uniform_buffer_subbuffer = {
                            let rotation = Matrix4::from_angle_x(Rad(transform_data.rotation.x))
                                * Matrix4::from_angle_y(Rad(transform_data.rotation.y))
                                * Matrix4::from_angle_z(Rad(transform_data.rotation.z));
                            let proj = {
                                let camera_data = camera.data.lock().unwrap();

                                cgmath::ortho(
                                    -camera_data.far,
                                    camera_data.far,
                                    -camera_data.far,
                                    camera_data.far,
                                    -camera_data.far,
                                    camera_data.far,
                                )
                            };
                            let light_rotation =
                                Matrix4::from_angle_x(Rad(light_transform_data.rotation.x))
                                    * Matrix4::from_angle_y(Rad(light_transform_data.rotation.y))
                                    * Matrix4::from_angle_z(Rad(light_transform_data.rotation.z));
                            let translation = Matrix4::from_translation(transform_data.position);
                            let light_translation =
                                Matrix4::from_translation(light_transform_data.position);
                            let scale = Matrix4::from_nonuniform_scale(
                                transform_data.scale.x,
                                transform_data.scale.y,
                                transform_data.scale.z,
                            );
                            let look_at = Matrix4::look_at_lh(
                                Point3::from_vec(light_transform_data.position),
                                Point3::from_vec(Vector3::zero()),
                                Vector3::new(0.0, 1.0, 0.0),
                            );
                            let uniform_data = depth::vertex::ty::Data {
                                proj: proj.into(),
                                scale: scale.into(),
                                transform: (rotation * look_at * translation).into(),
                                cam_transform: (light_rotation * light_translation).into(),
                            };

                            Arc::new(
                                initialized_engine
                                    .depth_uniform_buffer
                                    .next(uniform_data)
                                    .unwrap(),
                            )
                        };

                        let set_layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();
                        let mut set_builder = PersistentDescriptorSet::start(set_layout.clone());

                        set_builder.add_buffer(uniform_buffer_subbuffer).unwrap();

                        let set = Arc::new(set_builder.build().unwrap());

                        builder
                            .bind_descriptor_sets(
                                PipelineBindPoint::Graphics,
                                pipeline.layout().clone(),
                                0,
                                set.clone(),
                            )
                            .bind_vertex_buffers(
                                0,
                                (data.mesh.vertices.clone(), data.mesh.normals.clone()),
                            )
                            .bind_index_buffer(data.mesh.indices.clone())
                            .draw_indexed(data.mesh.indices.len() as u32, 1, 0, 0, 0)
                            .unwrap();
                    }
                }
            }
        }
    }

    pub fn draw(
        &self,
        initialized_engine: &mut InitializedDefaultEngine,
        camera: Arc<Camera>,
        device: Arc<Device>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandPoolBuilder,
        >,
        pipeline: &GraphicsPipeline,
        lights: &Option<Vec<Arc<Light>>>,
        shadow_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
        dimensions: &[u32; 2],
    ) {
        let data = self.data.lock().unwrap();
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
                        let proj = {
                            let camera_data = camera.data.lock().unwrap();

                            cgmath::perspective(
                                Rad(camera_data.fov),
                                aspect_ratio,
                                camera_data.near,
                                camera_data.far,
                            )
                        };
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
                            transform: (rotation * translation).into(),
                            cam_transform: (cam_rotation * cam_translation).into(),
                        };

                        Arc::new(
                            initialized_engine
                                .uniform_buffer
                                .next(uniform_data)
                                .unwrap(),
                        )
                    };

                    let frag_uniform_buffer_subbuffer = {
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
                                                let light_data = light.data.lock().unwrap();

                                                initialized_engine.lights_array[i] =
                                                    fragment::ty::Light {
                                                        position: position.into(),
                                                        rotation: rotation.into(),
                                                        color: light_data.color.into(),
                                                        directional: light_data.directional as u32,
                                                        intensity: light_data.intensity,
                                                        cutoff: light_data.cutoff,
                                                        outer_cutoff: light_data.outer_cutoff,
                                                        attenuation: light_data.attenuation,
                                                    };
                                            }
                                        }
                                    }

                                    fragment::ty::LightArray {
                                        len: lights.len() as u32,
                                        array: initialized_engine.lights_array,
                                        _dummy0: [0; 12],
                                    }
                                }

                                None => fragment::ty::LightArray {
                                    len: 0 as u32,
                                    array: initialized_engine.lights_array,
                                    _dummy0: [0; 12],
                                },
                            }
                        };

                        let uniform_data = fragment::ty::Data {
                            color: data.color.into(),
                            ambient: data.material.ambient,
                            diff_strength: data.material.diff_strength,
                            spec_strength: data.material.spec_strength,
                            spec_power: data.material.spec_power,
                            lights,
                        };

                        Arc::new(
                            initialized_engine
                                .frag_uniform_buffer
                                .next(uniform_data)
                                .unwrap(),
                        )
                    };
                    let descriptor_set_layouts = pipeline.layout().descriptor_set_layouts();
                    let set_layout = descriptor_set_layouts.get(0).unwrap();
                    let mut set_builder = PersistentDescriptorSet::start(set_layout.clone());

                    set_builder
                        .add_buffer(uniform_buffer_subbuffer)
                        .unwrap()
                        .add_buffer(frag_uniform_buffer_subbuffer)
                        .unwrap();

                    let set_layout = descriptor_set_layouts.get(1).unwrap();
                    let set = Arc::new(set_builder.build().unwrap());
                    let mut set_builder = PersistentDescriptorSet::start(set_layout.clone());
                    let sampler = Sampler::new(
                        device.clone(),
                        Filter::Linear,
                        Filter::Linear,
                        MipmapMode::Nearest,
                        SamplerAddressMode::ClampToBorder(BorderColor::FloatOpaqueBlack),
                        SamplerAddressMode::ClampToBorder(BorderColor::FloatOpaqueBlack),
                        SamplerAddressMode::ClampToBorder(BorderColor::FloatOpaqueBlack),
                        0.0,
                        1.0,
                        1.0,
                        1.0,
                    )
                    .unwrap();

                    set_builder
                        .add_sampled_image(data.texture.image.clone(), data.texture.sampler.clone())
                        .unwrap()
                        .add_sampled_image(shadow_buffer, sampler)
                        .unwrap();

                    let image_set = Arc::new(set_builder.build().unwrap());

                    builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            vec![set.clone(), image_set.clone()],
                        )
                        .bind_vertex_buffers(
                            0,
                            (data.mesh.vertices.clone(), data.mesh.normals.clone()),
                        )
                        .bind_index_buffer(data.mesh.indices.clone())
                        .draw_indexed(data.mesh.indices.len() as u32, 1, 0, 0, 0)
                        .unwrap();
                }
            }
        }
    }
}
