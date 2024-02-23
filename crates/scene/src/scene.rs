use std::path::{Path, PathBuf};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedDataRc,
};
use inox_serialize::{
    inox_serializable::SerializableRegistryRc, read_from_file, SerializationType, SerializeFile,
};
use inox_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{Camera, Object, SceneData};

pub type SceneId = ResourceId;

#[derive(Clone)]
pub struct Scene {
    filepath: PathBuf,
    objects: Vec<Resource<Object>>,
    cameras: Vec<Resource<Camera>>,
}

impl UIProperties for Scene {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Scene [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.collapsing(format!("Objects [{}]", self.objects.len()), |ui| {
                    for o in self.objects.iter() {
                        let id = o.id();
                        o.get_mut().show(id, ui_registry, ui, collapsed);
                    }
                });
            });
    }
}

impl SerializableResource for Scene {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.filepath = path.to_path_buf();
        self
    }

    fn extension() -> &'static str {
        SceneData::extension()
    }

    fn deserialize_data(
        path: &std::path::Path,
        registry: SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, SerializationType::Binary, f);
    }
}

impl ResourceTrait for Scene {
    fn is_initialized(&self) -> bool {
        !self.objects.is_empty()
    }
    fn invalidate(&mut self) -> &mut Self {
        self
    }
}

impl DataTypeResource for Scene {
    type DataType = SceneData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            filepath: PathBuf::new(),
            objects: Vec::new(),
            cameras: Vec::new(),
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: SceneId,
        scene_data: &Self::DataType,
    ) -> Self {
        let mut scene = Self::new(id, shared_data, message_hub);

        for object in scene_data.objects.iter() {
            let o = Object::request_load(shared_data, message_hub, object.as_path(), None);
            scene.add_object(o);
        }

        for camera in scene_data.cameras.iter() {
            let c = Camera::request_load(shared_data, message_hub, camera.as_path(), None);
            scene.add_camera(c);
        }

        scene
    }
}

impl Scene {
    pub fn set_filepath(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.cameras.clear();
    }

    pub fn add_camera(&mut self, camera: Resource<Camera>) {
        self.cameras.push(camera);
    }

    pub fn cameras(&self) -> &Vec<Resource<Camera>> {
        &self.cameras
    }

    pub fn add_object(&mut self, object: Resource<Object>) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &Vec<Resource<Object>> {
        &self.objects
    }
}
