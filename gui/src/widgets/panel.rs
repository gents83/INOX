use nrg_math::Vector2;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize};

use crate::{
    implement_widget_with_data, InternalWidget, WidgetData, WidgetEvent, DEFAULT_WIDGET_HEIGHT,
};

pub const DEFAULT_PANEL_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 10., DEFAULT_WIDGET_HEIGHT * 10.];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Panel {
    data: WidgetData,
}
implement_widget_with_data!(Panel);

impl InternalWidget for Panel {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_PANEL_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .style(WidgetStyle::Invisible);
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>();
    }
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
}
