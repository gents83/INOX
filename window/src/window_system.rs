use nrg_core::*;
use nrg_platform::*;
use nrg_resources::SharedDataRw;

pub struct WindowSystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl WindowSystem {
    pub fn new(shared_data: &mut SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
        }
    }
}

impl System for WindowSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {}
    fn run(&mut self) -> (bool, Vec<Job>) {
        let data = self.shared_data.read().unwrap();
        let mut window_res = data.get_unique_resource_mut::<Window>();
        let can_continue = (*window_res).update();
        (can_continue, Vec::new())
    }
    fn uninit(&mut self) {}
}
