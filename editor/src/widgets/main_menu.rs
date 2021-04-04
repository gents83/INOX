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
    show_history: bool,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            file_id: INVALID_UID,
            settings_id: INVALID_UID,
            show_history_id: INVALID_UID,
            show_history: false,
        }
    }
}

impl MainMenu {
    pub fn get_size(&self) -> Vector2u {
        self.menu.get_data().state.get_size()
    }
    pub fn show_history(&self) -> bool {
        self.show_history
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
        checkbox
            .horizontal_alignment(HorizontalAlignment::Left)
            .with_label(renderer, "Show History");
        self.show_history_id = checkbox.id();
        self.menu
            .add_submenu_entry(self.settings_id, Box::new(checkbox));
    }

    fn manage_events(&mut self, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<CheckboxEvent>() {
            for event in widget_events.iter() {
                if let CheckboxEvent::Checked(widget_id) = event {
                    if *widget_id == self.show_history_id {
                        self.show_history = true;
                    }
                } else if let CheckboxEvent::Unchecked(widget_id) = event {
                    if *widget_id == self.show_history_id {
                        self.show_history = false;
                    }
                }
            }
        }
    }

    pub fn update(
        &mut self,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        input_handler: &InputHandler,
    ) {
        self.menu
            .update(Screen::get_draw_area(), renderer, events, input_handler);

        self.manage_events(events);
    }
}
