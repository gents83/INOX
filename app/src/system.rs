use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemId(pub u64);

impl Default for SystemId {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemId {
    pub fn new() -> Self {
        let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as _;
        SystemId(secs)
    }
}

pub trait System: Send + Sync + 'static {
    type In;
    type Out;
    
    fn id(&self) -> SystemId;
    
    fn init(&mut self);
    fn run(&mut self, input: Self::In) -> Self::Out;
    fn uninit(&mut self);
}

pub type SystemBoxed<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;