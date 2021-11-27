use crate::{Component, Entity, World};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

pub struct Registry {
    pub id: String,
    pub world: Arc<Mutex<Arc<World>>>,
    pub entities: Arc<Mutex<Vec<Arc<Entity>>>>,
}

impl Registry {
    pub fn new(
        id: String,
        world: Arc<Mutex<Arc<World>>>,
        entities: Arc<Mutex<Vec<Arc<Entity>>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            world,
            entities,
        })
    }

    pub fn create(self: Arc<Self>, id: String, components: Vec<Arc<dyn Component>>) -> Arc<Entity> {
        let entity = Entity::new(
            id,
            Arc::new(Mutex::new(self.clone())),
            Arc::new(Mutex::new(components)),
        );

        let mut entities = self.entities.lock().unwrap();

        entities.push(entity.clone());

        entity
    }

    pub fn remove<T>(&self, entity: T)
    where
        T: Deref<Target = Entity>,
    {
        self.remove_by_id(&entity.id);
    }

    pub fn remove_by_id(&self, id: &String) {
        let mut entities = self.entities.lock().unwrap();

        for (i, entity) in entities.clone().into_iter().enumerate() {
            if entity.id == *id {
                entities.remove(i);
            }
        }
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        self.world.lock().unwrap().remove_by_id(&self.id);
    }
}
