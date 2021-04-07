use nrg_graphics::Renderer;
use nrg_math::{Vector2u, Vector4u};
use nrg_platform::{EventsRw, InputHandler};
use nrg_serialize::{Deserialize, Serialize, INVALID_UID, UID};

use crate::{implement_widget, InternalWidget, Text, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    title_widget: UID,
    data: WidgetData,
}
implement_widget!(GraphNode);

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
            title_widget: INVALID_UID,
        }
    }
}

impl InternalWidget for GraphNode {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        let size: Vector2u = [200, 100].into();

        self.position(Screen::get_center() - size / 2)
            .size(size)
            .draggable(true)
            .style(WidgetStyle::DefaultBackground);

        let mut title = Text::default();
        title.init(renderer);
        title
            .draggable(false)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Center);
        title.set_text("Title");
        self.title_widget = self.add_child(Box::new(title));
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
