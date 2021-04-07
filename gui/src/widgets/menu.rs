use nrg_graphics::Renderer;
use nrg_math::{Vector2u, Vector4u};
use nrg_platform::{EventsRw, InputHandler};
use nrg_serialize::{Deserialize, Serialize, INVALID_UID, UID};

use crate::{
    implement_widget, Button, InternalWidget, WidgetData, WidgetEvent, DEFAULT_BUTTON_SIZE,
    DEFAULT_WIDGET_SIZE,
};

const DEFAULT_MENU_LAYER: f32 = 0.5;
const DEFAULT_MENU_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 10,
    y: DEFAULT_WIDGET_SIZE.y * 3 / 2,
};
const DEFAULT_MENU_ITEM_SIZE: Vector2u = Vector2u {
    x: DEFAULT_BUTTON_SIZE.x,
    y: DEFAULT_BUTTON_SIZE.y,
};
const DEFAULT_SUBMENU_ITEM_SIZE: Vector2u = Vector2u {
    x: DEFAULT_MENU_ITEM_SIZE.x * 5,
    y: DEFAULT_MENU_ITEM_SIZE.y * 5,
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
    data: WidgetData,
    entries: Vec<MenuItemPanel>,
    entries_uid: Vec<UID>,
}
implement_widget!(Menu);

impl Default for Menu {
    fn default() -> Self {
        Self {
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
            .vertical_alignment(VerticalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .with_text(label)
            .text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left)
            .style(WidgetStyle::DefaultBackground);

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
            .visible(false)
            .selectable(true)
            .vertical_alignment(VerticalAlignment::None)
            .horizontal_alignment(HorizontalAlignment::None)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(false)
            .space_between_elements(DEFAULT_WIDGET_SIZE.x / 2 * Screen::get_scale_factor() as u32)
            .style(WidgetStyle::FullInactive);

        submenu.move_to_layer(DEFAULT_MENU_LAYER);

        let menu_item_id = self.add_child(Box::new(button));
        self.entries_uid.push(menu_item_id);
        self.entries.push(MenuItemPanel {
            submenu,
            opened: false,
        });
        menu_item_id
    }
    pub fn get_submenu(&mut self, menu_item_id: UID) -> Option<&mut Menu> {
        if let Some(index) = self.entries_uid.iter().position(|el| *el == menu_item_id) {
            let entry = &mut self.entries[index];
            return Some(&mut entry.submenu);
        }
        None
    }
    pub fn add_submenu_entry(&mut self, menu_item_id: UID, widget: Box<dyn Widget>) -> UID {
        let mut id = INVALID_UID;
        if let Some(index) = self.entries_uid.iter().position(|el| *el == menu_item_id) {
            let entry = &mut self.entries[index];
            id = entry.submenu.add_child(widget);
        }
        id
    }
    pub fn add_submenu_entry_default(
        &mut self,
        renderer: &mut Renderer,
        menu_item_id: UID,
        label: &str,
    ) -> UID {
        let mut id = INVALID_UID;
        if let Some(index) = self.entries_uid.iter().position(|el| *el == menu_item_id) {
            let mut button = Button::default();
            button.init(renderer);
            button
                .with_text(label)
                .text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left)
                .style(WidgetStyle::DefaultBackground);

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
            .space_between_elements(DEFAULT_WIDGET_SIZE.x * Screen::get_scale_factor() as u32)
            .fill_type(ContainerFillType::Horizontal)
            .use_space_before_and_after(false)
            .style(WidgetStyle::DefaultBorder);
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
        self.manage_menu_interactions(events_rw);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
