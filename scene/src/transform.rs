use nrg_math::{Deg, Mat4Ops, MatBase, Matrix4, Rad};
use nrg_resources::{ResourceData, ResourceId, ResourceRef};
use nrg_serialize::generate_random_uid;
use nrg_ui::{UIProperties, Ui};

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

impl UIProperties for Transform {
    fn show(&mut self, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            let (mut translation, mut rotation, mut scale) =
                self.matrix.get_translation_rotation_scale();
            ui.horizontal(|ui| {
                ui.label("Translation: ");
                translation.show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Rotation: ");
                rotation.show(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Scale: ");
                scale.show(ui);
            });
            self.matrix = Matrix4::from_translation(translation)
                * Matrix4::from_angle_z(Rad::from(Deg(rotation.x)))
                * Matrix4::from_angle_y(Rad::from(Deg(rotation.y)))
                * Matrix4::from_angle_z(Rad::from(Deg(rotation.z)))
                * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        });
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
