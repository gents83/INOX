use nrg_math::{Mat4Ops, MatBase, Matrix4};
use nrg_resources::{ResourceData, ResourceId, ResourceRef};
use nrg_serialize::generate_random_uid;

pub type TransformId = ResourceId;
pub type TransformRc = ResourceRef<Transform>;

pub struct Transform {
    id: ResourceId,
    matrix: Matrix4,
}

impl ResourceData for Transform {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn info(&self) -> String {
        let (translation, rotation, scale) = self.matrix.get_translation_rotation_scale();
        format!(
            "Matrix {:?}
            Position {:?}
            Rotation {:?}
            Scale {:?}",
            self.id().to_simple().to_string(),
            translation,
            rotation,
            scale
        )
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
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }

    pub fn set_matrix(&mut self, matrix: Matrix4) {
        self.matrix = matrix;
    }
}
