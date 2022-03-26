use crate::{
    ecs::{self, reexports::*},
    Vector3, Zero,
};

pub struct TransformData {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl TransformData {
    pub fn new(position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

#[derive(Component)]
pub struct Transform {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<RwLock<Option<Arc<Entity>>>>,
    pub data: RwLock<TransformData>,
}

impl Transform {
    pub fn new(
        id: Arc<String>,
        position: Vector3<f32>,
        rotation: Vector3<f32>,
        scale: Vector3<f32>,
    ) -> Arc<Self> {
        let data = RwLock::new(TransformData::new(position, rotation, scale));

        Arc::new(Self {
            id,
            tid: ecs::id("transform"),
            entity: ecs::entity(None),
            data,
        })
    }

    fn calculate_inner(&self, data: &mut TransformData) {
        {
            let d = self.data.read().unwrap();

            data.position += d.position;
            data.rotation += d.rotation;
            data.scale += d.scale;
        }

        let entity = self.entity.read().unwrap();

        if let Some(entity) = &*entity {
            let entity = entity.entity.read().unwrap();

            if let Some(entity) = &*entity {
                if let Some(transform) = entity.get_first::<Self>(ecs::id("transform")) {
                    transform.calculate_inner(data);
                }
            }
        }
    }

    pub fn calculate(&self) -> TransformData {
        let mut data = TransformData::new(Vector3::zero(), Vector3::zero(), Vector3::zero());

        self.calculate_inner(&mut data);

        data
    }
}
