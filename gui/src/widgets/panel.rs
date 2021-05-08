use nrg_math::Vector2;
use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_HEIGHT};

pub const DEFAULT_PANEL_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 10., DEFAULT_WIDGET_HEIGHT * 10.];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Panel {
    data: WidgetData,
}
implement_widget!(Panel);

impl Panel {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        let mut w = Self {
            data: WidgetData::new(shared_data),
        };
        w.init();
        w
    }
}

impl InternalWidget for Panel {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_PANEL_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .style(WidgetStyle::Invisible);
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {}
}
