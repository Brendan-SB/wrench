use crate::{Component, World};
use std::{collections: HashMap, sync::{Arc, Mutex}};

pub struct Entity {
    pub id: String,
    pub world: Mutex<Arc<World>>,
    pub components: Mutex<Vec<HashMap<String, Arc<dyn Component>>>>,
}

impl Entity {
    pub fn new(
        id: String,
        world: Mutex<Arc<World>>,
        components: Mutex<Vec<Arc<dyn Component>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            world,
            components,
        })
    }

    pub fn get(&self, id: &String) -> Vec<Arc<dyn Component>> {
        self.components
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|c| *c.id() == *id)
            .collect::<Vec<Arc<dyn Component>>>()
    }

    pub fn get_type(&self, type_id: &String) -> Vec<Arc<dyn Component>> {
        self.components
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|c| *c.type_id() == *type_id)
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

impl Drop for Entity {
    fn drop(&mut self) {
        let world = self.world.lock().unwrap();

        world.remove_by_id(&self.id);
    }
}
