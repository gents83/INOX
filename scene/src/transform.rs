use nrg_math::{Mat4Ops, MatBase, Matrix4, VecBase, Vector3};
use nrg_resources::{ResourceId, ResourceTrait};
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

pub type TransformId = ResourceId;

#[derive(Clone)]
pub struct Transform {
    position: Vector3,
    rotation: Vector3,
    scale: Vector3,
}
impl ResourceTrait for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::default_zero(),
            rotation: Vector3::default_zero(),
            scale: Vector3::default_one(),
        }
    }
}

impl UIProperties for Transform {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Transform [{:?}]", id.to_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position: ");
                    self.position.show(id, ui_registry, ui, false);
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation: ");
                    let mut rotation = self.rotation.to_degrees();
                    rotation.show(id, ui_registry, ui, false);
                    self.rotation = rotation.to_radians();
                });
                ui.horizontal(|ui| {
                    ui.label("Scale: ");
                    self.scale.show(id, ui_registry, ui, false);
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
    pub fn position(&self) -> Vector3 {
        self.position
    }
    pub fn rotation(&self) -> Vector3 {
        self.rotation
    }
    pub fn scale(&self) -> Vector3 {
        self.scale
    }
    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
    }
    pub fn set_rotation(&mut self, rotation: Vector3) {
        self.rotation = rotation;
    }
    pub fn set_scale(&mut self, scale: Vector3) {
        self.scale = scale;
    }
    pub fn translate(&mut self, offset: Vector3) {
        self.position += offset;
    }
    pub fn rotate(&mut self, offset: Vector3) {
        self.rotation += offset;
    }
    pub fn add_scale(&mut self, scale: Vector3) {
        self.scale += scale;
    }
    pub fn set_matrix(&mut self, matrix: Matrix4) {
        let (translation, rotation, scale) = matrix.get_translation_rotation_scale();
        self.position = translation;
        self.rotation = rotation;
        self.scale = scale;
    }
}
