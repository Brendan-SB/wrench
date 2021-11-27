use crate::Entity;
use std::sync::Arc;

pub trait Component {
    fn id(&self) -> Arc<String>;

    fn type_id(&self) -> Arc<String>;

    fn entity(&self) -> Arc<Entity>;

    fn on_drop(&mut self) {}
}
