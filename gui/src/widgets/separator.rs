use nrg_graphics::Renderer;
use nrg_math::Vector4u;
use nrg_platform::EventsRw;
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
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size([DEFAULT_WIDGET_SIZE.x, 1].into())
            .stroke(1)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .selectable(false)
            .style(WidgetStyle::FullActive)
            .border_style(WidgetStyle::FullActive);
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
