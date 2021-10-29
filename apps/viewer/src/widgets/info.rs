use std::{
    collections::VecDeque,
    time::{Duration, SystemTime},
};

use nrg_resources::{Resource, SharedDataRc};
use nrg_ui::{implement_widget_data, UIWidget, Window};

struct Data {
    frame_seconds: VecDeque<SystemTime>,
    time: SystemTime,
    shared_data: SharedDataRc,
}
implement_widget_data!(Data);

pub struct Info {
    ui_page: Resource<UIWidget>,
}

impl Info {
    pub fn new(shared_data: &SharedDataRc) -> Self {
        let data = Data {
            time: SystemTime::now(),
            frame_seconds: VecDeque::default(),
            shared_data: shared_data.clone(),
        };
        Self {
            ui_page: Self::create(shared_data, data),
        }
    }

    fn create(shared_data: &SharedDataRc, data: Data) -> Resource<UIWidget> {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<Data>() {
                let last_time = data.time;
                data.time = SystemTime::now();
                let one_sec_before = data.time - Duration::from_secs(1);
                data.frame_seconds.push_back(data.time);
                data.frame_seconds.retain(|t| *t >= one_sec_before);

                Window::new("Stats")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!(
                            "FPS: {} - ms: {:?}",
                            data.frame_seconds.len(),
                            data.time.duration_since(last_time).unwrap().as_millis()
                        ));
                    });
            }
        })
    }
}
