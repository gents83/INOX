use std::path::Path;

use nrg_graphics::{MaterialInstance, TextureInstance};
use nrg_math::Vector2;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, Text, WidgetData, WidgetEvent,
    DEFAULT_WIDGET_HEIGHT,
};

pub const DEFAULT_ICON_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 4., DEFAULT_WIDGET_HEIGHT * 4.];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Icon {
    data: WidgetData,
    image: Uid,
    text: Uid,
}
implement_widget_with_custom_members!(Icon {
    image: INVALID_UID,
    text: INVALID_UID
});

impl InternalWidget for Icon {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_ICON_SIZE.into();
        self.position(Screen::get_center() - size / 2.)
            .size(size * Screen::get_scale_factor())
            .selectable(false)
            .fill_type(ContainerFillType::Vertical)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .keep_fixed_height(true)
            .keep_fixed_width(true)
            .space_between_elements((4. * Screen::get_scale_factor()) as _)
            .style(WidgetStyle::Invisible);

        let mut image = Panel::new(self.get_shared_data(), self.get_global_messenger());
        image
            .selectable(true)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Top)
            .size(size * 0.75 * Screen::get_scale_factor())
            .style(WidgetStyle::FullActive);

        let texture_uid = TextureInstance::create_from_path(
            self.get_shared_data(),
            &Path::new("./data/icons/file.png"),
        );
        let material_uid = image.graphics().get_material_id();
        MaterialInstance::add_texture(self.get_shared_data(), material_uid, texture_uid);

        self.image = self.add_child(Box::new(image));

        let mut text = Text::new(self.get_shared_data(), self.get_global_messenger());
        text.editable(false)
            .selectable(false)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Bottom)
            .set_text("File");
        self.text = self.add_child(Box::new(text));
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>();
    }
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
}
