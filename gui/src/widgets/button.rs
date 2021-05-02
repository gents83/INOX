use nrg_events::EventsRw;
use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget, InternalWidget, Text, WidgetData, DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_SIZE,
    DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_BUTTON_WIDTH: f32 = DEFAULT_WIDGET_WIDTH * 20.;
pub const DEFAULT_BUTTON_SIZE: [f32; 2] = [DEFAULT_BUTTON_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Button {
    data: WidgetData,
    label_id: Uid,
}
implement_widget!(Button);

impl Default for Button {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
            label_id: INVALID_UID,
        }
    }
}

impl Button {
    pub fn with_text(&mut self, text: &str) -> &mut Self {
        let label_id = self.label_id;
        if let Some(label) = self.get_data_mut().node.get_child::<Text>(label_id) {
            label.set_text(text);
        }
        self
    }

    pub fn text_alignment(
        &mut self,
        vertical_alignment: VerticalAlignment,
        horizontal_alignment: HorizontalAlignment,
    ) -> &mut Self {
        let label_id = self.label_id;
        if let Some(label) = self.get_data_mut().node.get_child::<Text>(label_id) {
            label
                .vertical_alignment(vertical_alignment)
                .horizontal_alignment(horizontal_alignment);
        }
        self
    }
}

impl InternalWidget for Button {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }
        let size: Vector2 = DEFAULT_BUTTON_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements((DEFAULT_WIDGET_SIZE[0] / 5. * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(true)
            .keep_fixed_width(false);

        let mut text = Text::default();
        text.init(renderer);
        text.vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("Button Text");
        self.label_id = self.add_child(Box::new(text));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
