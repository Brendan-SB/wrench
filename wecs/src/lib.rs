pub mod component;
pub mod entity;
pub mod world;

pub use component::Component;
pub use entity::Entity;
pub use world::World;

use std::sync::Arc;

pub fn id(id: &str) -> Arc<String> {
    Arc::new(id.to_string())
}
