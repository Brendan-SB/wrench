use crate::ecs::reexports::*;
use winit::event::Event;

pub trait Handler: Send + Sync + 'static {
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
        Arc::new(Self {
            entity: Arc::new(Mutex::new(None)),
            id,
            tid: Arc::new("event handler".to_string()),
            handler: Mutex::new(handler),
        })
    }

    pub fn handle<'a>(&self, event: &Event<'a, ()>) {
        self.handler.lock().unwrap().handle(event);
    }
}
