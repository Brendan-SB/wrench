use crate::Component;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    any::Any,
};

pub struct Entity {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    components: Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component>>>>>>>,
}

impl Entity {
    pub fn new(id: Arc<String>) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: Arc::new("entity".to_string()),
            entity: Arc::new(Mutex::new(None)),
            components: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn setup_component(component: Arc<dyn Component>, entity: &Option<Arc<Entity>>) {
        let entity = match entity {
            Some(entity) => entity.clone(),
            None => return,
        };

        entity.remove(&component);
    }

    pub fn add<C>(self: &Arc<Self>, component: &Arc<C>)
    where
        C: Component,
    {
        let entity = component.entity();
        let mut entity = entity.lock().unwrap();

        Self::setup_component(component.clone(), &*entity);

        *entity = Some(self.clone());

        let mut components = self.components.lock().unwrap();

        match components.get(&component.tid()) {
            Some(components) => {
                let mut components = components.lock().unwrap();

                components.push(component.clone());
            }

            None => {
                components.insert(
                    component.tid(),
                    Arc::new(Mutex::new(vec![component.clone()])),
                );
            }
        }
    }

    pub fn components(&self) -> HashMap<Arc<String>, Vec<Arc<dyn Component>>> {
        self.components
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.lock().unwrap().clone()))
            .collect()
    }

    pub fn get_type<T>(&self, tid: Arc<String>) -> Option<Arc<Vec<Arc<T>>>>
    where
        T: Component,
    {
        match self.components.lock().unwrap().get(&tid) {
            Some(components) => Some(Arc::new(
                components
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|c| c.clone().as_any().downcast::<T>().unwrap())
                    .collect::<Vec<Arc<T>>>(),
            )),

            None => None,
        }
    }

    pub fn get<T>(&self, tid: Arc<String>, id: Arc<String>) -> Option<Arc<Vec<Arc<T>>>>
    where
        T: Component,
    {
        match self.components.lock().unwrap().get(&tid) {
            Some(components) => Some(Arc::new(
                components
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|c| *c.id() == *id)
                    .map(|c| c.clone().as_any().downcast::<T>().unwrap())
                    .collect::<Vec<Arc<T>>>(),
            )),

            None => None,
        }
    }

    pub fn get_first<T>(&self, tid: Arc<String>) -> Option<Arc<T>>
    where
        T: Component,
    {
        match self.get_type::<T>(tid) {
            Some(components) => match components.first() {
                Some(component) => Some(component.clone()),

                None => None,
            },

            None => None,
        }
    }

    pub fn remove<T>(&self, component: &Arc<T>)
    where T: Component + ?Sized {
        self.remove_by_id(&mut *self.components.lock().unwrap(), component.tid(), component.id());
    }

    fn remove_from_target(target: &Mutex<Vec<Arc<dyn Component>>>, id: Arc<String>) -> bool {
        let mut target = target.lock().unwrap();

        target
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, c)| *c.id() == *id)
            .for_each(|(i, v)| {
                v.remove();
                target.remove(i);
                *v.entity().lock().unwrap() = None;
            });

        target.is_empty()
    }

    fn remove_by_id(&self, components: &mut HashMap<Arc<String>, Arc<Mutex<Vec<Arc<dyn Component>>>>>, tid: Arc<String>, id: Arc<String>) {
        if let Some(target) = components.get(&tid) {
            let target_is_empty = Self::remove_from_target(target, id);

            if target_is_empty {
                components.remove(&tid);
            }
        }
    }
}

impl Component for Entity {
    fn entity(&self) -> Arc<Mutex<Option<Arc<Entity>>>> {
        self.entity.clone()
    }

    fn id(&self) -> Arc<String> {
        self.id.clone()
    }

    fn tid(&self) -> Arc<String> {
        self.tid.clone()
    }

    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> {
        self.clone() as Arc<dyn Any + Send + Sync + 'static>
    }

    fn init(&self) {
        for (_, v) in &*self.components.lock().unwrap() {
            for component in &*v.lock().unwrap() {
                component.init();
            }
        }
    }

    fn update(&self) {
        for (_, v) in &*self.components.lock().unwrap() {
            for component in &*v.lock().unwrap() {
                component.update();
            }
        }
    }

    fn remove(&self) {
        let components = {
            self.components.lock().unwrap().clone()
        };

        for (_, v) in components {
            let v = {
                v.lock().unwrap().clone()
            };

            for component in v {
                self.remove_by_id(&mut self.components.lock().unwrap(), component.tid(), component.id());
            }
        }
    }
}
