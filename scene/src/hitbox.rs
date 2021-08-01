use nrg_math::{Mat4Ops, MatBase, Matrix4, VecBase, Vector3};
use nrg_resources::{ResourceData, ResourceId, ResourceRef};
use nrg_serialize::generate_random_uid;
use nrg_ui::{UIProperties, UIPropertiesRegistry, Ui};

pub type HitboxId = ResourceId;
pub type HitboxRc = ResourceRef<Hitbox>;

pub struct Hitbox {
    id: ResourceId,
    min: Vector3,
    max: Vector3,
    transform: Matrix4,
}

impl ResourceData for Hitbox {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl UIProperties for Hitbox {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Min: ");
                self.min.show(ui_registry, ui);
            });
            ui.horizontal(|ui| {
                ui.label("Diffuse Color: ");
                self.max.show(ui_registry, ui);
            });
        });
    }
}

impl Default for Hitbox {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            min: Vector3::default_zero(),
            max: Vector3::default_zero(),
            transform: Matrix4::default_identity(),
        }
    }
}

impl Hitbox {
    #[inline]
    pub fn set_transform(&mut self, matrix: Matrix4) {
        self.transform = matrix;
    }
    #[inline]
    pub fn set_dimensions(&mut self, min: Vector3, max: Vector3) {
        self.min = min;
        self.max = max;
    }

    #[inline]
    pub fn min(&self) -> Vector3 {
        self.transform.transform(self.min)
    }
    #[inline]
    pub fn max(&self) -> Vector3 {
        self.transform.transform(self.max)
    }
}
