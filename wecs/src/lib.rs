pub mod component;
pub mod entity;

pub use component::Component;
pub use entity::Entity;

use std::sync::{Mutex, Arc};

pub fn id(id: &str) -> Arc<String> {
    Arc::new(id.to_string())
}

pub fn entity(value: Option<Arc<Entity>>) -> Arc<Mutex<Option<Arc<Entity>>>> {
    Arc::new(Mutex::new(value))
}
