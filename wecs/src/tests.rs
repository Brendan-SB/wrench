use crate::{Component, Entity, World};
use std::{any::Any, sync::{Arc, Mutex}};

struct TestComponent {
    test: usize,
    entity: Arc<Mutex<Option<Arc<Entity>>>>,
    tid: Arc<String>,
    id: Arc<String>,
}

impl TestComponent {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            test: 10,
            entity: Arc::new(Mutex::new(None)),
            tid: Arc::new("test".to_string()),
            id: Arc::new("name".to_string()),
        })
    }
}

impl Component for TestComponent {
    fn entity(&self) -> Arc<Mutex<Option<Arc<Entity>>>> {
        self.entity.clone()
    }

    fn set_entity(&self, new_entity: Arc<Entity>) {
        let mut entity = self.entity.lock().unwrap();

        *entity = Some(new_entity);
    }

    fn id(&self) -> Arc<String> {
        self.id.clone()
    }

    fn tid(&self) -> Arc<String> {
        self.tid.clone()
    }

    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self.clone()
    }
}

#[test]
fn test() {
    let world = World::new();
    let entity = world.clone().create_default(Arc::new("entity".to_string()));

    entity.clone().add(TestComponent::new());

    let component = entity
        .get::<TestComponent>(Arc::new("test".to_string()), Arc::new("name".to_string()))[0].clone();

    assert_eq!(component.test, 10);
}
