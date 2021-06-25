use std::path::{Path, PathBuf};

use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

use crate::ObjectId;

pub type SceneId = ResourceId;

pub struct Scene {
    id: ResourceId,
    filepath: PathBuf,
    objects: Vec<ObjectId>,
    shared_data: SharedDataRw,
}

impl ResourceTrait for Scene {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        self.filepath.clone()
    }
}

impl Scene {
    pub fn create(shared_data: &SharedDataRw) -> SceneId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(Self {
            id: generate_random_uid(),
            filepath: PathBuf::new(),
            objects: Vec::new(),
            shared_data: shared_data.clone(),
        })
    }

    pub fn set_filepath(shared_data: &SharedDataRw, scene_id: SceneId, path: &Path) {
        let scene = SharedData::get_resource::<Self>(shared_data, scene_id);
        let scene = &mut scene.get_mut();
        scene.filepath = path.to_path_buf();
    }

    pub fn add_object(shared_data: &SharedDataRw, scene_id: SceneId, object_id: ObjectId) {
        let scene = SharedData::get_resource::<Self>(shared_data, scene_id);
        let scene = &mut scene.get_mut();
        scene.objects.push(object_id);
    }
}
