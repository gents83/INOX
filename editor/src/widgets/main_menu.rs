use nrg_graphics::*;
use nrg_gui::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;
use nrg_events::EventsRw;

use super::{DialogResult, FilenameDialog};

pub struct MainMenu {
    menu: Menu,
    file_id: Uid,
    new_id: Uid,
    open_id: Uid,
    exit_id: Uid,
    settings_id: Uid,
    show_history_id: Uid,
    filename_dialog: Option<FilenameDialog>,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            file_id: INVALID_UID,
            new_id: INVALID_UID,
            open_id: INVALID_UID,
            exit_id: INVALID_UID,
            settings_id: INVALID_UID,
            show_history_id: INVALID_UID,
            filename_dialog: None,
        }
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
    fn manage_events(&mut self, events_rw: &mut EventsRw, renderer: &mut Renderer) -> bool {
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
                    if self.new_id == *widget_id && self.filename_dialog.is_none() {
                        let mut dialog = FilenameDialog::default();
                        dialog.init(renderer);
                        self.filename_dialog = Some(dialog);
                    } else if self.exit_id == *widget_id {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn init(&mut self, renderer: &mut Renderer) {
        self.menu.init(renderer);
        self.menu.move_to_layer(0.5);

        self.file_id = self.menu.add_menu_item(renderer, "File");
        self.new_id = self
            .menu
            .add_submenu_entry_default(renderer, self.file_id, "New");
        self.open_id = self
            .menu
            .add_submenu_entry_default(renderer, self.file_id, "Open");
        self.exit_id = self
            .menu
            .add_submenu_entry_default(renderer, self.file_id, "Exit");

        self.settings_id = self.menu.add_menu_item(renderer, "Settings");
        let mut checkbox = Checkbox::default();
        checkbox.init(renderer);
        checkbox.with_label(renderer, "Show History").checked(false);
        self.show_history_id = self
            .menu
            .add_submenu_entry(self.settings_id, Box::new(checkbox));
    }

    pub fn update(&mut self, renderer: &mut Renderer, events_rw: &mut EventsRw) {
        self.menu
            .update(Screen::get_draw_area(), renderer, events_rw);

        let should_exit = self.manage_events(events_rw, renderer);
        if should_exit {
            let mut events = events_rw.write().unwrap();
            events.send_event(WindowEvent::Close);
        }

        if let Some(dialog) = &mut self.filename_dialog {
            if dialog.get_result() == DialogResult::Ok {
                let filename = dialog.get_filename();
                println!("Filename = {}", filename);
                dialog.uninit(renderer);
                self.filename_dialog = None;
            } else if dialog.get_result() == DialogResult::Cancel {
                dialog.uninit(renderer);
                self.filename_dialog = None;
            } else {
                dialog.update(renderer, events_rw);
            }
        }
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) {
        if let Some(dialog) = &mut self.filename_dialog {
            dialog.uninit(renderer);
            self.filename_dialog = None;
        }
        self.menu.uninit(renderer);
    }
}
