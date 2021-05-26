use nrg_core::*;
use nrg_platform::*;
use nrg_resources::{SharedData, SharedDataRw};

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
    fn run(&mut self) -> bool {
        let window_res = SharedData::get_unique_resource::<Window>(&self.shared_data);
        let can_continue = window_res.get_mut().update();
        can_continue
    }
    fn uninit(&mut self) {}
}
