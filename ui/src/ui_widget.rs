use std::any::Any;

use egui::CtxRef;
use nrg_resources::{ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

pub type UIWidgetId = ResourceId;
pub type UIWidgetRc = ResourceRef<UIWidget>;

pub trait UIWidgetData: Send + Sync + Any {
    fn as_any(&mut self) -> &mut dyn Any;
}
#[macro_export]
macro_rules! implement_widget_data {
    ($Type:ident) => {
        impl $crate::UIWidgetData for $Type {
            #[inline]
            fn as_any(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

pub struct UIWidget {
    id: ResourceId,
    data: Box<dyn UIWidgetData>,
    func: Box<dyn FnMut(&mut dyn UIWidgetData, &CtxRef)>,
}

unsafe impl Send for UIWidget {}
unsafe impl Sync for UIWidget {}

impl ResourceData for UIWidget {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl UIWidget {
    pub fn register<D, F>(shared_data: &SharedDataRw, data: D, f: F) -> UIWidgetRc
    where
        D: UIWidgetData + Sized + 'static,
        F: FnMut(&mut dyn UIWidgetData, &CtxRef) + 'static,
    {
        let ui_page = Self {
            id: generate_random_uid(),
            data: Box::new(data),
            func: Box::new(f),
        };
        SharedData::add_resource::<UIWidget>(shared_data, ui_page)
    }

    pub fn execute(&mut self, ui_context: &CtxRef) {
        (self.func)(self.data.as_mut(), ui_context);
    }
}
