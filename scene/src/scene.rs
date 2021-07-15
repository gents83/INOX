use std::path::{Path, PathBuf};

use nrg_math::{MatBase, Matrix4};
use nrg_resources::{ResourceData, ResourceId, ResourceRef, SharedDataRw};
use nrg_serialize::generate_random_uid;

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

impl ResourceData for Scene {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl Scene {
    pub fn set_filepath(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add_object(&mut self, object: ObjectRc) {
        self.objects.push(object);
    }

    pub fn update_hierarchy(&mut self, shared_data: &SharedDataRw) {
        for object in self.objects.iter() {
            object
                .resource()
                .get_mut()
                .update_from_parent(shared_data, Matrix4::default_identity());
        }
    }
}
