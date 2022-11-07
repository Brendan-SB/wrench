use crate::Entity;
use std::{
    any::Any,
    sync::{Arc, RwLock},
};

pub trait Component: Send + Sync + 'static {
    fn entity(&self) -> Arc<RwLock<Option<Arc<Entity>>>>;

    fn id(&self) -> Arc<String>;

    fn tid(&self) -> Arc<String>;

    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static>;

    fn on_init(&self) {}

    fn on_update(&self) {}

    fn on_remove(&self) {}
}
