use crate::{
    ecs::{self, reexports::*},
    Vector3, Zero,
};
use std::sync::{Arc, Mutex};

pub struct TransformData {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

#[derive(Component)]
pub struct Transform {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub data: Mutex<TransformData>,
}

impl Transform {
    pub fn new(
        id: Arc<String>,
        position: Vector3<f32>,
        rotation: Vector3<f32>,
        scale: Vector3<f32>,
    ) -> Arc<Self> {
        let data = Mutex::new(TransformData {
            position,
            rotation,
            scale,
        });

        Arc::new(Self {
            id,
            tid: ecs::id("transform"),
            entity: ecs::entity(None),
            data,
        })
    }

    pub fn scale_1(id: Arc<String>, position: Vector3<f32>, rotation: Vector3<f32>) -> Arc<Self> {
        Self::new(id, position, rotation, Vector3::new(1.0, 1.0, 1.0))
    }

    fn calculate_transform_inner(&self, data: &mut TransformData) {
        {
            let d = self.data.lock().unwrap();

            data.position += d.position;
            data.rotation += d.rotation;
            data.scale += d.scale;
        }

        let entity = { self.entity.lock().unwrap().clone() };

        if let Some(entity) = entity {
            let entity = { entity.entity.lock().unwrap().clone() };

            if let Some(entity) = entity {
                if let Some(transform) = entity.get_first::<Self>(ecs::id("transform")) {
                    transform.calculate_transform_inner(data);
                }
            }
        }
    }

    pub fn calculate_transform(&self) -> TransformData {
        let mut data = TransformData {
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: Vector3::zero(),
        };

        self.calculate_transform_inner(&mut data);

        data
    }
}
