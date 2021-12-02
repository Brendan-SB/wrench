use crate::Entity;
use std::sync::{Arc, Mutex};

pub struct World {
    entities: Arc<Mutex<Vec<Arc<Entity>>>>,
}

impl World {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entities: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn create(self: &Arc<Self>, id: Arc<String>) -> Arc<Entity> {
        let entity = Entity::new(id, Mutex::new(Some(self.clone())));
        let mut entities = self.entities.lock().unwrap();

        entities.push(entity.clone());

        entity
    }

    pub fn entities(&self) -> Arc<Mutex<Vec<Arc<Entity>>>> {
        self.entities.clone()
    }

    pub fn get(&self) -> Arc<Mutex<Vec<Arc<Entity>>>> {
        self.entities.clone()
    }

    pub fn remove<T>(&self, entity: Arc<Entity>) {
        self.remove_by_id(entity.id.clone());
    }

    pub fn remove_by_id(&self, id: Arc<String>) {
        let mut entities = self.entities.lock().unwrap();

        entities
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, e)| *e.id == *id)
            .for_each(|(i, e)| {
                entities.remove(i);
                *e.world.lock().unwrap() = None;
            });
    }
}
