use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

const DEFAULT_MENU_SIZE: Vector2u = Vector2u { x: 200, y: 20 };
const DEFAULT_MENU_ITEM_SIZE: Vector2u = Vector2u { x: 100, y: 20 };
const DEFAULT_SUBMENU_ITEM_SIZE: Vector2u = Vector2u { x: 300, y: 500 };

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
struct MenuItemPanel {
    menu_item_id: UID,
    submenu: Menu,
    opened: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Menu {
    #[serde(skip)]
    container: ContainerData,
    data: WidgetData,
    entries: Vec<MenuItemPanel>,
}
implement_widget!(Menu);
implement_container!(Menu);

impl Default for Menu {
    fn default() -> Self {
        Self {
            container: ContainerData::default(),
            data: WidgetData::default(),
            entries: Vec::new(),
        }
    }
}

impl Menu {
    pub fn add_menu_item(&mut self, renderer: &mut Renderer, label: &str) -> UID {
        let mut button = Button::default();
        button.init(renderer);
        button
            .size(DEFAULT_MENU_ITEM_SIZE * Screen::get_scale_factor())
            .vertical_alignment(VerticalAlignment::Center)
            .set_text(label);
        button
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::DefaultBackground);

        let mut submenu = Menu::default();
        submenu.init(renderer);
        submenu
            .position(
                [
                    button.get_data().state.get_position().x,
                    self.get_data().state.get_size().y,
                ]
                .into(),
            )
            .size(DEFAULT_SUBMENU_ITEM_SIZE * Screen::get_scale_factor())
            .selectable(false)
            .visible(false)
            .vertical_alignment(VerticalAlignment::None)
            .horizontal_alignment(HorizontalAlignment::None)
            .fill_type(ContainerFillType::Vertical)
            .fit_to_content(true)
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::DefaultBackground);

        let menu_item_id = self.add_child(Box::new(button));
        self.entries.push(MenuItemPanel {
            menu_item_id,
            submenu,
            opened: false,
        });
        menu_item_id
    }

    pub fn add_submenu_entry_for(
        &mut self,
        renderer: &mut Renderer,
        menu_item_id: UID,
        label: &str,
    ) -> UID {
        let mut id = INVALID_ID;
        if let Some(index) = self
            .entries
            .iter()
            .position(|el| el.menu_item_id == menu_item_id)
        {
            let mut button = Button::default();
            button.init(renderer);
            button
                .size(DEFAULT_MENU_ITEM_SIZE * Screen::get_scale_factor())
                .vertical_alignment(VerticalAlignment::None)
                .set_text(label);
            button
                .get_data_mut()
                .graphics
                .set_style(WidgetStyle::DefaultBackground);

            let entry = &mut self.entries[index];
            id = entry.submenu.add_child(Box::new(button));
        }
        id
    }

    fn manage_menu_interactions(&mut self, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Released(widget_id) = event {
                    self.entries.iter_mut().for_each(|e| {
                        if e.submenu.id() == *widget_id {
                            e.opened = !e.opened;
                            e.submenu.visible(e.opened);
                        }
                    });
                }
            }
        }
    }
}

impl InternalWidget for Menu {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        self.size(DEFAULT_MENU_SIZE * Screen::get_scale_factor())
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .fit_to_content(false);

        let data = self.get_data_mut();
        data.graphics.set_style(WidgetStyle::DefaultBackground);
    }

    fn widget_update(
        &mut self,
        drawing_area_in_px: Vector4u,
        renderer: &mut Renderer,
        events_rw: &mut EventsRw,
        input_handler: &InputHandler,
    ) {
        self.entries.iter_mut().for_each(|e| {
            e.submenu
                .update(drawing_area_in_px, renderer, events_rw, input_handler);
        });
        self.apply_fit_to_content();

        self.manage_menu_interactions(events_rw);

        let data = self.get_data_mut();
        let pos = Screen::convert_from_pixels_into_screen_space(data.state.get_position());
        let size = Screen::convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
