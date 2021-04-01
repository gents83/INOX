use nrg_graphics::*;
use nrg_gui::*;
use nrg_math::*;
use nrg_platform::*;

pub struct MainMenu {
    menu: Menu,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
        }
    }
}

impl MainMenu {
    pub fn get_size(&self) -> Vector2u {
        self.menu.get_data().state.get_size()
    }

    pub fn init(&mut self, renderer: &mut Renderer) {
        self.menu.init(renderer);

        self.menu.add_menu_item(renderer, "File");
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
