pub mod component;
pub mod entity;

pub use component::Component;
pub use entity::{Entity, ENTITY_ID};

use std::sync::{Arc, RwLock};

pub fn id(id: &str) -> Arc<String> {
    Arc::new(id.to_string())
}

pub fn entity(value: Option<Arc<Entity>>) -> Arc<RwLock<Option<Arc<Entity>>>> {
    Arc::new(RwLock::new(value))
}
