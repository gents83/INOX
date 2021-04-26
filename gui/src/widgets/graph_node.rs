use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget, InternalWidget, TitleBar, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    data: WidgetData,
}
implement_widget!(GraphNode);

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
        }
    }
}

impl InternalWidget for GraphNode {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [200., 100.].into();

        self.position(Screen::get_center() - size / 2.)
            .size(size)
            .draggable(true)
            .style(WidgetStyle::DefaultBackground);

        let mut title_bar = TitleBar::default();
        title_bar.init(renderer);
        self.add_child(Box::new(title_bar));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
