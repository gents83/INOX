use std::sync::Arc;

use sabi_resources::{Resource, SharedData, SharedDataRc};
use sabi_scene::{Object, ObjectId};
use sabi_serialize::INVALID_UID;
use sabi_ui::{
    implement_widget_data, Id, ScrollArea, SidePanel, UIProperties, UIPropertiesRegistry, UIWidget,
    Ui,
};

#[derive(Clone)]
struct PropertiesRuntimeData {
    width: f32,
}

struct PropertiesData {
    shared_data: SharedDataRc,
    ui_registry: Arc<UIPropertiesRegistry>,
    selected_object: ObjectId,
    id: Id,
}
implement_widget_data!(PropertiesData);

pub struct Properties {
    ui_page: Resource<UIWidget>,
}

impl Properties {
    pub fn new(shared_data: &SharedDataRc, ui_registry: Arc<UIPropertiesRegistry>) -> Self {
        let data = PropertiesData {
            shared_data: shared_data.clone(),
            ui_registry,
            selected_object: INVALID_UID,
            id: Id::new("PropertiesData"),
        };
        Self {
            ui_page: Self::create(shared_data, data),
        }
    }

    pub fn select_object(&mut self, object_id: ObjectId) -> &mut Self {
        if let Some(data) = self.ui_page.get_mut().data_mut::<PropertiesData>() {
            data.selected_object = object_id;
        }
        self
    }

    fn create(shared_data: &SharedDataRc, data: PropertiesData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<PropertiesData>() {
                let min_width = 200.;
                let mut width = min_width;
                if let Some(panel_runtime_data) = ui_context
                    .memory()
                    .data
                    .get_temp::<PropertiesRuntimeData>(data.id)
                {
                    width = panel_runtime_data.width;
                }
                SidePanel::right("Properties")
                    .resizable(true)
                    .min_width(width)
                    .show(ui_context, |ui| {
                        width = ui.available_width();
                        ui.heading("Properties:");

                        if !data.selected_object.is_nil() {
                            ScrollArea::vertical().show(ui, |ui| {
                                Self::object_properties(
                                    &data.shared_data,
                                    &data.ui_registry,
                                    ui,
                                    &data.selected_object,
                                );
                            });
                        }
                    });

                ui_context.memory().data.insert_temp(
                    data.id,
                    PropertiesRuntimeData {
                        width: width.max(min_width),
                    },
                );
            }
        })
    }

    fn object_properties(
        shared_data: &SharedDataRc,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        object_id: &ObjectId,
    ) {
        if let Some(object) = SharedData::get_resource::<Object>(shared_data, object_id) {
            object.get_mut().show(object_id, ui_registry, ui, false);
        }
    }
}
