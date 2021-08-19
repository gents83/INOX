use std::sync::Arc;

use nrg_resources::{ResourceData, SharedData, SharedDataRw};
use nrg_scene::{Object, ObjectId};
use nrg_serialize::INVALID_UID;
use nrg_ui::{
    implement_widget_data, UIProperties, UIPropertiesRegistry, UIWidget, UIWidgetRc, Ui, Window,
};

struct PropertiesData {
    shared_data: SharedDataRw,
    ui_registry: Arc<UIPropertiesRegistry>,
    selected_object: ObjectId,
}
implement_widget_data!(PropertiesData);

pub struct Properties {
    ui_page: UIWidgetRc,
}

impl Properties {
    pub fn new(shared_data: &SharedDataRw, ui_registry: Arc<UIPropertiesRegistry>) -> Self {
        let data = PropertiesData {
            shared_data: shared_data.clone(),
            ui_registry,
            selected_object: INVALID_UID,
        };
        Self {
            ui_page: Self::create(shared_data, data),
        }
    }

    fn create(shared_data: &SharedDataRw, data: PropertiesData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<PropertiesData>() {
                Window::new("Properties")
                    .scroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        if !data.selected_object.is_nil() {
                            Self::resource_ui_properties::<Object>(
                                &data.shared_data,
                                &data.ui_registry,
                                ui,
                                data.selected_object,
                            );
                        }
                    });
            }
        })
    }

    fn resource_ui_properties<R>(
        shared_data: &SharedDataRw,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        object_id: ObjectId,
    ) where
        R: ResourceData + UIProperties,
    {
        ui.collapsing(
            format!("Object: {}", object_id.to_string().as_str(),),
            |ui| {
                let object = SharedData::get_resource::<R>(shared_data, object_id);
                object.resource().get_mut().show(ui_registry, ui);
            },
        );
    }
}
