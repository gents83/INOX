use std::path::{Path, PathBuf};

use nrg_graphics::Mesh;
use nrg_math::{MatBase, Matrix4};
use nrg_resources::{Resource, ResourceId, ResourceTrait, SharedDataRc};
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::Object;

pub type SceneId = ResourceId;

#[derive(Default, Clone)]
pub struct Scene {
    filepath: PathBuf,
    objects: Vec<Resource<Object>>,
}
impl ResourceTrait for Scene {}

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

impl Scene {
    pub fn set_filepath(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    pub fn clear(&mut self) {
        self.objects.clear();
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
