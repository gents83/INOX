use std::path::{Path, PathBuf};

use sabi_messenger::MessengerRw;
use sabi_nodes::LogicData;
use sabi_resources::{DataTypeResource, Handle, ResourceId, SerializableResource, SharedDataRc};
use sabi_serialize::{read_from_file, SerializeFile};
use sabi_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::Object;

pub type ScriptId = ResourceId;

#[derive(Clone)]
pub struct Script {
    filepath: PathBuf,
    parent: Handle<Object>,
    logic: LogicData,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            filepath: PathBuf::new(),
            parent: None,
            logic: LogicData::default(),
        }
    }
}

impl UIProperties for Script {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Script [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Num Nodes: ");
                    self.logic
                        .tree()
                        .get_nodes_count()
                        .show(id, ui_registry, ui, collapsed);
                });
                ui.horizontal(|ui| {
                    ui.label("Num Links: ");
                    self.logic
                        .tree()
                        .get_links_count()
                        .show(id, ui_registry, ui, collapsed);
                });
            });
    }
}

impl SerializableResource for Script {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    fn extension() -> &'static str {
        LogicData::extension()
    }
}
impl DataTypeResource for Script {
    type DataType = LogicData;

    fn is_initialized(&self) -> bool {
        self.logic.tree().get_nodes_count() > 0
    }
    fn invalidate(&mut self) {
        panic!("Script cannot be invalidated!");
    }
    fn deserialize_data(path: &std::path::Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        _id: ScriptId,
        data: Self::DataType,
    ) -> Self {
        Self {
            logic: data,
            ..Default::default()
        }
    }
}
