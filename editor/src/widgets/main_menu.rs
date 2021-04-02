use nrg_graphics::*;
use nrg_gui::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct MainMenu {
    menu: Menu,
    file_id: UID,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            file_id: INVALID_ID,
        }
    }
}

impl MainMenu {
    pub fn get_size(&self) -> Vector2u {
        self.menu.get_data().state.get_size()
    }

    pub fn init(&mut self, renderer: &mut Renderer) {
        self.menu.init(renderer);

        self.file_id = self.menu.add_menu_item(renderer, "File");
        self.menu
            .add_submenu_entry_for(renderer, self.file_id, "New");
        self.menu
            .add_submenu_entry_for(renderer, self.file_id, "Exit");
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
