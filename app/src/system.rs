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
        .as_secs();
        SystemId(secs)
    }
}

pub trait System: Send + Sync + 'static {
    type In;
    type Out;
    
    fn id(&self) -> SystemId;
    unsafe fn run_unsafe(&mut self, input: Self::In) -> Self::Out;
    fn run(&mut self, input: Self::In) -> Self::Out {
        unsafe { self.run_unsafe(input) }
    }
}

pub type SystemBoxed<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;