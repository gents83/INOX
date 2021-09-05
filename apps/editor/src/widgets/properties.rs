use std::sync::Arc;

use nrg_resources::{Resource, SharedData, SharedDataRw};
use nrg_scene::{Object, ObjectId};
use nrg_serialize::INVALID_UID;
use nrg_ui::{
    implement_widget_data, ScrollArea, SidePanel, UIProperties, UIPropertiesRegistry, UIWidget, Ui,
};

#[derive(Clone)]
struct PropertiesRuntimeData {
    width: f32,
}

struct PropertiesData {
    shared_data: SharedDataRw,
    ui_registry: Arc<UIPropertiesRegistry>,
    selected_object: ObjectId,
}
implement_widget_data!(PropertiesData);

pub struct Properties {
    ui_page: Resource<UIWidget>,
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
        if let Some(data) = self.ui_page.get_mut().data_mut::<PropertiesData>() {
            data.selected_object = object_id;
        }
        self
    }

    fn create(shared_data: &SharedDataRw, data: PropertiesData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<PropertiesData>() {
                let min_width = 200.;
                let mut width = min_width;
                if let Some(panel_runtime_data) =
                    ui_context.memory().data_temp.get::<PropertiesRuntimeData>()
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
                            ScrollArea::auto_sized().show(ui, |ui| {
                                Self::object_properties(
                                    &data.shared_data,
                                    &data.ui_registry,
                                    ui,
                                    data.selected_object,
                                );
                            });
                        }
                    });

                ui_context.memory().data_temp.insert(PropertiesRuntimeData {
                    width: width.max(min_width),
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
        if SharedData::has::<Object>(shared_data, object_id) {
            let object = SharedData::get_resource::<Object>(shared_data, object_id);

            object.get_mut().show(ui_registry, ui, false);
        }
    }
}
