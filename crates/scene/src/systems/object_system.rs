use std::collections::HashMap;

use inox_core::{implement_unique_system_uid, ContextRc, System};
use inox_math::{MatBase, Matrix4};
use inox_messenger::Listener;
use inox_resources::{ResourceEvent, SharedDataRc};

use crate::{Object, ObjectId};

pub struct ObjectSystem {
    shared_data: SharedDataRc,
    listener: Listener,
    map: HashMap<ObjectId, Matrix4>,
}

implement_unique_system_uid!(ObjectSystem);

impl System for ObjectSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.listener.register::<ResourceEvent<Object>>();
    }

    fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("object_system::run");

        self.update_events();

        self.map.retain(|id, m| {
            if let Some(o) = self.shared_data.get_resource::<Object>(id) {
                o.get_mut().update_transform(Some(*m));
            }
            false
        });

        true
    }
    fn uninit(&mut self) {
        self.listener.unregister::<ResourceEvent<Object>>();
    }
}

impl ObjectSystem {
    pub fn new(context: &ContextRc) -> Self {
        Self {
            shared_data: context.shared_data().clone(),
            listener: Listener::new(context.message_hub()),
            map: HashMap::new(),
        }
    }
    fn update_events(&mut self) {
        inox_profiler::scoped_profile!("object_system::update_events");
        self.listener.process_messages(|e: &ResourceEvent<Object>| {
            if let ResourceEvent::Changed(id) = e {
                if let Some(object) = self.shared_data.get_resource::<Object>(id) {
                    let parent_transform = object
                        .get()
                        .parent()
                        .map(|parent| parent.get().transform())
                        .unwrap_or_else(Matrix4::default_identity);
                    self.map.insert(*id, parent_transform);
                }
            }
        });
    }
}
