use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use nrg_graphics::{Font, Material, Mesh, Pipeline, Texture, View};
use nrg_resources::{Resource, ResourceTrait, SharedData, SharedDataRc};
use nrg_scene::{Camera, Hitbox, Object, Scene};
use nrg_ui::{implement_widget_data, UIProperties, UIPropertiesRegistry, UIWidget, Ui, Window};

struct DebugData {
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRc,
    ui_registry: Arc<UIPropertiesRegistry>,
}
implement_widget_data!(DebugData);

pub struct DebugInfo {
    ui_page: Resource<UIWidget>,
}

impl DebugInfo {
    pub fn new(shared_data: &SharedDataRc, ui_registry: Arc<UIPropertiesRegistry>) -> Self {
        let data = DebugData {
            frame_seconds: VecDeque::default(),
            shared_data: shared_data.clone(),
            ui_registry,
        };
        Self {
            ui_page: Self::create(shared_data, data),
        }
    }

    fn create(shared_data: &SharedDataRc, data: DebugData) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<DebugData>() {
                let now = Instant::now();
                let one_sec_before = now - Duration::from_secs(1);
                data.frame_seconds.push_back(now);
                data.frame_seconds.retain(|t| *t >= one_sec_before);

                Window::new("Stats")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!("FPS: {}", data.frame_seconds.len()));
                        ui.separator();
                        Self::resource_ui_properties::<Pipeline>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Pipeline",
                        );
                        Self::resource_ui_properties::<Font>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Font",
                        );
                        Self::resource_ui_properties::<Material>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Material",
                        );
                        Self::resource_ui_properties::<Texture>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Texture",
                        );
                        Self::resource_ui_properties::<Mesh>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Mesh",
                        );
                        ui.separator();
                        Self::resource_ui_properties::<View>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "View",
                        );
                        ui.separator();
                        Self::resource_ui_properties::<Scene>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Scene",
                        );
                        Self::resource_ui_properties::<Object>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Object",
                        );
                        Self::resource_ui_properties::<Camera>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Camera",
                        );
                        Self::resource_ui_properties::<Hitbox>(
                            &data.shared_data,
                            &data.ui_registry,
                            ui,
                            "Hitbox",
                        );
                    });
            }
        })
    }

    fn resource_ui_properties<R>(
        shared_data: &SharedDataRc,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        title: &str,
    ) where
        R: ResourceTrait + UIProperties,
    {
        ui.collapsing(
            format!(
                "{}: {}",
                title,
                SharedData::get_num_resources_of_type::<R>(shared_data)
            ),
            |ui| {
                SharedData::for_each_resource_mut(shared_data, |rh, r: &mut R| {
                    r.show(rh.id(), ui_registry, ui, true);
                });
            },
        );
    }
}
