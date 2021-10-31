use std::any::{type_name, Any};

use egui::{CollapsingHeader, CtxRef, Ui};
use nrg_resources::{Resource, ResourceId, ResourceTrait, SharedData, SharedDataRc};
use nrg_serialize::generate_random_uid;

use crate::{UIProperties, UIPropertiesRegistry};

pub type UIWidgetId = ResourceId;

pub trait UIWidgetData: Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
#[macro_export]
macro_rules! implement_widget_data {
    ($Type:ident) => {
        unsafe impl Sync for $Type {}
        unsafe impl Send for $Type {}

        impl $crate::UIWidgetData for $Type {
            #[inline]
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            #[inline]
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

pub struct UIWidget {
    type_name: String,
    data: Box<dyn UIWidgetData>,
    func: Box<dyn FnMut(&mut dyn UIWidgetData, &CtxRef)>,
}
impl ResourceTrait for UIWidget {
    fn on_resource_swap(&mut self, _new: &Self)
    where
        Self: Sized,
    {
        //println!("UIWidget resource swapped {:?}", self.type_name);
    }
}

unsafe impl Send for UIWidget {}
unsafe impl Sync for UIWidget {}

impl UIProperties for UIWidget {
    fn show(
        &mut self,
        id: &ResourceId,
        _ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!(
            "UIWidget_{:?} [{:?}]",
            self.type_name,
            id.to_simple().to_string()
        ))
        .show_background(true)
        .default_open(!collapsed)
        .show(ui, |ui| {
            let widget_name = type_name::<Self>()
                .split(':')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string();
            ui.label(widget_name);
        });
    }
}

impl UIWidget {
    pub fn register<D, F>(shared_data: &SharedDataRc, data: D, f: F) -> Resource<Self>
    where
        D: UIWidgetData + Sized,
        F: FnMut(&mut dyn UIWidgetData, &CtxRef) + 'static,
    {
        let ui_page = Self {
            type_name: type_name::<D>().to_string(),
            data: Box::new(data),
            func: Box::new(f),
        };
        SharedData::add_resource::<UIWidget>(shared_data, generate_random_uid(), ui_page)
    }

    pub fn data<D>(&self) -> Option<&D>
    where
        D: UIWidgetData + Sized,
    {
        self.data.as_any().downcast_ref::<D>()
    }

    pub fn data_mut<D>(&mut self) -> Option<&mut D>
    where
        D: UIWidgetData + Sized + 'static,
    {
        self.data.as_any_mut().downcast_mut::<D>()
    }

    pub fn execute(&mut self, ui_context: &CtxRef) {
        nrg_profiler::scoped_profile!(
            format!("{} {:?}", "ui_widget::execute", self.type_name).as_str()
        );
        (self.func)(self.data.as_mut(), ui_context);
    }
}
