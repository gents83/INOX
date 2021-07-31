use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use nrg_graphics::{
    FontInstance, MaterialInstance, MeshInstance, PipelineInstance, TextureInstance,
};
use nrg_resources::{ResourceData, SharedData, SharedDataRw};
use nrg_scene::{Object, Scene, Transform};
use nrg_ui::{implement_widget_data, UIProperties, UIWidget, UIWidgetRc, Ui, Window};

struct DebugData {
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRw,
}
implement_widget_data!(DebugData);

pub struct DebugInfo {
    ui_page: UIWidgetRc,
}

impl DebugInfo {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        let data = DebugData {
            frame_seconds: VecDeque::default(),
            shared_data: shared_data.clone(),
        };
        let ui_page = Self::create(shared_data, data);
        Self { ui_page }
    }

    fn create(shared_data: &SharedDataRw, data: DebugData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<DebugData>() {
                let now = Instant::now();
                let one_sec_before = now - Duration::from_secs(1);
                data.frame_seconds.push_back(now);
                data.frame_seconds.retain(|t| *t >= one_sec_before);

                Window::new("Stats")
                    .scroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!("FPS: {}", data.frame_seconds.len()));
                        ui.separator();
                        Self::resource_ui::<PipelineInstance>(&data.shared_data, ui, "Pipeline");
                        Self::resource_ui::<FontInstance>(&data.shared_data, ui, "Font");
                        Self::resource_ui::<MaterialInstance>(&data.shared_data, ui, "Material");
                        Self::resource_ui::<TextureInstance>(&data.shared_data, ui, "Texture");
                        Self::resource_ui::<MeshInstance>(&data.shared_data, ui, "Mesh");
                        ui.separator();
                        Self::resource_ui::<Scene>(&data.shared_data, ui, "Scene");
                        Self::resource_ui::<Object>(&data.shared_data, ui, "Object");
                        //Self::resource_ui::<Transform>(&data.shared_data, ui, "Transform");
                        Self::resource_ui_properties(&data.shared_data, ui, "Transform");
                    });
            }
        })
    }

    fn resource_ui<R>(shared_data: &SharedDataRw, ui: &mut Ui, title: &str)
    where
        R: ResourceData,
    {
        ui.collapsing(
            format!(
                "{}: {}",
                title,
                SharedData::get_num_resources_of_type::<R>(shared_data)
            ),
            |ui| {
                let resources = SharedData::get_resources_of_type::<R>(shared_data);
                for r in resources.iter() {
                    let string = r.resource().info();
                    let mut lines = string.lines();
                    ui.collapsing(lines.next().unwrap(), |ui| {
                        for l in lines {
                            ui.label(l);
                        }
                    });
                }
            },
        );
    }

    fn resource_ui_properties(shared_data: &SharedDataRw, ui: &mut Ui, title: &str) {
        ui.collapsing(
            format!(
                "{}: {}",
                title,
                SharedData::get_num_resources_of_type::<Transform>(shared_data)
            ),
            |ui| {
                let resources = SharedData::get_resources_of_type::<Transform>(shared_data);
                for r in resources.iter() {
                    r.resource().get_mut().show(ui);
                }
            },
        );
    }
}
