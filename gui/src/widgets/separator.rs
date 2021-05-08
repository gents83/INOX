use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_SIZE};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Separator {
    data: WidgetData,
}
implement_widget!(Separator);

impl Default for Separator {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
        }
    }
}

impl InternalWidget for Separator {
    fn widget_init(&mut self, _shared_data: &SharedDataRw) {
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size([DEFAULT_WIDGET_SIZE[0], 1. * Screen::get_scale_factor()].into())
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Top)
            .selectable(false)
            .style(WidgetStyle::FullActive);
    }

    fn widget_update(&mut self, _shared_data: &SharedDataRw) {}

    fn widget_uninit(&mut self, _shared_data: &SharedDataRw) {}
}
