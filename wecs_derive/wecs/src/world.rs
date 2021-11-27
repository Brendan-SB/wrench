use crate::{Entity, Registry};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

pub struct World {
    pub registries: Mutex<Vec<Arc<Registry>>>,
}

impl World {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            registries: Mutex::new(Vec::new()),
        })
    }

    pub fn create(self: Arc<Self>, id: String, entities: Vec<Arc<Entity>>) -> Arc<Registry> {
        let registry = Registry::new(
            id,
            Arc::new(Mutex::new(self.clone())),
            Arc::new(Mutex::new(entities)),
        );
        let mut registries = self.registries.lock().unwrap();

        registries.push(registry.clone());

        registry
    }

    pub fn remove<T>(&self, registry: T)
    where
        T: Deref<Target = Registry>,
    {
        self.remove_by_id(&registry.id);
    }

    pub fn remove_by_id(&self, id: &String) {
        let mut registries = self.registries.lock().unwrap();

        for (i, registry) in registries.clone().into_iter().enumerate() {
            if registry.id == *id {
                registries.remove(i);
            }
        }
    }
}
