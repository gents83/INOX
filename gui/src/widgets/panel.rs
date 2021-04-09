use nrg_graphics::Renderer;
use nrg_math::Vector2u;
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_SIZE};

pub const DEFAULT_PANEL_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 10,
    y: DEFAULT_WIDGET_SIZE.y * 10,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Panel {
    data: WidgetData,
}
implement_widget!(Panel);

impl Default for Panel {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
        }
    }
}

impl InternalWidget for Panel {
    fn widget_init(&mut self, _renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        self.size(DEFAULT_PANEL_SIZE)
            .selectable(false)
            .style(WidgetStyle::DefaultBackground)
            .border_style(WidgetStyle::DefaultBorder);
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
