use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{implement_widget, InternalWidget, Text, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct TitleBar {
    title_widget: Uid,
    data: WidgetData,
}
implement_widget!(TitleBar);

impl Default for TitleBar {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
            title_widget: INVALID_UID,
        }
    }
}

impl InternalWidget for TitleBar {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [400., 100.].into();

        self.position(Screen::get_center() - size / 2.)
            .size(size)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(true)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Stretch)
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
        self.title_widget = self.add_child(Box::new(title));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
