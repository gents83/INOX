use std::any::{type_name, Any};

use egui::{CollapsingHeader, Context, Ui};
use inox_messenger::MessageHubRc;
use inox_resources::{Resource, ResourceId, ResourceTrait, SharedDataRc};
use inox_uid::generate_random_uid;

use crate::{UIProperties, UIPropertiesRegistry};

pub type UIWidgetId = ResourceId;

pub trait UIWidgetData: Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_boxed(&self) -> Box<dyn UIWidgetData>;
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
            fn as_boxed(&self) -> Box<dyn $crate::UIWidgetData> {
                Box::new(self.clone())
            }
        }
    };
}
impl Clone for Box<dyn UIWidgetData> {
    fn clone(&self) -> Self {
        (**self).as_boxed()
    }
}

pub trait UIWidgetUpdateFn: FnMut(&mut dyn UIWidgetData, &Context) -> bool {
    fn as_boxed(&self) -> Box<dyn UIWidgetUpdateFn>;
}
impl<F> UIWidgetUpdateFn for F
where
    F: 'static + FnMut(&mut dyn UIWidgetData, &Context) -> bool + Clone,
{
    fn as_boxed(&self) -> Box<dyn UIWidgetUpdateFn> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn UIWidgetUpdateFn> {
    fn clone(&self) -> Self {
        (**self).as_boxed()
    }
}

#[derive(Clone)]
pub struct UIWidget {
    type_name: String,
    data: Box<dyn UIWidgetData>,
    func: Box<dyn UIWidgetUpdateFn>,
    is_interacting: bool,
}

impl ResourceTrait for UIWidget {
    fn is_initialized(&self) -> bool {
        true
    }
    fn invalidate(&mut self) -> &mut Self {
        eprintln!("UIWidget cannot be invalidated!");
        self
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
            id.as_simple().to_string()
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
    pub fn register<D, F>(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data: D,
        f: F,
    ) -> Resource<Self>
    where
        D: UIWidgetData + Sized,
        F: FnMut(&mut dyn UIWidgetData, &Context) -> bool + 'static + Clone,
    {
        let ui_page = Self {
            type_name: type_name::<D>().to_string(),
            data: Box::new(data),
            func: Box::new(f),
            is_interacting: false,
        };
        shared_data.add_resource::<UIWidget>(message_hub, generate_random_uid(), ui_page)
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

    pub fn execute(&mut self, ui_context: &Context) {
        inox_profiler::scoped_profile!("{} {:?}", "ui_widget::execute", self.type_name);
        self.is_interacting = (self.func)(self.data.as_mut(), ui_context);
        #[allow(deprecated)]
        {
            self.is_interacting |= ui_context.is_using_pointer();
        }
    }
    pub fn is_interacting(&self) -> bool {
        self.is_interacting
    }
}
