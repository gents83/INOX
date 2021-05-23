use std::path::Path;

use nrg_graphics::{MaterialInstance, TextureInstance};
use nrg_math::Vector2;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, Text, WidgetData, WidgetEvent,
    DEFAULT_TEXT_SIZE, DEFAULT_WIDGET_HEIGHT,
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

impl Icon {
    pub fn set_text(&mut self, string: &str) -> &mut Self {
        let uid = self.text;
        if let Some(text) = self.node_mut().get_child::<Text>(uid) {
            text.set_text(string);
        }
        self
    }
    pub fn create_icons(path: &str, parent_widget: &mut dyn Widget) {
        if let Ok(dir) = std::fs::read_dir(path) {
            dir.for_each(|entry| {
                if let Ok(dir_entry) = entry {
                    let path = dir_entry.path();
                    if !path.is_dir() {
                        let mut icon = Icon::new(
                            parent_widget.get_shared_data(),
                            parent_widget.get_global_messenger(),
                        );
                        icon.horizontal_alignment(HorizontalAlignment::Left)
                            .vertical_alignment(VerticalAlignment::Top);
                        icon.set_text(path.file_name().unwrap().to_str().unwrap());
                        parent_widget.add_child(Box::new(icon));
                    }
                }
            });
        }
    }
}

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
        let text_size: Vector2 = DEFAULT_TEXT_SIZE.into();
        text.editable(false)
            .size(text_size * Screen::get_scale_factor() * 0.5)
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
