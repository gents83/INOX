use std::{any::TypeId, marker::PhantomData};

use egui::{Checkbox, DragValue, TextEdit, Ui, Widget};
use nrg_graphics::{
    FontInstance, MaterialInstance, MeshInstance, PipelineInstance, TextureInstance, ViewInstance,
};
use nrg_math::{Vector2, Vector3, Vector4};
use nrg_resources::{FileResource, GenericRef, HandleCastTo, ResourceData, SerializableResource};
pub trait UIProperties {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui);
}

trait UIData {
    fn id(&self) -> TypeId;
    fn show(&self, handle: &GenericRef, ui_registry: &UIPropertiesRegistry, ui: &mut Ui);
}

struct UIPropertiesData<T> {
    _marker: PhantomData<T>,
}
impl<T> UIData for UIPropertiesData<T>
where
    T: UIProperties + ResourceData,
{
    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn show(&self, handle: &GenericRef, ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        let handle = handle.clone().of_type::<T>();
        handle.resource().get_mut().show(ui_registry, ui);
    }
}
pub struct UIPropertiesRegistry {
    registry: Vec<Box<dyn UIData>>,
}

unsafe impl Send for UIPropertiesRegistry {}
unsafe impl Sync for UIPropertiesRegistry {}

impl Default for UIPropertiesRegistry {
    fn default() -> Self {
        Self {
            registry: Vec::new(),
        }
    }
}
impl UIPropertiesRegistry {
    pub fn register<T>(&mut self) -> &mut Self
    where
        T: UIProperties + ResourceData,
    {
        self.registry.push(Box::new(UIPropertiesData {
            _marker: PhantomData::<T>::default(),
        }));
        self
    }
    pub fn show(&self, typeid: TypeId, handle: &GenericRef, ui: &mut Ui) {
        if let Some(index) = self.registry.iter().position(|e| e.id() == typeid) {
            self.registry[index].as_ref().show(handle, self, ui);
        } else {
            panic!("Trying to create an type not registered {:?}", typeid);
        }
    }
}

impl UIProperties for f32 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(self).prefix("value: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector2 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector3 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector4 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("w: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for PipelineInstance {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Name: ");
                let mut name = self.data().name.clone();
                TextEdit::singleline(&mut name).enabled(false).ui(ui);
            });
        });
    }
}

impl UIProperties for FontInstance {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Path: ");
                let mut path = self.path().to_str().unwrap().to_string();
                TextEdit::singleline(&mut path).enabled(false).ui(ui);
            });
        });
    }
}

impl UIProperties for MaterialInstance {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Path: ");
                let mut path = self.path().to_str().unwrap().to_string();
                TextEdit::singleline(&mut path).enabled(false).ui(ui);
            });
            self.pipeline().resource().get_mut().show(ui_registry, ui);
            ui.collapsing(format!("Textures [{}]", self.textures().len()), |ui| {
                for t in self.textures() {
                    t.resource().get_mut().show(ui_registry, ui);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Diffuse Color: ");
                self.diffuse_color().show(ui_registry, ui);
            });
            ui.horizontal(|ui| {
                ui.label("Outline Color: ");
                self.outline_color().show(ui_registry, ui);
            });
        });
    }
}

impl UIProperties for MeshInstance {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Path: ");
                let mut path = self.path().to_str().unwrap().to_string();
                TextEdit::singleline(&mut path).enabled(false).ui(ui);
            });
            let mut is_visible = self.is_visible();
            Checkbox::new(&mut is_visible, "Visible").ui(ui);
            self.set_visible(is_visible);
            ui.horizontal(|ui| {
                ui.label("Num vertices: ");
                let mut vertices = format!("{}", self.mesh_data().vertices.len());
                TextEdit::singleline(&mut vertices).enabled(false).ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Draw Area: ");
                self.draw_area().show(ui_registry, ui);
            });
            self.material().resource().get_mut().show(ui_registry, ui);
        });
    }
}

impl UIProperties for TextureInstance {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Path: ");
                let mut path = self.path().to_str().unwrap().to_string();
                TextEdit::singleline(&mut path).enabled(false).ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Texture Index: ");
                let mut texture_index = format!("{}", self.texture_index());
                TextEdit::singleline(&mut texture_index)
                    .enabled(false)
                    .ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Layer Index: ");
                let mut layer_index = format!("{}", self.layer_index());
                TextEdit::singleline(&mut layer_index).enabled(false).ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Dimensions: ");
                let mut width = format!("{}", self.dimensions().0);
                TextEdit::singleline(&mut width).enabled(false).ui(ui);
                ui.label("x");
                let mut heigth = format!("{}", self.dimensions().1);
                TextEdit::singleline(&mut heigth).enabled(false).ui(ui);
            });
        });
    }
}

impl UIProperties for ViewInstance {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.collapsing(self.id().to_simple().to_string(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Index: ");
                let mut index = format!("{}", self.view_index());
                TextEdit::singleline(&mut index).enabled(false).ui(ui);
            });
        });
    }
}
