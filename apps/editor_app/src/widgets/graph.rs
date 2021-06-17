use nrg_math::Vector4;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize};

use nrg_gui::{implement_widget_with_data, InternalWidget, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Graph {
    data: WidgetData,
}
implement_widget_with_data!(Graph);

impl InternalWidget for Graph {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }

        self.size(Screen::get_size())
            .selectable(false)
            .draggable(false)
            .style(WidgetStyle::Invisible);
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
}
