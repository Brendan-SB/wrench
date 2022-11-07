use crate::{
    components::{Camera, Light, LIGHT_ID},
    ecs,
    ecs::{Entity, ENTITY_ID},
};
use cgmath::Vector4;
use std::sync::{Arc, RwLock};

pub struct Scene {
    pub root: Arc<Entity>,
    pub camera: RwLock<Arc<Camera>>,
    pub bg: RwLock<Vector4<f32>>,
}

impl Scene {
    pub fn new(root: &Arc<Entity>, camera: Arc<Camera>, bg: Vector4<f32>) -> Arc<Self> {
        Arc::new(Self {
            root: root.clone(),
            camera: RwLock::new(camera),
            bg: RwLock::new(bg),
        })
    }

    fn get_lights_inner(entity: Arc<Entity>, lights: &mut Vec<Arc<Light>>) {
        let mut l = entity.get_type(ecs::id(LIGHT_ID));

        lights.append(&mut l);

        for entity in &entity.get_type::<Entity>(ecs::id(ENTITY_ID)) {
            Self::get_lights_inner(entity.clone(), lights);
        }
    }

    pub fn get_lights(&self) -> Vec<Arc<Light>> {
        let mut lights = Vec::new();

        Self::get_lights_inner(self.root.clone(), &mut lights);

        lights
    }
}
