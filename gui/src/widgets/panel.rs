use nrg_events::EventsRw;
use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_HEIGHT};

pub const DEFAULT_PANEL_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 10., DEFAULT_WIDGET_HEIGHT * 10.];

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

        let size: Vector2 = DEFAULT_PANEL_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .style(WidgetStyle::Invisible);
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
