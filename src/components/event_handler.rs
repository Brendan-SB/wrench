use crate::ecs::{self, reexports::*};
use winit::event::Event;

pub trait Handler: Send + Sync {
    fn set_event_handler(&self, _: Option<Arc<EventHandler>>) {}

    fn handle<'a>(&self, event: &Event<'a, ()>);
}

#[derive(Component)]
pub struct EventHandler {
    pub entity: Arc<Mutex<Option<Arc<Entity>>>>,
    pub id: Arc<String>,
    pub tid: Arc<String>,
    pub handler: Mutex<Arc<dyn Handler>>,
}

impl EventHandler {
    pub fn new<'a>(id: Arc<String>, handler: Arc<dyn Handler>) -> Arc<Self> {
        let event_handler = Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: ecs::id("event handler"),
            handler: Mutex::new(handler.clone()),
        });

        handler.set_event_handler(Some(event_handler.clone()));

        event_handler
    }

    pub fn handle<'a>(&self, event: &Event<'a, ()>) {
        self.handler.lock().unwrap().handle(event);
    }
}
