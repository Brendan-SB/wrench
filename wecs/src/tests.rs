use crate::{Component, Entity, World};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

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

    fn set_entity(&self, new_entity: Option<Arc<Entity>>) {
        let mut entity = self.entity.lock().unwrap();

        *entity = new_entity;
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
fn get_test() {
    let world = World::new();
    let entity = world.clone().create_default(Arc::new("entity".to_string()));

    entity.clone().add(TestComponent::new());

    let component = entity
        .get::<TestComponent>(Arc::new("test".to_string()), Arc::new("name".to_string()))[0]
        .clone();

    assert_eq!(component.test, 10);
}

#[test]
fn get_type_test() {
    let world = World::new();
    let entity = world.clone().create_default(Arc::new("entity".to_string()));

    entity.clone().add(TestComponent::new());

    let component = entity.get_type::<TestComponent>(Arc::new("test".to_string()))[0].clone();

    assert_eq!(component.test, 10);

    entity.remove(component);

    assert_eq!(
        entity
            .get_type::<TestComponent>(Arc::new("test".to_string()))
            .iter()
            .map(|c| c.test)
            .collect::<Vec<usize>>(),
        vec![]
    );

    world.remove(entity);

    assert_eq!(
        world
            .get()
            .lock()
            .unwrap()
            .iter()
            .map(|e| e.id.clone())
            .collect::<Vec<Arc<String>>>(),
        vec![]
    );
}
