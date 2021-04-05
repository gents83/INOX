use nrg_graphics::*;
use nrg_gui::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct MainMenu {
    menu: Menu,
    file_id: UID,
    settings_id: UID,
    show_history_id: UID,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            file_id: INVALID_UID,
            settings_id: INVALID_UID,
            show_history_id: INVALID_UID,
        }
    }
}

impl MainMenu {
    pub fn get_size(&self) -> Vector2u {
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

    pub fn init(&mut self, renderer: &mut Renderer) {
        self.menu.init(renderer);

        self.file_id = self.menu.add_menu_item(renderer, "File");
        self.menu
            .add_submenu_entry_default(renderer, self.file_id, "New");
        self.menu
            .add_submenu_entry_default(renderer, self.file_id, "Exit");

        self.settings_id = self.menu.add_menu_item(renderer, "Settings");
        let mut checkbox = Checkbox::default();
        checkbox.init(renderer);
        checkbox.with_label(renderer, "Show History").checked(false);
        self.show_history_id = checkbox.id();
        self.menu
            .add_submenu_entry(self.settings_id, Box::new(checkbox));
    }

    pub fn update(
        &mut self,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        input_handler: &InputHandler,
    ) {
        self.menu
            .update(Screen::get_draw_area(), renderer, events, input_handler);
    }
}
