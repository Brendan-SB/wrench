use crate::{Component, Registry, World};
use std::sync::{Arc, Mutex};

pub struct Entity {
    pub id: String,
    pub registry: Arc<Mutex<Arc<Registry>>>,
    pub components: Arc<Mutex<Vec<Arc<dyn Component>>>>,
}

impl Entity {
    pub fn new(
        id: String,
        registry: Arc<Mutex<Arc<Registry>>>,
        components: Arc<Mutex<Vec<Arc<dyn Component>>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            registry,
            components,
        })
    }

    pub fn get_of_type_id(&self, type_id: &String) -> Vec<Arc<dyn Component>> {
        self.components
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|c| *c.type_id() == *type_id)
            .collect::<Vec<Arc<dyn Component>>>()
    }

    pub fn get_of_id(&self, id: &String) -> Vec<Arc<dyn Component>> {
        self.components
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|c| *c.id() == *id)
            .collect::<Vec<Arc<dyn Component>>>()
    }

    pub fn remove(&self, component: Arc<dyn Component>) {
        self.remove_by_id(component.id());
    }

    pub fn remove_by_id(&self, id: &String) {
        let mut components = self.components.lock().unwrap();

        for (i, component) in components.clone().into_iter().enumerate() {
            if component.id() == id {
                components.remove(i);
            }
        }
    }
}
