use std::path::{Path, PathBuf};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use inox_serialize::{read_from_file, SerializeFile};
use inox_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{Camera, Object, SceneData};

pub type SceneId = ResourceId;

#[derive(Default, Clone)]
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

    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    fn extension() -> &'static str {
        SceneData::extension()
    }
}
impl DataTypeResource for Scene {
    type DataType = SceneData;
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &SceneId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &SceneId,
    ) {
    }

    fn is_initialized(&self) -> bool {
        !self.objects.is_empty()
    }
    fn invalidate(&mut self) -> &mut Self {
        self.clear();
        self
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        _id: SceneId,
        scene_data: Self::DataType,
    ) -> Self {
        let mut scene = Self::default();

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
