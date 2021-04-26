use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{implement_widget, InternalWidget, Panel, Text, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    title_widget: Uid,
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
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [200., 100.].into();

        self.position(Screen::get_center() - size / 2.)
            .size(size)
            .draggable(true)
            .style(WidgetStyle::DefaultBackground);

        let mut title_bar = Panel::default();
        title_bar.init(renderer);
        title_bar
            .position(Screen::get_center() - size / 2.)
            .size(size)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(true)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Center)
            .draggable(false)
            .style(WidgetStyle::DefaultTitleBar);

        let mut title = Text::default();
        title.init(renderer);
        title
            .draggable(false)
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center);
        title.set_text("Title");
        self.title_widget = title_bar.add_child(Box::new(title));

        self.add_child(Box::new(title_bar));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
