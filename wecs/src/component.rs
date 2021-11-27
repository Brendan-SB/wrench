use crate::Entity;
use std::{any::Any, sync::{Arc, Mutex}};

pub trait Component: Send + Sync {
    fn entity(&self) -> Arc<Mutex<Option<Arc<Entity>>>>;

    fn set_entity(&self, entity: Option<Arc<Entity>>);

    fn id(&self) -> Arc<String>;

    fn tid(&self) -> Arc<String>;

    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    fn on_update(&self) {}

    fn on_drop(&mut self) {}
}
