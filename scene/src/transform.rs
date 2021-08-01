use nrg_math::{Mat4Ops, MatBase, Matrix4, VecBase, Vector3};
use nrg_resources::{ResourceData, ResourceId, ResourceRef};
use nrg_serialize::generate_random_uid;
use nrg_ui::{UIProperties, UIPropertiesRegistry, Ui};

pub type TransformId = ResourceId;
pub type TransformRc = ResourceRef<Transform>;

pub struct Transform {
    id: ResourceId,
    position: Vector3,
    rotation: Vector3,
    scale: Vector3,
}

impl ResourceData for Transform {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn info(&self) -> String {
        format!(
            "Matrix {:?}
            Position {:?}
            Rotation {:?}
            Scale {:?}",
            self.id().to_simple().to_string(),
            self.position,
            self.rotation,
            self.scale
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            position: Vector3::default_zero(),
            rotation: Vector3::default_zero(),
            scale: Vector3::default_one(),
        }
    }
}

impl UIProperties for Transform {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Position: ");
                self.position.show(ui_registry, ui);
            });
            ui.horizontal(|ui| {
                ui.label("Rotation: ");
                let mut rotation = self.rotation.to_degrees();
                rotation.show(ui_registry, ui);
                self.rotation = rotation.to_radians();
            });
            ui.horizontal(|ui| {
                ui.label("Scale: ");
                self.scale.show(ui_registry, ui);
            });
        });
    }
}

impl Transform {
    pub fn matrix(&self) -> Matrix4 {
        let mut matrix = Matrix4::default_identity();
        matrix.from_translation_rotation_scale(self.position, self.rotation, self.scale);
        matrix
    }

    pub fn set_matrix(&mut self, matrix: Matrix4) {
        let (translation, rotation, scale) = matrix.get_translation_rotation_scale();
        self.position = translation;
        self.rotation = rotation;
        self.scale = scale;
    }
}
