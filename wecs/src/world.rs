use crate::{Component, Entity};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

pub struct World {
    pub entities: Arc<Mutex<Vec<Arc<Entity>>>>,
}

impl World {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entities: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn create(
        self: Arc<Self>,
        id: Arc<String>,
        components: Mutex<HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component + Send + Sync>>>>>>,
    ) -> Arc<Entity> {
        let entity = Entity::new(id, Mutex::new(Some(self.clone())), components);
        let mut entities = self.entities.lock().unwrap();

        entities.push(entity.clone());

        entity
    }

    pub fn create_default(self: Arc<Self>, id: Arc<String>) -> Arc<Entity> {
        let entity = Entity::new(
            id,
            Mutex::new(Some(self.clone())),
            Mutex::new(HashMap::new()),
        );

        entity
    }

    pub fn get(&self) -> Arc<Mutex<Vec<Arc<Entity>>>> {
        self.entities.clone()
    }

    pub fn remove<T>(&self, entity: T)
    where
        T: Deref<Target = Entity>,
    {
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
