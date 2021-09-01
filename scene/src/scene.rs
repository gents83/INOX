use std::path::{Path, PathBuf};

use nrg_graphics::Mesh;
use nrg_math::{MatBase, Matrix4};
use nrg_resources::{ResourceData, ResourceId, ResourceRef, SharedDataRw};
use nrg_serialize::{generate_random_uid, generate_uid_from_string};
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::ObjectRc;

pub type SceneId = ResourceId;
pub type SceneRc = ResourceRef<Scene>;

pub struct Scene {
    id: ResourceId,
    filepath: PathBuf,
    objects: Vec<ObjectRc>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            filepath: PathBuf::new(),
            objects: Vec::new(),
        }
    }
}

impl UIProperties for Scene {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!("Scene [{:?}]", self.id().to_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.collapsing(format!("Objects [{}]", self.objects.len()), |ui| {
                    for c in self.objects.iter() {
                        c.resource().get_mut().show(ui_registry, ui, collapsed);
                    }
                });
            });
    }
}

impl ResourceData for Scene {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl Scene {
    pub fn set_filepath(&mut self, path: &Path) {
        self.id = generate_uid_from_string(path.to_str().unwrap());
        self.filepath = path.to_path_buf();
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add_object(&mut self, object: ObjectRc) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> Vec<ObjectRc> {
        self.objects.clone()
    }

    pub fn update_hierarchy(&mut self, shared_data: &SharedDataRw) {
        for object in self.objects.iter() {
            object.resource().get_mut().update_from_parent(
                shared_data,
                Matrix4::default_identity(),
                |object, object_matrix| {
                    if let Some(mesh) = object.get_component::<Mesh>() {
                        mesh.resource().get_mut().set_matrix(object_matrix);
                    }
                },
            );
        }
    }
}
