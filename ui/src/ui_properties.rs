use std::{any::TypeId, marker::PhantomData};

use egui::{DragValue, Ui};
use nrg_math::Vector3;
use nrg_resources::{GenericRef, HandleCastTo, ResourceData};
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

impl UIProperties for Vector3 {
    fn show(&mut self, _ui_registry: &UIPropertiesRegistry, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).prefix("x: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.y).prefix("y: ").fixed_decimals(3));
            ui.add(DragValue::new(&mut self.z).prefix("z: ").fixed_decimals(3));
        });
    }
}
