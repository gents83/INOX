use nrg_math::{Vector2, Vector4};
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize};

use nrg_gui::{
    implement_widget_with_data, InternalWidget, Text, WidgetData, DEFAULT_WIDGET_HEIGHT,
    DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_PROPERTIES_SIZE: [f32; 2] =
    [DEFAULT_WIDGET_WIDTH * 10., DEFAULT_WIDGET_HEIGHT * 10.];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Properties {
    data: WidgetData,
}
implement_widget_with_data!(Properties);

impl Properties {
    fn add_label(&mut self) -> &mut Self {
        let mut label = Text::new(&self.get_shared_data(), &self.get_global_messenger());
        label.editable(false).set_text("Properties:");
        self.add_child(Box::new(label));

        self
    }
}

impl InternalWidget for Properties {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }
        let size: Vector2 = DEFAULT_PROPERTIES_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .draggable(false)
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Bottom)
            .style(WidgetStyle::Default);

        self.add_label();
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
    fn widget_on_layout_changed(&mut self) {}
}
