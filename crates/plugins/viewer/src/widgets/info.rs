use inox_core::ContextRc;
use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SharedDataRc};
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
            ui_page: Self::create(context.shared_data(), context.message_hub(), data),
        }
    }

    fn create(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data: Data,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, message_hub, data, |ui_data, ui_context| {
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
