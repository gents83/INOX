use std::any::TypeId;

use nrg_gui::{
    implement_widget_with_custom_members, Checkbox, InternalWidget, Menu, WidgetData, WidgetEvent,
};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::Message;
use nrg_platform::WindowEvent;
use nrg_serialize::*;

use super::{DialogEvent, FolderDialog};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct MainMenu {
    data: WidgetData,
    menu: Option<Menu>,
    #[serde(skip)]
    file_id: Uid,
    #[serde(skip)]
    new_id: Uid,
    #[serde(skip)]
    open_id: Uid,
    #[serde(skip)]
    save_id: Uid,
    #[serde(skip)]
    exit_id: Uid,
    #[serde(skip)]
    settings_id: Uid,
    #[serde(skip)]
    show_history_id: Uid,
    #[serde(skip)]
    filename_dialog: Option<FolderDialog>,
}
implement_widget_with_custom_members!(MainMenu {
    menu: None,
    file_id: INVALID_UID,
    new_id: INVALID_UID,
    save_id: INVALID_UID,
    open_id: INVALID_UID,
    exit_id: INVALID_UID,
    settings_id: INVALID_UID,
    show_history_id: INVALID_UID,
    filename_dialog: None
});

impl MainMenu {
    pub fn menu(&self) -> &Menu {
        self.menu.as_ref().unwrap()
    }
    fn menu_mut(&mut self) -> &mut Menu {
        self.menu.as_mut().unwrap()
    }
    pub fn get_size(&self) -> Vector2 {
        self.menu().state().get_size()
    }
    pub fn show_history(&mut self) -> bool {
        let mut show = false;
        let settings_uid = self.settings_id;
        let uid = self.show_history_id;
        if let Some(submenu) = self.menu_mut().get_submenu(settings_uid) {
            if let Some(checkbox) = submenu.node_mut().get_child_mut::<Checkbox>(uid) {
                show = checkbox.is_checked();
            }
        }
        show
    }
    pub fn is_new_uid(&self, entry_uid: Uid) -> bool {
        self.new_id == entry_uid
    }
    pub fn is_open_uid(&self, entry_uid: Uid) -> bool {
        self.open_id == entry_uid
    }
    pub fn is_save_uid(&self, entry_uid: Uid) -> bool {
        self.save_id == entry_uid
    }
}

impl InternalWidget for MainMenu {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<DialogEvent>()
            .register_to_listen_event::<WidgetEvent>();

        self.menu = Some(Menu::new(
            self.get_shared_data(),
            self.get_global_messenger(),
        ));
        self.menu_mut().move_to_layer(0.5);

        let file_id = self.menu_mut().add_menu_item("File");
        self.file_id = file_id;
        self.new_id = self.menu_mut().add_submenu_entry_default(file_id, "New");
        self.open_id = self.menu_mut().add_submenu_entry_default(file_id, "Open");
        self.save_id = self.menu_mut().add_submenu_entry_default(file_id, "Save");
        self.exit_id = self.menu_mut().add_submenu_entry_default(file_id, "Exit");

        let settings_id = self.menu_mut().add_menu_item("Settings");
        self.settings_id = settings_id;
        let mut checkbox = Checkbox::new(self.get_shared_data(), self.get_global_messenger());
        checkbox.with_label("Show History").checked(false);
        self.show_history_id = self
            .menu_mut()
            .add_submenu_entry(settings_id, Box::new(checkbox));
    }

    fn widget_update(&mut self, drawing_area_in_px: Vector4) {
        self.menu_mut()
            .update(drawing_area_in_px, drawing_area_in_px);

        if let Some(dialog) = &mut self.filename_dialog {
            dialog.update(drawing_area_in_px, drawing_area_in_px);
        }
    }

    fn widget_uninit(&mut self) {
        if let Some(dialog) = &mut self.filename_dialog {
            dialog.uninit();
            self.filename_dialog = None;
        }
        self.menu_mut().uninit();

        self.unregister_to_listen_event::<DialogEvent>()
            .unregister_to_listen_event::<WidgetEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<DialogEvent>() {
            if let Some(dialog) = &mut self.filename_dialog {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match event {
                    DialogEvent::Confirmed(widget_id, _requester_uid, _text) => {
                        if *widget_id == dialog.id() {
                            dialog.uninit();
                            self.filename_dialog = None;
                        }
                    }
                    DialogEvent::Canceled(widget_id) => {
                        if *widget_id == dialog.id() {
                            dialog.uninit();
                            self.filename_dialog = None;
                        }
                    }
                }
            }
        } else if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = *event {
                if self.new_id == widget_id && self.filename_dialog.is_none() {
                    self.filename_dialog = Some(FolderDialog::new(
                        self.get_shared_data(),
                        self.get_global_messenger(),
                    ));
                    let dialog = self.filename_dialog.as_mut().unwrap();
                    dialog.set_requester_uid(self.new_id);
                    dialog.set_title("New Widget");
                } else if self.open_id == widget_id && self.filename_dialog.is_none() {
                    self.filename_dialog = Some(FolderDialog::new(
                        self.get_shared_data(),
                        self.get_global_messenger(),
                    ));
                    let dialog = self.filename_dialog.as_mut().unwrap();
                    dialog.set_requester_uid(self.open_id);
                    dialog.set_title("Open Widget");
                } else if self.save_id == widget_id && self.filename_dialog.is_none() {
                    self.filename_dialog = Some(FolderDialog::new(
                        self.get_shared_data(),
                        self.get_global_messenger(),
                    ));
                    let dialog = self.filename_dialog.as_mut().unwrap();
                    dialog.set_requester_uid(self.save_id);
                    dialog.set_title("Save Widget");
                } else if self.exit_id == widget_id {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(WindowEvent::Close.as_boxed())
                        .ok();
                }
            }
        }
    }
}
