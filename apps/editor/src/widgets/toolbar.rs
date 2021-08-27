use std::path::PathBuf;

use nrg_filesystem::convert_from_local_path;
use nrg_graphics::{TextureId, TextureInstance, TextureRc};
use nrg_messenger::{Message, MessageBox, MessengerRw};

use nrg_resources::{FileResource, SharedData, SharedDataRw, DATA_FOLDER};
use nrg_ui::{
    implement_widget_data, ImageButton, TextureId as eguiTextureId, TopBottomPanel, UIWidget,
    UIWidgetRc, Ui, Widget,
};

use crate::{EditMode, EditorEvent};

struct ToolbarData {
    shared_data: SharedDataRw,
    global_dispatcher: MessageBox,
    select_icon: TextureRc,
    move_icon: TextureRc,
    rotate_icon: TextureRc,
    scale_icon: TextureRc,
    mode: EditMode,
}
implement_widget_data!(ToolbarData);

pub struct Toolbar {
    ui_page: UIWidgetRc,
}

impl Toolbar {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        let select_icon = TextureInstance::create_from_file(
            shared_data,
            convert_from_local_path(
                PathBuf::from(DATA_FOLDER).as_path(),
                PathBuf::from("./icons/select.png").as_path(),
            )
            .as_path(),
        );
        let move_icon = TextureInstance::create_from_file(
            shared_data,
            convert_from_local_path(
                PathBuf::from(DATA_FOLDER).as_path(),
                PathBuf::from("./icons/move.png").as_path(),
            )
            .as_path(),
        );
        let rotate_icon = TextureInstance::create_from_file(
            shared_data,
            convert_from_local_path(
                PathBuf::from(DATA_FOLDER).as_path(),
                PathBuf::from("./icons/rotate.png").as_path(),
            )
            .as_path(),
        );
        let scale_icon = TextureInstance::create_from_file(
            shared_data,
            convert_from_local_path(
                PathBuf::from(DATA_FOLDER).as_path(),
                PathBuf::from("./icons/scale.png").as_path(),
            )
            .as_path(),
        );
        let data = ToolbarData {
            shared_data: shared_data.clone(),
            select_icon,
            move_icon,
            rotate_icon,
            scale_icon,
            mode: EditMode::View,
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
        };
        let ui_page = Self::create(shared_data, data);
        Self { ui_page }
    }

    fn create(shared_data: &SharedDataRw, data: ToolbarData) -> UIWidgetRc {
        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<ToolbarData>() {
                TopBottomPanel::top("toolbar")
                    .resizable(false)
                    .show(ui_context, |ui| {
                        ui.horizontal(|ui| {
                            let mode = data.mode;
                            if Self::show_icon(ui, &data.shared_data, data.select_icon.id()) {
                                data.mode = EditMode::Select;
                            }
                            if Self::show_icon(ui, &data.shared_data, data.move_icon.id()) {
                                data.mode = EditMode::Move;
                            }
                            if Self::show_icon(ui, &data.shared_data, data.rotate_icon.id()) {
                                data.mode = EditMode::Rotate;
                            }
                            if Self::show_icon(ui, &data.shared_data, data.scale_icon.id()) {
                                data.mode = EditMode::Scale;
                            }
                            if data.mode != mode {
                                data.global_dispatcher
                                    .write()
                                    .unwrap()
                                    .send(EditorEvent::ChangeMode(data.mode).as_boxed())
                                    .ok();
                            }
                        });
                    });
            }
        })
    }

    fn show_icon(ui: &mut Ui, shared_data: &SharedDataRw, texture_id: TextureId) -> bool {
        let textures = SharedData::get_resources_of_type::<TextureInstance>(shared_data);
        if let Some(index) = textures.iter().position(|t| t.id() == texture_id) {
            let response = ImageButton::new(eguiTextureId::User(index as _), [32., 32.]).ui(ui);
            return response.clicked();
        }
        false
    }
}
