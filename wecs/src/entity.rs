use crate::{self as ecs, Component};
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
            tid: ecs::id("entity"),
            entity: ecs::entity(None),
            components: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn add<C>(self: &Arc<Self>, component: &Arc<C>)
    where
        C: Component,
    {
        let e = component.entity();
        let entity = {
            e.lock().unwrap().clone()
        };

        if let Some(entity) = entity {
            entity.remove(&component);
        }

        *e.lock().unwrap() = Some(self.clone());

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

    pub fn add_all<C>(self: &Arc<Self>, components: &[&Arc<C>])
    where
        C: Component, {
            components.into_iter().for_each(|c| {
                self.add(c);
            });
        }

    pub fn components(&self) -> HashMap<Arc<String>, Vec<Arc<dyn Component>>> {
        self.components
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.lock().unwrap().clone()))
            .collect()
    }

    pub fn get_type<C>(&self, tid: Arc<String>) -> Option<Arc<Vec<Arc<C>>>>
    where
        C: Component,
    {
        match self.components.lock().unwrap().get(&tid) {
            Some(components) => Some(Arc::new(
                components
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|c| c.clone().as_any().downcast::<C>().unwrap())
                    .collect(),
            )),

            None => None,
        }
    }

    pub fn get<C>(&self, tid: Arc<String>, id: Arc<String>) -> Option<Arc<Vec<Arc<C>>>>
    where
        C: Component,
    {
        match self.components.lock().unwrap().get(&tid) {
            Some(components) => Some(Arc::new(
                components
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|c| *c.id() == *id)
                    .map(|c| c.clone().as_any().downcast::<C>().unwrap())
                    .collect(),
            )),

            None => None,
        }
    }

    pub fn get_first<C>(&self, tid: Arc<String>) -> Option<Arc<C>>
    where
        C: Component,
    {
        match self.get_type::<C>(tid) {
            Some(components) => match components.first() {
                Some(component) => Some(component.clone()),

                None => None,
            },

            None => None,
        }
    }

    pub fn remove<C>(&self, component: &Arc<C>)
    where C: Component + ?Sized {
        self.remove_by_id(component.tid(), component.id());
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

    fn remove_by_id(&self, tid: Arc<String>, id: Arc<String>) {
        let mut components = self.components.lock().unwrap();

        if let Some(target) = components.get(&tid) {
            let target_is_empty = Self::remove_from_target(target, id);

            if target_is_empty {
                components.remove(&tid);
            }
        }
    }

    pub fn remove_all<C>(self: &Arc<Self>, components: &[&Arc<C>])
    where
        C: Component, {
            components.into_iter().for_each(|c| {
                self.remove(c);
            });
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
        let components = {
            self.components.lock().unwrap().clone()
        };

        for (_, v) in components {
            let v = {
                v.lock().unwrap().clone()
            };

            for component in v {
                component.init();
            }
        }
    }

    fn update(&self) {
        let components = {
            self.components.lock().unwrap().clone()
        };

        for (_, v) in components {
            let v = {
                v.lock().unwrap().clone()
            };

            for component in v {
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
                self.remove(&component);
            }
        }
    }
}
