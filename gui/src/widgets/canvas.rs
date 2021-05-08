use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Canvas {
    data: WidgetData,
}
implement_widget!(Canvas);

impl Canvas {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        let mut w = Self {
            data: WidgetData::new(shared_data),
        };
        w.init();
        w
    }
}

impl InternalWidget for Canvas {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }

        self.size(Screen::get_size())
            .selectable(false)
            .draggable(false)
            .style(WidgetStyle::DefaultCanvas);
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {}
}
