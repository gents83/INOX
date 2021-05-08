use nrg_events::EventsRw;
use nrg_gui::{implement_widget, *};
use nrg_math::*;
use nrg_platform::*;
use nrg_resources::SharedDataRw;
use nrg_serialize::*;

use super::{DialogResult, FilenameDialog};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct MainMenu {
    data: WidgetData,
    menu: Menu,
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
implement_widget!(MainMenu);

impl MainMenu {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        let mut w = Self {
            data: WidgetData::new(shared_data),
            menu: Menu::new(shared_data),
            file_id: INVALID_UID,
            new_id: INVALID_UID,
            open_id: INVALID_UID,
            exit_id: INVALID_UID,
            settings_id: INVALID_UID,
            show_history_id: INVALID_UID,
            filename_dialog: None,
        };
        w.init();
        w
    }
}

impl MainMenu {
    pub fn get_size(&self) -> Vector2 {
        self.menu.get_data().state.get_size()
    }
    pub fn show_history(&mut self) -> bool {
        let mut show = false;
        let settings_uid = self.settings_id;
        if let Some(submenu) = self.menu.get_submenu(settings_uid) {
            let uid = self.show_history_id;
            if let Some(checkbox) = submenu.get_data_mut().node.get_child::<Checkbox>(uid) {
                show = checkbox.is_checked();
            }
        }
        show
    }
    fn manage_events(&mut self) -> bool {
        let mut need_init = false;
        {
            let read_data = self.get_shared_data().read().unwrap();
            let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
            let events = events_rw.read().unwrap();
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
            self.filename_dialog = Some(FilenameDialog::new(self.get_shared_data()));
        }
        false
    }
}

impl InternalWidget for MainMenu {
    fn widget_init(&mut self) {
        self.menu.move_to_layer(0.5);

        self.file_id = self.menu.add_menu_item("File");
        self.new_id = self.menu.add_submenu_entry_default(self.file_id, "New");
        self.open_id = self.menu.add_submenu_entry_default(self.file_id, "Open");
        self.exit_id = self.menu.add_submenu_entry_default(self.file_id, "Exit");

        self.settings_id = self.menu.add_menu_item("Settings");
        let mut checkbox = Checkbox::new(self.get_shared_data());
        checkbox.with_label("Show History").checked(false);
        self.show_history_id = self
            .menu
            .add_submenu_entry(self.settings_id, Box::new(checkbox));
    }

    fn widget_update(&mut self) {
        self.menu.update(Screen::get_draw_area());

        let should_exit = self.manage_events();
        if should_exit {
            let read_data = self.get_shared_data().read().unwrap();
            let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
            let mut events = events_rw.write().unwrap();
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
        self.menu.uninit();
    }
}
