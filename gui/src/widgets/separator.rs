use nrg_math::Vector4;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget_with_data, InternalWidget, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Separator {
    data: WidgetData,
}
implement_widget_with_data!(Separator);

impl InternalWidget for Separator {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size(
                [
                    1. * Screen::get_scale_factor(),
                    1. * Screen::get_scale_factor(),
                ]
                .into(),
            )
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Top)
            .selectable(false)
            .style(WidgetStyle::DefaultText);
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
    fn widget_on_layout_changed(&mut self) {}
}
