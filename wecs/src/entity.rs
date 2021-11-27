use crate::{Component, World};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Entity {
    pub id: Arc<String>,
    pub world: Mutex<Option<Arc<World>>>,
    pub components: Mutex<HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component + Send + Sync>>>>>>,
}

impl Entity {
    pub fn new(
        id: Arc<String>,
        world: Mutex<Option<Arc<World>>>,
        components: Mutex<HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component + Send + Sync>>>>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            world,
            components,
        })
    }

    pub fn add(self: Arc<Self>, component: Arc<dyn Component + Send + Sync>) {
        if let Some(entity) = component.entity().clone().lock().unwrap().as_ref() {
            entity.remove_by_id(component.tid(), component.id());
        }

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

impl Drop for Entity {
    fn drop(&mut self) {
        if let Some(world) = self.world.lock().unwrap().as_ref() {
            world.remove_by_id(self.id.clone());
        }
    }
}
