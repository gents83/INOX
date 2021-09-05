use std::path::{Path, PathBuf};

use nrg_filesystem::convert_from_local_path;
use nrg_graphics::Texture;
use nrg_math::{Vector2, Vector4};
use nrg_messenger::Message;
use nrg_resources::{FileResource, Handle, ResourceData, DATA_FOLDER};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, Screen, Text, WidgetData,
    WidgetEvent, DEFAULT_TEXT_SIZE, DEFAULT_WIDGET_HEIGHT,
};

pub const DEFAULT_ICON_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 2., DEFAULT_WIDGET_HEIGHT * 2.];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Icon {
    data: WidgetData,
    image: Uid,
    text: Uid,
    #[serde(skip)]
    texture: Handle<Texture>,
}
implement_widget_with_custom_members!(Icon {
    image: INVALID_UID,
    text: INVALID_UID,
    texture: None
});

impl Icon {
    pub fn set_text(&mut self, string: &str) -> &mut Self {
        let uid = self.text;
        if let Some(text) = self.node().get_child_mut::<Text>(uid) {
            text.set_text(string);
        }
        self
    }
    pub fn collapsed(&mut self) -> &mut Self {
        self.fill_type(ContainerFillType::Vertical)
            .horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Top)
            .keep_fixed_height(false)
            .keep_fixed_width(false);
        let size: Vector2 = self.state().get_size();
        let image = self.image;
        if let Some(image) = self.node().get_child_mut::<Panel>(image) {
            image
                .size(size * 0.75 * Screen::get_scale_factor())
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Top);
        }
        let text = self.text;
        if let Some(text) = self.node().get_child_mut::<Text>(text) {
            let mut text_size: Vector2 = size * Screen::get_scale_factor();
            text_size.y *= 0.25;
            text.size(text_size)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Bottom);
        }
        self
    }
    pub fn expanded(&mut self) -> &mut Self {
        self.horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Top)
            .selectable(true)
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_width(false);
        let size: Vector2 = self.state().get_size();
        let image = self.image;
        if let Some(image) = self.node().get_child_mut::<Panel>(image) {
            image
                .size(size * 0.5 * Screen::get_scale_factor())
                .horizontal_alignment(HorizontalAlignment::Left)
                .vertical_alignment(VerticalAlignment::Center);
        }
        let text = self.text;
        if let Some(text) = self.node().get_child_mut::<Text>(text) {
            let mut text_size: Vector2 = size * Screen::get_scale_factor();
            text_size.y *= 0.25;
            text.size(text_size)
                .horizontal_alignment(HorizontalAlignment::Left)
                .vertical_alignment(VerticalAlignment::Center);
        }
        self
    }
    pub fn create_icons(path: &Path, parent_widget: &mut dyn Widget) {
        if let Ok(dir) = std::fs::read_dir(path) {
            dir.for_each(|entry| {
                if let Ok(dir_entry) = entry {
                    let path = dir_entry.path();
                    if !path.is_dir() {
                        let mut icon = Icon::new(
                            parent_widget.get_shared_data(),
                            parent_widget.get_global_messenger(),
                        );
                        icon.node_mut()
                            .set_name(path.file_name().unwrap().to_str().unwrap());
                        icon.expanded()
                            .set_text(path.file_name().unwrap().to_str().unwrap());
                        parent_widget.add_child(Box::new(icon));
                    }
                }
            });
        }
    }

    pub fn set_texture(&mut self, path: &Path) -> &mut Self {
        let image = self.image;
        if let Some(image) = self.node().get_child_mut::<Panel>(image) {
            let material = image.graphics().get_material();
            if let Some(texture) = &self.texture {
                material.get_mut().remove_texture(texture.id());
            }
            let texture_path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), path);
            let texture = Texture::create_from_file(self.get_shared_data(), texture_path.as_path());
            material.get_mut().add_texture(texture.clone());
            self.texture = Some(texture);
        }
        self
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
            .style(WidgetStyle::Default);

        let mut image = Panel::new(self.get_shared_data(), self.get_global_messenger());
        image
            .selectable(false)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Top)
            .size(size * 0.75 * Screen::get_scale_factor())
            .style(WidgetStyle::DefaultText);

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

        self.set_texture(PathBuf::from("icons/file.png").as_path());
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>();
    }
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
    fn widget_on_layout_changed(&mut self) {}
}
