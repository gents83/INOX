use std::path::{Path, PathBuf};

use nrg_camera::Camera;
use nrg_graphics::Mesh;
use nrg_math::{MatBase, Matrix4};
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::read_from_file;
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{Object, SceneData};

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
        CollapsingHeader::new(format!("Scene [{:?}]", id.to_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.collapsing(format!("Objects [{}]", self.objects.len()), |ui| {
                    for o in self.objects.iter() {
                        let id = o.id();
                        o.get_mut(|o| {
                            o.show(id, ui_registry, ui, collapsed);
                        });
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

    fn is_matching_extension(path: &Path) -> bool {
        const SCENE_EXTENSION: &str = "scene_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == SCENE_EXTENSION;
        }
        false
    }
}
impl DataTypeResource for Scene {
    type DataType = SceneData;

    fn is_initialized(&self) -> bool {
        !self.objects.is_empty()
    }

    fn invalidate(&mut self) {
        self.clear();
    }

    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: SceneId,
        scene_data: Self::DataType,
    ) -> Resource<Self> {
        let mut scene = Self::default();

        for object in scene_data.objects.iter() {
            let o = Object::load_from_file(shared_data, global_messenger, object.as_path());
            scene.add_object(o);
        }

        for camera in scene_data.cameras.iter() {
            let c = Camera::load_from_file(shared_data, global_messenger, camera.as_path());
            scene.add_camera(c);
        }

        SharedData::add_resource(shared_data, id, scene)
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

    pub fn update_hierarchy(&mut self, shared_data: &SharedDataRc) {
        for object in self.objects.iter() {
            object.get_mut(|o| {
                o.update_from_parent(
                    shared_data,
                    Matrix4::default_identity(),
                    |object, object_matrix| {
                        if let Some(mesh) = object.get_component::<Mesh>() {
                            mesh.get_mut(|m| {
                                m.set_matrix(object_matrix);
                            });
                        }
                    },
                );
            });
        }
    }
}
