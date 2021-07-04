use std::path::{Path, PathBuf};

use nrg_graphics::{MaterialInstance, MeshInstance};
use nrg_math::{MatBase, Matrix4};
use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

use crate::{Object, ObjectId, Transform};

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

    pub fn update_hierarchy(shared_data: &SharedDataRw, scene_id: SceneId) {
        let scene = SharedData::get_resource::<Self>(shared_data, scene_id);
        let scene = &mut scene.get_mut();
        for object_id in scene.objects.iter() {
            Self::update_object_transform(shared_data, *object_id, Matrix4::default_identity());
        }
    }

    fn update_object_transform(
        shared_data: &SharedDataRw,
        object_id: ObjectId,
        parent_transform: Matrix4,
    ) {
        if let Some(transform_id) =
            Object::get_component_with_id::<Transform>(shared_data, object_id)
        {
            let object_matrix = Transform::get(shared_data, transform_id);
            let object_matrix = parent_transform * object_matrix;
            Transform::set(shared_data, transform_id, object_matrix);

            if let Some(material_id) =
                Object::get_component_with_id::<MaterialInstance>(shared_data, object_id)
            {
                for mesh_id in MaterialInstance::get_meshes(shared_data, material_id) {
                    let matrix = MeshInstance::get_transform(shared_data, mesh_id);
                    let matrix = object_matrix * matrix;
                    MeshInstance::set_transform(shared_data, mesh_id, matrix);
                }
            }

            let children = Object::get_children(shared_data, object_id);
            for child_id in children {
                Self::update_object_transform(shared_data, child_id, object_matrix);
            }
        }
    }
}
