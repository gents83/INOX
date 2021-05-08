use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_SIZE};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Separator {
    data: WidgetData,
}
implement_widget!(Separator);

impl Separator {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        let mut w = Self {
            data: WidgetData::new(shared_data),
        };
        w.init();
        w
    }
}

impl InternalWidget for Separator {
    fn widget_init(&mut self) {
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

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {}
}
