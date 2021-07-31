use egui::{DragValue, Ui};
use nrg_math::Vector3;

pub trait UIProperties {
    fn show(&mut self, ui: &mut Ui);
}

impl UIProperties for Vector3 {
    fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: "));
            ui.add(DragValue::new(&mut self.y).prefix("y: "));
            ui.add(DragValue::new(&mut self.z).prefix("z: "));
        });
    }
}
