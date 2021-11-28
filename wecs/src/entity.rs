use crate::{Component, World};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Entity {
    pub id: Arc<String>,
    pub world: Mutex<Option<Arc<World>>>,
    components: Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component + Send + Sync>>>>>>>,
}

impl Entity {
    pub fn new(id: Arc<String>, world: Mutex<Option<Arc<World>>>) -> Arc<Self> {
        Arc::new(Self {
            id,
            world,
            components: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn add(self: Arc<Self>, component: Arc<dyn Component + Send + Sync>) {
        let mut components = self.components.lock().unwrap();

        component.set_entity(Some(self.clone()));

        match components.get(&component.tid()) {
            Some(components) => {
                let mut components = components.lock().unwrap();

                components.push(component);
            }

            None => {
                components.insert(component.tid(), Arc::new(Mutex::new(vec![component])));
            }
        }
    }

    pub fn components(&self) -> Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component + Send + Sync>>>>>>> {
        self.components.clone()
    }

    pub fn get_type<T>(&self, type_id: Arc<String>) -> Arc<Vec<Arc<T>>>
    where
        T: Component + Send + Sync + 'static,
    {
        match self.components.lock().unwrap().get(&type_id) {
            Some(components) => Arc::new(
                components
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .map(|c| c.as_any().downcast::<T>().unwrap())
                    .collect::<Vec<Arc<T>>>(),
            ),

            None => Arc::new(Vec::new()),
        }
    }

    pub fn get<T>(&self, type_id: Arc<String>, id: Arc<String>) -> Arc<Vec<Arc<T>>>
    where
        T: Component + Send + Sync + 'static,
    {
        match self.components.lock().unwrap().get(&type_id) {
            Some(components) => Arc::new(
                components
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .filter(|c| *c.id() == *id)
                    .map(|c| c.as_any().downcast::<T>().unwrap())
                    .collect::<Vec<Arc<T>>>(),
            ),

            None => Arc::new(Vec::new()),
        }
    }

    pub fn remove(&self, component: Arc<dyn Component>) {
        self.remove_by_id(component.tid().clone(), component.id().clone());
    }

    pub fn remove_by_id(&self, type_id: Arc<String>, id: Arc<String>) {
        let components = self.components.lock().unwrap();

        if let Some(components) = components.get(&type_id) {
            let mut components = components.lock().unwrap();

            components
                .clone()
                .into_iter()
                .enumerate()
                .filter(|(_, c)| *c.id() == *id)
                .for_each(|(i, v)| {
                    components.remove(i);
                    v.set_entity(None);
                })
        }
    }
}