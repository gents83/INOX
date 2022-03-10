use inox_core::ContextRc;
use inox_resources::Resource;
use inox_ui::{implement_widget_data, UIWidget, Window};

struct Data {
    context: ContextRc,
}
implement_widget_data!(Data);

pub struct Info {
    ui_page: Resource<UIWidget>,
}

impl Info {
    pub fn new(context: &ContextRc) -> Self {
        let data = Data {
            context: context.clone(),
        };
        Self {
            ui_page: Self::create(data),
        }
    }

    fn create(data: Data) -> Resource<UIWidget> {
        let shared_data = data.context.shared_data().clone();
        let message_hub = data.context.message_hub().clone();
        UIWidget::register(&shared_data, &message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<Data>() {
                Window::new("Stats")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!(
                            "FPS: {} - ms: {:?}",
                            data.context.global_timer().fps(),
                            data.context.global_timer().dt().as_millis()
                        ));
                    });
            }
        })
    }
}
