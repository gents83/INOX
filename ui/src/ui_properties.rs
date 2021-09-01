use std::{any::TypeId, marker::PhantomData};

use egui::{Checkbox, CollapsingHeader, DragValue, TextEdit, Ui, Widget};
use nrg_graphics::{Font, Material, Mesh, Pipeline, Texture, View};
use nrg_math::{Vector2, Vector3, Vector4};
use nrg_resources::{FileResource, GenericRef, HandleCastTo, ResourceData, SerializableResource};
pub trait UIProperties {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool);
}

trait UIData {
    fn id(&self) -> TypeId;
    fn show(
        &self,
        handle: &GenericRef,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    );
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
    fn show(
        &self,
        handle: &GenericRef,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        let handle = handle.clone().of_type::<T>();
        handle.resource().get_mut().show(ui_registry, ui, collapsed);
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
            self.registry[index].as_ref().show(handle, self, ui, false);
        } else {
            panic!("Trying to create an type not registered {:?}", typeid);
        }
    }
}

impl UIProperties for f32 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, _collapsed: bool) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(self).prefix("value: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector2 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, _collapsed: bool) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector3 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, _collapsed: bool) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector4 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, _collapsed: bool) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.w).prefix("w: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Pipeline {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!(
            "Pipeline [{:?}]",
            self.id().to_simple().to_string()
        ))
        .show_background(true)
        .default_open(!collapsed)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    let mut path = self.data().path().to_str().unwrap_or_default().to_string();
                    TextEdit::singleline(&mut path).enabled(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("VertexShader: ");
                    let mut shader = self
                        .data()
                        .vertex_shader
                        .to_str()
                        .unwrap_or_default()
                        .to_string();
                    TextEdit::singleline(&mut shader).enabled(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("FragmentShader: ");
                    let mut shader = self
                        .data()
                        .fragment_shader
                        .to_str()
                        .unwrap_or_default()
                        .to_string();
                    TextEdit::singleline(&mut shader).enabled(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Culling Type: ");
                    let mut culling = format!("{:?}", self.data().culling);
                    TextEdit::singleline(&mut culling).enabled(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Poligon Mode: ");
                    let mut mode = format!("{:?}", self.data().mode);
                    TextEdit::singleline(&mut mode).enabled(false).ui(ui);
                });
            });
        });
    }
}

impl UIProperties for Font {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!("Font [{:?}]", FileResource::get_name(self)))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    let mut path = self.path().to_str().unwrap().to_string();
                    TextEdit::singleline(&mut path).enabled(false).ui(ui);
                });
            });
    }
}

impl UIProperties for Material {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!(
            "Material [{:?}]",
            SerializableResource::get_name(self)
        ))
        .show_background(true)
        .default_open(!collapsed)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Path: ");
                let mut path = self.path().to_str().unwrap().to_string();
                TextEdit::singleline(&mut path).enabled(false).ui(ui);
            });
            self.pipeline()
                .resource()
                .get_mut()
                .show(ui_registry, ui, true);
            ui.collapsing(format!("Textures [{}]", self.textures().len()), |ui| {
                for t in self.textures() {
                    t.resource().get_mut().show(ui_registry, ui, collapsed);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Diffuse Color: ");
                let mut diffuse_color = self.diffuse_color();
                diffuse_color.show(ui_registry, ui, collapsed);
                self.set_diffuse_color(diffuse_color);
            });
            ui.horizontal(|ui| {
                ui.label("Outline Color: ");
                let mut outline_color = self.outline_color();
                outline_color.show(ui_registry, ui, collapsed);
                self.set_outline_color(outline_color);
            });
        });
    }
}

impl UIProperties for Mesh {
    fn show(&mut self, ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!("Mesh [{:?}]", SerializableResource::get_name(self)))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
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
                self.material()
                    .resource()
                    .get_mut()
                    .show(ui_registry, ui, true);
            });
    }
}

impl UIProperties for Texture {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!("Texture [{:?}]", FileResource::get_name(self)))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
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

impl UIProperties for View {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui, collapsed: bool) {
        CollapsingHeader::new(format!("View [{:?}]", self.get_name()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Index: ");
                    let mut index = format!("{}", self.view_index());
                    TextEdit::singleline(&mut index).enabled(false).ui(ui);
                });
            });
    }
}
