use crate::{self as ecs, Component};
use std::{
    any::Any,
    sync::{Arc, RwLock},
};

pub const ENTITY_ID: &str = "entity";

pub struct Entity {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<RwLock<Option<Arc<Entity>>>>,
    components: Arc<RwLock<Vec<Arc<dyn Component>>>>,
}

impl Entity {
    pub fn new(id: Arc<String>) -> Arc<Self> {
        Arc::new(Self {
            id,
            tid: ecs::id(ENTITY_ID),
            entity: ecs::entity(None),
            components: Arc::new(RwLock::new(Vec::new())),
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

        components.push(component.clone());
    }

    pub fn add_all<C>(self: &Arc<Self>, components: &[&Arc<C>])
    where
        C: Component,
    {
        components.into_iter().for_each(|c| {
            self.add(c);
        });
    }

    pub fn components(&self) -> Arc<RwLock<Vec<Arc<dyn Component>>>> {
        self.components.clone()
    }

    pub fn get<C>(&self, tid: Arc<String>, id: Arc<String>) -> Option<Arc<C>>
    where
        C: Component,
    {
        match self
            .components
            .read()
            .unwrap()
            .iter()
            .filter_map(|c| {
                if *c.id() == *id && c.tid() == tid {
                    return Some(c);
                }

                None
            })
            .next()
        {
            Some(component) => Some(component.clone().as_any().downcast::<C>().unwrap()),

            None => None,
        }
    }

    pub fn get_first<C>(&self, tid: Arc<String>) -> Option<Arc<C>>
    where
        C: Component,
    {
        match self.get_type::<C>(tid).first() {
            Some(component) => Some(component.clone()),

            None => None,
        }
    }

    pub fn get_type<C>(&self, tid: Arc<String>) -> Vec<Arc<C>>
    where
        C: Component,
    {
        self.components
            .read()
            .unwrap()
            .iter()
            .filter_map(|c| {
                if *c.tid() == *tid {
                    return Some(c.clone().as_any().downcast::<C>().unwrap());
                }

                None
            })
            .collect()
    }

    pub fn remove<C>(&self, component: &Arc<C>)
    where
        C: Component + ?Sized,
    {
        self.components.write().unwrap().retain(|c| {
            if *c.id() == *component.id() && *c.tid() == *component.tid() {
                c.on_remove();
                *c.entity().write().unwrap() = None;

                return false;
            }

            true
        });
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

    fn on_init(&self) {
        let components = { self.components.read().unwrap().clone() };

        for component in components {
            component.on_init();
        }
    }

    fn on_update(&self) {
        let components = { self.components.read().unwrap().clone() };

        for component in components {
            component.on_update();
        }
    }
}
