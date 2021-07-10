use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_resources::{DynamicResource, Resource, ResourceId, ResourceTrait};
use nrg_serialize::generate_random_uid;

pub type TransformId = ResourceId;
pub type TransformRc = Resource;

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

impl DynamicResource for Transform {}

impl Transform {
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }

    pub fn set_matrix(&mut self, matrix: Matrix4) {
        self.matrix = matrix;
    }
}
