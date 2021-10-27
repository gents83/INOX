use std::path::{Path, PathBuf};

use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::read_from_file;
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{LightData, LightType, Object};

pub type LightId = ResourceId;

#[derive(Clone)]
pub struct Light {
    filepath: PathBuf,
    parent: Handle<Object>,
    data: LightData,
    is_active: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            filepath: PathBuf::new(),
            parent: None,
            data: LightData::default(),
            is_active: true,
        }
    }
}

impl UIProperties for Light {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Light [{:?}]", id.to_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Type: ");
                    match self.data.light_type {
                        LightType::Directional => ui.label("Directional"),
                        LightType::Point => ui.label("Point"),
                        LightType::Spot(_, _) => ui.label("Spot"),
                    };
                });
                ui.horizontal(|ui| {
                    ui.label("Color: ");
                    self.data.color.show(id, ui_registry, ui, collapsed);
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity: ");
                    self.data.intensity.show(id, ui_registry, ui, collapsed);
                });
                ui.horizontal(|ui| {
                    ui.label("Range: ");
                    self.data.range.show(id, ui_registry, ui, collapsed);
                });
            });
    }
}

impl SerializableResource for Light {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    fn is_matching_extension(path: &Path) -> bool {
        const LIGHT_EXTENSION: &str = "light_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == LIGHT_EXTENSION;
        }
        false
    }
}
impl DataTypeResource for Light {
    type DataType = LightData;
    fn is_initialized(&self) -> bool {
        true
    }

    fn invalidate(&mut self) {
        panic!("Light cannot be invalidated!");
    }

    fn deserialize_data(path: &std::path::Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        id: LightId,
        data: Self::DataType,
    ) -> Resource<Self> {
        let light = Self {
            data,
            ..Default::default()
        };
        SharedData::add_resource(shared_data, id, light)
    }
}

impl Light {
    #[inline]
    pub fn parent(&self) -> &Handle<Object> {
        &self.parent
    }

    #[inline]
    pub fn set_parent(&mut self, parent: &Resource<Object>) -> &mut Self {
        self.parent = Some(parent.clone());
        self
    }

    #[inline]
    pub fn data(&self) -> &LightData {
        &self.data
    }

    #[inline]
    pub fn set_active(&mut self, is_active: bool) -> &mut Self {
        self.is_active = is_active;
        self
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
