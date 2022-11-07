use crate::{
    assets::{Material, Mesh, Texture},
    components::{Camera, Light, Transform, TRANSFORM_ID},
    ecs::{self, reexports::*, Component, Entity},
    engine::InitializedEngine,
    shaders::{
        fragment::{self, MAX_LIGHTS},
        vertex,
    },
};
use cgmath::{EuclideanSpace, Matrix4, Point3, Rad, Vector3, Vector4, Zero};
use vulkano::{
    buffer::TypedBufferAccess,
    command_buffer::PrimaryAutoCommandBuffer,
    command_buffer::{pool::standard::StandardCommandPoolBuilder, AutoCommandBufferBuilder},
    descriptor_set::persistent::PersistentDescriptorSet,
    pipeline::{GraphicsPipeline, PipelineBindPoint},
};

pub const MODEL_ID: &str = "model";

pub struct ModelData {
    pub mesh: Arc<Mesh>,
    pub texture: Arc<Texture>,
    pub material: Arc<Material>,
    pub color: Vector4<f32>,
    pub visible: bool,
    pub lit: bool,
}

impl ModelData {
    pub fn new(
        mesh: Arc<Mesh>,
        texture: Arc<Texture>,
        material: Arc<Material>,
        color: Vector4<f32>,
        visible: bool,
        lit: bool,
    ) -> Self {
        Self {
            mesh,
            texture,
            material,
            color,
            visible,
            lit,
        }
    }
}

#[derive(Component)]
pub struct Model {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<RwLock<Option<Arc<Entity>>>>,
    pub data: RwLock<ModelData>,
}

impl Model {
    pub fn new(
        id: Arc<String>,
        mesh: Arc<Mesh>,
        texture: Arc<Texture>,
        material: Arc<Material>,
        color: Vector4<f32>,
        visible: bool,
        lit: bool,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id(MODEL_ID),
            entity: ecs::entity(None),
            data: RwLock::new(ModelData::new(mesh, texture, material, color, visible, lit)),
        })
    }

    pub fn draw(
        &self,
        initialized_engine: &mut InitializedEngine,
        camera: Arc<Camera>,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandPoolBuilder,
        >,
        pipeline: &GraphicsPipeline,
        lights: &Vec<Arc<Light>>,
        dimensions: &[u32; 2],
    ) {
        let data = self.data.read().unwrap();

        let entity = { self.entity.read().unwrap().clone() };
        let camera_entity = { camera.entity.read().unwrap().clone() };

        if let (Some(entity), Some(camera_entity)) = (entity, camera_entity) {
            if let (Some(transform), Some(camera_transform)) = (
                entity.get_first::<Transform>(ecs::id(TRANSFORM_ID)),
                camera_entity.get_first::<Transform>(ecs::id(TRANSFORM_ID)),
            ) {
                let transform_data = transform.calculate();
                let camera_transform_data = camera_transform.calculate();
                let uniform_buffer_subbuffer = {
                    let rotation = Matrix4::from_angle_z(Rad(transform_data.rotation.z))
                        * Matrix4::from_angle_y(Rad(transform_data.rotation.y))
                        * Matrix4::from_angle_x(Rad(transform_data.rotation.x));
                    let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                    let proj = {
                        let camera_data = camera.data.read().unwrap();

                        cgmath::perspective(
                            Rad(camera_data.fov),
                            aspect_ratio,
                            camera_data.near,
                            camera_data.far,
                        )
                    };
                    let camera_rotation =
                        Matrix4::from_angle_z(Rad(camera_transform_data.rotation.z))
                            * Matrix4::from_angle_y(Rad(camera_transform_data.rotation.y))
                            * Matrix4::from_angle_x(Rad(camera_transform_data.rotation.x));
                    let translation = Matrix4::from_translation(transform_data.position);
                    let camera_translation =
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
                        cam_rotation: camera_rotation.into(),
                        cam_translation: camera_translation.into(),
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
                        for (i, light) in lights.iter().enumerate() {
                            let light_entity = { light.entity.read().unwrap().clone() };

                            if let Some(light_entity) = light_entity {
                                if let Some(light_transform) =
                                    light_entity.get_first::<Transform>(ecs::id(TRANSFORM_ID))
                                {
                                    let light_transform_data = light_transform.calculate();
                                    let light_translation =
                                        Matrix4::from_translation(light_transform_data.position);
                                    let light_rotation =
                                        Matrix4::from_angle_z(Rad(light_transform_data.rotation.z))
                                            * Matrix4::from_angle_y(Rad(light_transform_data
                                                .rotation
                                                .y))
                                            * Matrix4::from_angle_x(Rad(light_transform_data
                                                .rotation
                                                .x));
                                    let proj = {
                                        let camera_data = camera.data.read().unwrap();

                                        cgmath::ortho(
                                            -camera_data.far,
                                            camera_data.far,
                                            -camera_data.far,
                                            camera_data.far,
                                            camera_data.near,
                                            camera_data.far,
                                        )
                                    };
                                    let look_at = Matrix4::look_at_rh(
                                        Point3::from_vec(light_transform_data.position),
                                        Point3::from_vec(Vector3::zero()),
                                        Vector3::new(0.0, 1.0, 0.0),
                                    );
                                    let light_data = light.data.read().unwrap();

                                    initialized_engine.lights_array[i] = fragment::ty::Light {
                                        position: light_translation.into(),
                                        rotation: light_rotation.into(),
                                        proj: (proj * look_at).into(),
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
                            len: lights.len().clamp(0, MAX_LIGHTS) as u32,
                            array: initialized_engine.lights_array,
                            _dummy0: [0; 12],
                        }
                    };

                    let uniform_data = {
                        fragment::ty::Data {
                            lit: data.lit.into(),
                            color: data.color.into(),
                            ambient: data.material.ambient,
                            diff_strength: data.material.diff_strength,
                            spec_strength: data.material.spec_strength,
                            spec_power: data.material.spec_power,
                            lights,
                            _dummy0: [0; 12],
                        }
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

                set_builder
                    .add_sampled_image(data.texture.image.clone(), data.texture.sampler.clone())
                    .unwrap();

                let image_set = Arc::new(set_builder.build().unwrap());

                builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        vec![set, image_set],
                    )
                    .bind_vertex_buffers(0, (data.mesh.vertices.clone(), data.mesh.normals.clone()))
                    .bind_index_buffer(data.mesh.indices.clone())
                    .draw_indexed(data.mesh.indices.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}
