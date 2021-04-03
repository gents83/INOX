use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

const DEFAULT_MENU_LAYER: f32 = 0.5;
const DEFAULT_MENU_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 10,
    y: DEFAULT_WIDGET_SIZE.y,
};
const DEFAULT_MENU_ITEM_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 10,
    y: DEFAULT_WIDGET_SIZE.y,
};
const DEFAULT_SUBMENU_ITEM_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 30,
    y: DEFAULT_WIDGET_SIZE.y * 50,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
struct MenuItemPanel {
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
    entries_uid: Vec<UID>,
}
implement_widget!(Menu);
implement_container!(Menu);

impl Default for Menu {
    fn default() -> Self {
        Self {
            container: ContainerData::default(),
            data: WidgetData::default(),
            entries: Vec::new(),
            entries_uid: Vec::new(),
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
            .set_text(label)
            .set_text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left);
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

        submenu.move_to_layer(DEFAULT_MENU_LAYER);

        let menu_item_id = self.add_child(Box::new(button));
        self.entries_uid.push(menu_item_id);
        self.entries.push(MenuItemPanel {
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
        if let Some(index) = self.entries_uid.iter().position(|el| *el == menu_item_id) {
            let mut button = Button::default();
            button.init(renderer);
            button
                .size(DEFAULT_MENU_ITEM_SIZE * Screen::get_scale_factor())
                .vertical_alignment(VerticalAlignment::None)
                .set_text(label)
                .set_text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left);
            button
                .get_data_mut()
                .graphics
                .set_style(WidgetStyle::DefaultBackground);

            let entry = &mut self.entries[index];
            id = entry.submenu.add_child(Box::new(button));
        }
        id
    }
    fn is_submenu_opened(&self) -> bool {
        let mut is_opened = false;
        self.entries.iter().for_each(|e| {
            is_opened |= e.opened;
        });
        is_opened
    }
    fn is_hovering_entry(&mut self, entry_uid: UID) -> bool {
        let mut is_hover = false;
        if let Some(widget) = self.get_data_mut().node.get_child::<Button>(entry_uid) {
            if widget.is_hover() {
                is_hover = true;
            }
        }
        if !is_hover {
            if let Some(index) = self.entries_uid.iter().position(|el| *el == entry_uid) {
                let item = &self.entries[index];
                if item.opened && item.submenu.is_hover_recursive() {
                    is_hover = true;
                }
            }
        }
        is_hover
    }
    fn manage_hovering(&mut self) {
        if self.is_submenu_opened() {
            let count = self.entries.len();
            for i in 0..count {
                if self.entries[i].opened {
                    let entry_uid = self.entries_uid[i];
                    if !self.is_hovering_entry(entry_uid) {
                        let item = &mut self.entries[i];
                        item.opened = false;
                        item.submenu.visible(item.opened);
                    }
                }
            }
        }
    }

    fn manage_menu_interactions(&mut self, events_rw: &mut EventsRw) {
        self.manage_hovering();

        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Released(widget_id) = event {
                    let mut pos = Vector2u::default();
                    if let Some(button) = self.get_data_mut().node.get_child::<Button>(*widget_id) {
                        pos.x = button.get_data().state.get_position().x;
                        pos.y = self.get_data().state.get_size().y;
                    }
                    if let Some(index) = self.entries_uid.iter().position(|el| *el == *widget_id) {
                        let item = &mut self.entries[index];
                        item.opened = !item.opened;
                        item.submenu.position(pos).visible(item.opened);
                    }
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
            .space_between_elements(20)
            .fill_type(ContainerFillType::Horizontal)
            .use_space_before_and_after(false)
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
