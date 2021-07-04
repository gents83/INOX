use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

pub type TransformId = ResourceId;

pub struct Transform {
    id: ResourceId,
    matrix: Matrix4,
}

impl ResourceTrait for Transform {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        PathBuf::default()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            matrix: Matrix4::default_identity(),
        }
    }
}

impl Transform {
    pub fn create(shared_data: &SharedDataRw) -> TransformId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(Transform::default())
    }

    pub fn get(shared_data: &SharedDataRw, transform_id: TransformId) -> Matrix4 {
        let transform = SharedData::get_resource::<Self>(shared_data, transform_id);
        let transform = &mut transform.get_mut();
        transform.matrix
    }

    pub fn set(shared_data: &SharedDataRw, transform_id: TransformId, matrix: Matrix4) {
        let transform = SharedData::get_resource::<Self>(shared_data, transform_id);
        let transform = &mut transform.get_mut();
        transform.matrix = matrix;
    }
}
