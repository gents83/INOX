use nrg_gui::{
    implement_widget_with_custom_members, Checkbox, InternalWidget, Menu, WidgetData, WidgetEvent,
};
use nrg_math::Vector2;
use nrg_platform::WindowEvent;
use nrg_serialize::*;

use super::{DialogResult, FilenameDialog};

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
    exit_id: Uid,
    #[serde(skip)]
    settings_id: Uid,
    #[serde(skip)]
    show_history_id: Uid,
    #[serde(skip)]
    filename_dialog: Option<FilenameDialog>,
}
implement_widget_with_custom_members!(MainMenu {
    menu: None,
    file_id: INVALID_UID,
    new_id: INVALID_UID,
    open_id: INVALID_UID,
    exit_id: INVALID_UID,
    settings_id: INVALID_UID,
    show_history_id: INVALID_UID,
    filename_dialog: None
});

impl MainMenu {
    fn menu(&self) -> &Menu {
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
            if let Some(checkbox) = submenu.node_mut().get_child::<Checkbox>(uid) {
                show = checkbox.is_checked();
            }
        }
        show
    }
    fn manage_events(&mut self) -> bool {
        let mut need_init = false;
        {
            let events = self.get_events().read().unwrap();
            if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
                for event in widget_events.iter() {
                    if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
                        if self.new_id == *widget_id && self.filename_dialog.is_none() {
                            need_init = true;
                        } else if self.exit_id == *widget_id {
                            return true;
                        }
                    }
                }
            }
        }
        if need_init {
            self.filename_dialog = Some(FilenameDialog::new(
                self.get_shared_data(),
                self.get_events(),
            ));
        }
        false
    }
}

impl InternalWidget for MainMenu {
    fn widget_init(&mut self) {
        self.menu = Some(Menu::new(self.get_shared_data(), self.get_events()));
        self.menu_mut().move_to_layer(0.5);

        let file_id = self.menu_mut().add_menu_item("File");
        self.file_id = file_id;
        self.new_id = self.menu_mut().add_submenu_entry_default(file_id, "New");
        self.open_id = self.menu_mut().add_submenu_entry_default(file_id, "Open");
        self.exit_id = self.menu_mut().add_submenu_entry_default(file_id, "Exit");

        let settings_id = self.menu_mut().add_menu_item("Settings");
        self.settings_id = settings_id;
        let mut checkbox = Checkbox::new(self.get_shared_data(), self.get_events());
        checkbox.with_label("Show History").checked(false);
        self.show_history_id = self
            .menu_mut()
            .add_submenu_entry(settings_id, Box::new(checkbox));
    }

    fn widget_update(&mut self) {
        self.menu_mut().update(Screen::get_draw_area());

        let should_exit = self.manage_events();
        if should_exit {
            let mut events = self.get_events().write().unwrap();
            events.send_event(WindowEvent::Close);
        }

        if let Some(dialog) = &mut self.filename_dialog {
            if dialog.get_result() == DialogResult::Ok {
                let filename = dialog.get_filename();
                println!("Filename = {}", filename);
                dialog.uninit();
                self.filename_dialog = None;
            } else if dialog.get_result() == DialogResult::Cancel {
                dialog.uninit();
                self.filename_dialog = None;
            } else {
                dialog.update(Screen::get_draw_area());
            }
        }
    }

    fn widget_uninit(&mut self) {
        if let Some(dialog) = &mut self.filename_dialog {
            dialog.uninit();
            self.filename_dialog = None;
        }
        self.menu_mut().uninit();
    }
}
