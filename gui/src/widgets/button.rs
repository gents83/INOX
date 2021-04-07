use nrg_graphics::Renderer;
use nrg_math::{Vector2u, Vector4u};
use nrg_platform::{EventsRw, InputHandler};
use nrg_serialize::{Deserialize, Serialize, INVALID_UID, UID};

use crate::{implement_widget, InternalWidget, Text, WidgetData, DEFAULT_WIDGET_SIZE};

pub const DEFAULT_BUTTON_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 20,
    y: DEFAULT_WIDGET_SIZE.y,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Button {
    data: WidgetData,
    label_id: UID,
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
        let data = self.get_data_mut();
        data.graphics.init(renderer, "UI");

        if self.is_initialized() {
            return;
        }

        self.size(DEFAULT_BUTTON_SIZE * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements(DEFAULT_WIDGET_SIZE.x / 5 * Screen::get_scale_factor() as u32)
            .use_space_before_and_after(true);

        let mut text = Text::default();
        text.init(renderer);
        text.vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("Button Text");
        self.label_id = self.add_child(Box::new(text));
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
