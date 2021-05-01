use nrg_events::EventsRw;
use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget, EditableText, InternalWidget, Text, WidgetData, DEFAULT_WIDGET_HEIGHT,
    DEFAULT_WIDGET_WIDTH,
};
pub const DEFAULT_ICON_SIZE: [f32; 2] = [
    DEFAULT_WIDGET_WIDTH * 2. / 3.,
    DEFAULT_WIDGET_HEIGHT * 2. / 3.,
];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct TextBox {
    label: Uid,
    editable_text: Uid,
    data: WidgetData,
    is_editable: bool,
}
implement_widget!(TextBox);

impl Default for TextBox {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
            label: INVALID_UID,
            editable_text: INVALID_UID,
            is_editable: true,
        }
    }
}

impl TextBox {
    pub fn editable(&mut self, is_editable: bool) -> &mut Self {
        self.is_editable = is_editable;
        self
    }
    pub fn is_editable(&self) -> bool {
        self.is_editable
    }
    pub fn with_label(&mut self, text: &str) -> &mut Self {
        let uid = self.label;
        if let Some(label) = self.get_data_mut().node.get_child::<Text>(uid) {
            label.set_text(text);
        }
        self
    }
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        let uid = self.editable_text;
        if let Some(editable_text) = self.get_data_mut().node.get_child::<EditableText>(uid) {
            editable_text.set_text(text);
        }
        self
    }

    pub fn get_text(&mut self) -> String {
        let mut str = String::new();
        let uid = self.editable_text;
        if let Some(editable_text) = self.get_data_mut().node.get_child::<EditableText>(uid) {
            str = editable_text.get_text();
        }
        str
    }
}

impl InternalWidget for TextBox {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [400., 100.].into();

        self.position(Screen::get_center() - size / 2.)
            .size(size)
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_width(false)
            .keep_fixed_height(false)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .space_between_elements(10)
            .use_space_before_and_after(false)
            .draggable(false)
            .selectable(false)
            .style(WidgetStyle::Invisible);

        let mut label = Text::default();
        label.init(renderer);
        label
            .draggable(false)
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center);
        label.set_text("Label: ");

        self.label = self.add_child(Box::new(label));

        let mut editable_text = EditableText::default();
        editable_text.init(renderer);
        editable_text
            .draggable(false)
            .vertical_alignment(VerticalAlignment::Center);

        self.editable_text = self.add_child(Box::new(editable_text));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, _events_rw: &mut EventsRw) {}

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
