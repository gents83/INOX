use std::collections::HashMap;

use inox_core::{implement_unique_system_uid, System};
use inox_math::{MatBase, Matrix4};
use inox_resources::{Resource, SharedDataRc};

use crate::{Object, ObjectId};

pub struct ObjectSystem {
    shared_data: SharedDataRc,
    map: HashMap<ObjectId, Matrix4>,
}

implement_unique_system_uid!(ObjectSystem);

impl System for ObjectSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("object_system::run");

        self.shared_data
            .for_each_resource(|r: &Resource<Object>, o: &Object| {
                let parent_transform = o
                    .parent()
                    .map(|parent| parent.get().transform())
                    .unwrap_or_else(Matrix4::default_identity);
                self.map.insert(*r.id(), parent_transform);
            });
        self.shared_data.for_each_resource_mut(|r, o: &mut Object| {
            let parent_transform = self.map.remove(r.id());
            o.update_transform(parent_transform);
        });

        true
    }
    fn uninit(&mut self) {}
}

impl ObjectSystem {
    pub fn new(shared_data: &SharedDataRc) -> Self {
        Self {
            shared_data: shared_data.clone(),
            map: HashMap::new(),
        }
    }
}
