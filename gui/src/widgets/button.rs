use std::path::{Path, PathBuf};

use nrg_graphics::{MaterialInstance, TextureInstance};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::Message;
use nrg_platform::MouseEvent;
use nrg_resources::{get_absolute_path_from, DATA_FOLDER};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Text, WidgetData, WidgetEvent,
    DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_SIZE, DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_BUTTON_WIDTH: f32 = DEFAULT_WIDGET_WIDTH * 20.;
pub const DEFAULT_BUTTON_SIZE: [f32; 2] = [DEFAULT_BUTTON_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Button {
    data: WidgetData,
    label_id: Uid,
}
implement_widget_with_custom_members!(Button {
    label_id: INVALID_UID
});

impl Button {
    pub fn with_text(&mut self, text: &str) -> &mut Self {
        let label_id = self.label_id;
        if let Some(label) = self.node().get_child_mut::<Text>(label_id) {
            label.set_text(text);
        }
        self
    }
    pub fn with_texture(&mut self, texture_path: &Path) -> &mut Self {
        let material_uid = self.graphics().get_material_id();
        let texture_path =
            get_absolute_path_from(PathBuf::from(DATA_FOLDER).as_path(), texture_path);
        let texture_uid =
            TextureInstance::create_from_path(self.get_shared_data(), texture_path.as_path());
        MaterialInstance::add_texture(self.get_shared_data(), material_uid, texture_uid);
        self
    }

    pub fn text_alignment(
        &mut self,
        vertical_alignment: VerticalAlignment,
        horizontal_alignment: HorizontalAlignment,
    ) -> &mut Self {
        let label_id = self.label_id;
        if let Some(label) = self.node().get_child_mut::<Text>(label_id) {
            label
                .vertical_alignment(vertical_alignment)
                .horizontal_alignment(horizontal_alignment);
        }
        self
    }
}

impl InternalWidget for Button {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

        if self.is_initialized() {
            return;
        }
        let size: Vector2 = DEFAULT_BUTTON_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements((DEFAULT_WIDGET_SIZE[0] / 5. * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(true)
            .keep_fixed_width(false)
            .style(WidgetStyle::DefaultButton);

        let mut text = Text::new(self.get_shared_data(), self.get_global_messenger());
        text.vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("Button Text");
        self.label_id = self.add_child(Box::new(text));
    }
    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>();
    }
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
}
