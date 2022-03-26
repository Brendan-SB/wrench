use crate::{self as ecs, Component};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct Entity {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<RwLock<Option<Arc<Entity>>>>,
    components: Arc<RwLock<HashMap<Arc<String>, Arc<RwLock<Vec<Arc<dyn Component>>>>>>>,
}

impl Entity {
    pub fn new(id: Arc<String>) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id("entity"),
            entity: ecs::entity(None),
            components: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn add<C>(self: &Arc<Self>, component: &Arc<C>)
    where
        C: Component,
    {
        {
            let entity = component.entity().read().unwrap().clone();

            if let Some(entity) = &entity {
                entity.remove(&component);
            }
        }

        *component.entity().write().unwrap() = Some(self.clone());

        let mut components = self.components.write().unwrap();

        match components.get(&component.tid()) {
            Some(components) => {
                let mut components = components.write().unwrap();

                components.push(component.clone());
            }

            None => {
                components.insert(
                    component.tid(),
                    Arc::new(RwLock::new(vec![component.clone()])),
                );
            }
        }
    }

    pub fn add_all<C>(self: &Arc<Self>, components: &[&Arc<C>])
    where
        C: Component,
    {
        components.into_iter().for_each(|c| {
            self.add(c);
        });
    }

    pub fn components(&self) -> HashMap<Arc<String>, Vec<Arc<dyn Component>>> {
        self.components
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.read().unwrap().clone()))
            .collect()
    }

    pub fn get<C>(&self, tid: Arc<String>, id: Arc<String>) -> Option<Arc<C>>
    where
        C: Component,
    {
        match self.components.read().unwrap().get(&tid) {
            Some(components) => match components
                .read()
                .unwrap()
                .iter()
                .filter(|c| *c.id() == *id)
                .next()
            {
                Some(component) => Some(component.clone().as_any().downcast::<C>().unwrap()),

                None => None,
            },

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

    pub fn get_type<C>(&self, tid: Arc<String>) -> Option<Arc<Vec<Arc<C>>>>
    where
        C: Component,
    {
        match self.components.read().unwrap().get(&tid) {
            Some(components) => Some(Arc::new(
                components
                    .read()
                    .unwrap()
                    .iter()
                    .map(|c| c.clone().as_any().downcast::<C>().unwrap())
                    .collect(),
            )),

            None => None,
        }
    }

    pub fn remove<C>(&self, component: &Arc<C>)
    where
        C: Component + ?Sized,
    {
        self.remove_by_id(component.tid(), component.id());
    }

    fn remove_from_target(target: &RwLock<Vec<Arc<dyn Component>>>, id: Arc<String>) -> bool {
        let mut target = target.write().unwrap();

        target
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, c)| *c.id() == *id)
            .for_each(|(i, v)| {
                v.remove();
                target.remove(i);
                *v.entity().write().unwrap() = None;
            });

        target.is_empty()
    }

    fn remove_by_id(&self, tid: Arc<String>, id: Arc<String>) {
        let mut components = self.components.write().unwrap();

        if let Some(target) = components.get(&tid) {
            let target_is_empty = Self::remove_from_target(target, id);

            if target_is_empty {
                components.remove(&tid);
            }
        }
    }

    pub fn remove_all<C>(self: &Arc<Self>, components: &[&Arc<C>])
    where
        C: Component,
    {
        components.into_iter().for_each(|c| {
            self.remove(c);
        });
    }
}

impl Component for Entity {
    fn entity(&self) -> Arc<RwLock<Option<Arc<Entity>>>> {
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
        let components = { self.components.read().unwrap().clone() };

        for (_, v) in components {
            let v = { v.read().unwrap().clone() };

            for component in v {
                component.init();
            }
        }
    }

    fn update(&self) {
        let components = { self.components.read().unwrap().clone() };

        for (_, v) in components {
            let v = { v.read().unwrap().clone() };

            for component in v {
                component.update();
            }
        }
    }

    fn remove(&self) {
        let components = { self.components.read().unwrap().clone() };

        for (_, v) in components {
            let v = { v.read().unwrap().clone() };

            for component in v {
                self.remove(&component);
            }
        }
    }
}
