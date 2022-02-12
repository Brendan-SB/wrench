use crate::{
    components::{Camera, Light},
    ecs,
    ecs::Entity,
    Vector4,
};
use std::sync::{Arc, Mutex};

pub struct Scene {
    pub root: Arc<Entity>,
    pub camera: Mutex<Arc<Camera>>,
    pub bg: Mutex<Vector4<f32>>,
}

impl Scene {
    pub fn new(root: &Arc<Entity>, camera: Arc<Camera>, bg: Vector4<f32>) -> Arc<Self> {
        Arc::new(Self {
            root: root.clone(),
            camera: Mutex::new(camera),
            bg: Mutex::new(bg),
        })
    }

    fn get_lights_inner(&self, entity: Arc<Entity>, lights: &mut Option<Vec<Arc<Light>>>) {
        let l = entity.get_type(ecs::id("light"));

        if let Some(l) = l {
            let mut l = (*l).clone();

            match lights {
                Some(lights) => {
                    lights.append(&mut l);
                }

                None => *lights = Some(l),
            }
        }

        if let Some(entities) = entity.get_type::<Entity>(ecs::id("entity")) {
            for entity in &*entities {
                self.get_lights_inner(entity.clone(), lights);
            }
        }
    }

    pub fn get_lights(&self) -> Option<Vec<Arc<Light>>> {
        let mut lights = None;

        self.get_lights_inner(self.root.clone(), &mut lights);

        lights
    }
}
