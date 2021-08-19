use std::sync::Arc;

use nrg_resources::{SharedData, SharedDataRw};
use nrg_scene::{Object, ObjectId};
use nrg_serialize::INVALID_UID;
use nrg_ui::{
    implement_widget_data, Align2, UIProperties, UIPropertiesRegistry, UIWidget, UIWidgetRc, Ui,
    Window,
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

    pub fn select_object(&mut self, object_id: ObjectId) -> &mut Self {
        if let Some(data) = self
            .ui_page
            .resource()
            .get_mut()
            .data_mut::<PropertiesData>()
        {
            data.selected_object = object_id;
        }
        self
    }

    fn create(shared_data: &SharedDataRw, data: PropertiesData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<PropertiesData>() {
                Window::new("Properties")
                    .scroll(true)
                    .anchor(Align2::RIGHT_TOP, [-10., 0.])
                    .title_bar(true)
                    .resizable(true)
                    .min_height(ui_context.available_rect().max.y)
                    .show(ui_context, |ui| {
                        if !data.selected_object.is_nil() {
                            Self::object_properties(
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

    fn object_properties(
        shared_data: &SharedDataRw,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        object_id: ObjectId,
    ) {
        if SharedData::has_resource::<Object>(shared_data, object_id) {
            let object = SharedData::get_resource::<Object>(shared_data, object_id);

            object.resource().get_mut().show(ui_registry, ui, false);
        }
    }
}
