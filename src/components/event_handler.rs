use crate::ecs::{self, reexports::*};
use winit::event::Event;

pub trait Handler: Send + Sync {
    fn set_event_handler(&self, _: Option<Arc<EventHandler>>) {}

    fn handle<'a>(&self, event: &Event<'a, ()>);
}

#[derive(Component)]
pub struct EventHandler {
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub entity: Arc<RwLock<Option<Arc<Entity>>>>,
    pub handler: RwLock<Arc<dyn Handler>>,
}

impl EventHandler {
    pub fn new<'a>(id: Arc<String>, handler: Arc<dyn Handler>) -> Arc<Self> {
        let event_handler = Arc::new(Self {
            id,
            tid: ecs::id("event handler"),
            entity: ecs::entity(None),
            handler: RwLock::new(handler.clone()),
        });

        handler.set_event_handler(Some(event_handler.clone()));

        event_handler
    }

    pub fn handle<'a>(&self, event: &Event<'a, ()>) {
        self.handler.read().unwrap().handle(event);
    }
}
