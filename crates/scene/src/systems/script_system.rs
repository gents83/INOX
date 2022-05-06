use inox_core::{implement_unique_system_uid, ContextRc, System};

use crate::Script;

pub struct ScriptSystem {
    context: ContextRc,
}

implement_unique_system_uid!(ScriptSystem);

impl System for ScriptSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("object_system::run");

        let timer = self.context.global_timer();
        self.context
            .shared_data()
            .for_each_resource_mut(|_, s: &mut Script| {
                s.update(&timer);
            });
        true
    }
    fn uninit(&mut self) {}
}

impl ScriptSystem {
    pub fn new(context: &ContextRc) -> Self {
        Self {
            context: context.clone(),
        }
    }
}
