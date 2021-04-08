use nrg_graphics::Renderer;
use nrg_math::Vector4u;
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Canvas {
    data: WidgetData,
}
implement_widget!(Canvas);

impl Default for Canvas {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
        }
    }
}

impl InternalWidget for Canvas {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        self.size(Screen::get_size())
            .selectable(false)
            .draggable(false)
            .style(WidgetStyle::DefaultCanvas);
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
    ) {
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
