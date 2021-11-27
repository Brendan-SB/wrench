use crate::Entity;
use std::sync::{Arc, Mutex};

pub trait Component {
    fn entity(&self) -> Arc<Entity>;

    fn id(&self) -> &String;

    fn type_id(&self) -> &String;

    fn on_drop(&mut self) {}
}
