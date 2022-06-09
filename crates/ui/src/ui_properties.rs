use std::{any::TypeId, marker::PhantomData};

use egui::{Checkbox, CollapsingHeader, DragValue, TextEdit, Ui, Widget};
use inox_graphics::{
    Font, Light, LightType, Material, Mesh, RenderPipeline, Texture, View, MESH_FLAGS_VISIBLE,
};
use inox_math::{Degrees, Matrix4, Vector2, Vector3, Vector4};
use inox_resources::{
    GenericResource, ResourceCastTo, ResourceId, ResourceTrait, SerializableResource,
};

pub trait UIProperties {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    );
}

trait UIData {
    fn type_id(&self) -> TypeId;
    fn show(
        &self,
        resource: &GenericResource,
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
    T: UIProperties + ResourceTrait + 'static,
{
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn show(
        &self,
        resource: &GenericResource,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        let resource = resource.of_type::<T>();
        resource
            .get_mut()
            .show(resource.id(), ui_registry, ui, collapsed);
    }
}

#[derive(Default)]
pub struct UIPropertiesRegistry {
    registry: Vec<Box<dyn UIData>>,
}

unsafe impl Send for UIPropertiesRegistry {}
unsafe impl Sync for UIPropertiesRegistry {}

impl UIPropertiesRegistry {
    pub fn register<T>(&mut self) -> &mut Self
    where
        T: UIProperties + ResourceTrait + 'static,
    {
        self.registry.push(Box::new(UIPropertiesData {
            _marker: PhantomData::<T>::default(),
        }));
        self
    }
    pub fn show(&self, typeid: TypeId, resource: &GenericResource, ui: &mut Ui) {
        if let Some(index) = self.registry.iter().position(|e| e.type_id() == typeid) {
            self.registry[index]
                .as_ref()
                .show(resource, self, ui, false);
        } else {
            panic!("Trying to create an type not registered {:?}", typeid);
        }
    }
}

impl UIProperties for Degrees {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        _collapsed: bool,
    ) {
        ui.horizontal(|ui| {
            let mut value = self.0;
            let drag = DragValue::new(&mut value)
                .suffix("°")
                .prefix("degrees: ")
                .clamp_range(0.0..=360.0)
                .fixed_decimals(3);
            drag.ui(ui);
            self.0 = value;
        });
    }
}

impl UIProperties for f32 {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        _collapsed: bool,
    ) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(self).prefix("value: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for usize {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        _collapsed: bool,
    ) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(self).prefix("value: "));
        });
    }
}

impl UIProperties for Vector2 {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        _collapsed: bool,
    ) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector3 {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        _collapsed: bool,
    ) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Vector4 {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        _collapsed: bool,
    ) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.w).prefix("w: ").fixed_decimals(3));
        });
    }
}

impl UIProperties for Matrix4 {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        ui.vertical(|ui| {
            self.x.show(id, ui_registry, ui, collapsed);
            self.y.show(id, ui_registry, ui, collapsed);
            self.z.show(id, ui_registry, ui, collapsed);
            self.w.show(id, ui_registry, ui, collapsed);
        });
    }
}

impl UIProperties for RenderPipeline {
    fn show(
        &mut self,
        id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Pipeline [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Path: ");
                        let mut path = self.path().to_str().unwrap_or_default().to_string();
                        TextEdit::singleline(&mut path).interactive(false).ui(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Vertex Shader: ");
                        let mut shader = self
                            .data()
                            .vertex_shader
                            .to_str()
                            .unwrap_or_default()
                            .to_string();
                        TextEdit::singleline(&mut shader).interactive(false).ui(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Fragment Shader: ");
                        let mut shader = self
                            .data()
                            .fragment_shader
                            .to_str()
                            .unwrap_or_default()
                            .to_string();
                        TextEdit::singleline(&mut shader).interactive(false).ui(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Culling Type: ");
                        let mut culling = format!("{:?}", self.data().culling);
                        TextEdit::singleline(&mut culling).interactive(false).ui(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Poligon Mode: ");
                        let mut mode = format!("{:?}", self.data().mode);
                        TextEdit::singleline(&mut mode).interactive(false).ui(ui);
                    });
                });
            });
    }
}

impl UIProperties for Font {
    fn show(
        &mut self,
        id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Font [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    let mut path = self.path().to_str().unwrap().to_string();
                    TextEdit::singleline(&mut path).interactive(false).ui(ui);
                });
            });
    }
}

impl UIProperties for Material {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Material [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    let mut path = self.path().to_str().unwrap().to_string();
                    TextEdit::singleline(&mut path).interactive(false).ui(ui);
                });
                ui.collapsing(format!("Textures [{}]", self.textures().len()), |ui| {
                    self.textures().iter().for_each(|t| {
                        if let Some(t) = t {
                            let id = t.id();
                            t.get_mut().show(id, ui_registry, ui, collapsed);
                        }
                    });
                });
            });
    }
}

impl UIProperties for Mesh {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Mesh [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    let mut path = self.path().to_str().unwrap().to_string();
                    TextEdit::singleline(&mut path).interactive(false).ui(ui);
                });
                let mut is_visible = self.has_flags(MESH_FLAGS_VISIBLE);
                Checkbox::new(&mut is_visible, "Visible").ui(ui);
                if is_visible {
                    self.add_flag(MESH_FLAGS_VISIBLE);
                } else {
                    self.remove_flag(MESH_FLAGS_VISIBLE);
                }
                if let Some(material) = self.material() {
                    let id = material.id();
                    material.get_mut().show(id, ui_registry, ui, true);
                }
            });
    }
}

impl UIProperties for Texture {
    fn show(
        &mut self,
        id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Texture [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path: ");
                    let mut path = self.path().to_str().unwrap().to_string();
                    TextEdit::singleline(&mut path).interactive(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Texture Index: ");
                    let mut texture_index = format!("{}", self.texture_index());
                    TextEdit::singleline(&mut texture_index)
                        .interactive(false)
                        .ui(ui);
                });
                ui.horizontal(|ui| {
                    ui.label("Dimensions: ");
                    let mut width = format!("{}", self.dimensions().0);
                    TextEdit::singleline(&mut width).interactive(false).ui(ui);
                    ui.label("x");
                    let mut heigth = format!("{}", self.dimensions().1);
                    TextEdit::singleline(&mut heigth).interactive(false).ui(ui);
                });
            });
    }
}

impl UIProperties for View {
    fn show(
        &mut self,
        _id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("View [{:?}]", self.view_index()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Index: ");
                    let mut index = format!("{}", self.view_index());
                    TextEdit::singleline(&mut index).interactive(false).ui(ui);
                });
            });
    }
}

impl UIProperties for Light {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Light [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Type: ");
                    if self.data().light_type == LightType::Directional as u32 {
                        ui.label("Directional");
                    } else if self.data().light_type == LightType::Point as u32 {
                        ui.label("Point");
                    } else if self.data().light_type == LightType::Spot as u32 {
                        ui.label("Spot");
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Color: ");
                    let mut color: Vector4 = self.data().color.into();
                    color.show(id, ui_registry, ui, collapsed);
                    self.data_mut().color = color.into();
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity: ");
                    self.data_mut()
                        .intensity
                        .show(id, ui_registry, ui, collapsed);
                });
                ui.horizontal(|ui| {
                    ui.label("Range: ");
                    self.data_mut().range.show(id, ui_registry, ui, collapsed);
                });
            });
    }
}
