use crate::Entity;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

pub trait Component: Send + Sync + 'static {
    fn entity(&self) -> Arc<Mutex<Option<Arc<Entity>>>>;

    fn set_entity(&self, entity: Option<Arc<Entity>>);

    fn id(&self) -> Arc<String>;

    fn tid(&self) -> Arc<String>;

    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static>;
    
    fn on_init(&self) {}

    fn on_update(&self) {}
}
