use nrg_graphics::Renderer;
use nrg_math::{VecBase, Vector2};
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget, Button, InternalWidget, WidgetData, WidgetEvent, DEFAULT_BUTTON_WIDTH,
    DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_SIZE,
};

const DEFAULT_MENU_LAYER: f32 = 0.5;
const DEFAULT_MENU_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 10., DEFAULT_WIDGET_HEIGHT * 3. / 2.];
const DEFAULT_MENU_ITEM_SIZE: [f32; 2] = [DEFAULT_BUTTON_WIDTH, DEFAULT_WIDGET_HEIGHT];
const DEFAULT_SUBMENU_ITEM_SIZE: [f32; 2] = [DEFAULT_BUTTON_WIDTH * 5., DEFAULT_WIDGET_HEIGHT * 5.];

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
    entries_uid: Vec<Uid>,
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
    pub fn add_menu_item(&mut self, renderer: &mut Renderer, label: &str) -> Uid {
        let mut button = Button::default();
        button.init(renderer);
        button
            .vertical_alignment(VerticalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .with_text(label)
            .text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left)
            .style(WidgetStyle::DefaultBackground);

        let size: Vector2 = DEFAULT_SUBMENU_ITEM_SIZE.into();
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
            .size(size * Screen::get_scale_factor())
            .visible(false)
            .selectable(true)
            .vertical_alignment(VerticalAlignment::None)
            .horizontal_alignment(HorizontalAlignment::None)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(false)
            .space_between_elements((DEFAULT_WIDGET_SIZE[0] / 2. * Screen::get_scale_factor()) as _)
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
    pub fn get_submenu(&mut self, menu_item_id: Uid) -> Option<&mut Menu> {
        if let Some(index) = self.entries_uid.iter().position(|el| *el == menu_item_id) {
            let entry = &mut self.entries[index];
            return Some(&mut entry.submenu);
        }
        None
    }
    pub fn add_submenu_entry(&mut self, menu_item_id: Uid, widget: Box<dyn Widget>) -> Uid {
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
        menu_item_id: Uid,
        label: &str,
    ) -> Uid {
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
    fn is_hovering_entry(&mut self, entry_uid: Uid) -> bool {
        let mut is_hover = false;
        if let Some(widget) = self.get_data_mut().node.get_child::<Button>(entry_uid) {
            if widget.is_hover() {
                is_hover = true;
            }
        }
        if !is_hover {
            if let Some(index) = self.entries_uid.iter().position(|el| *el == entry_uid) {
                let item = &self.entries[index];
                if item.opened && item.submenu.is_hover() {
                    is_hover = true;
                }
            }
        }
        is_hover
    }
    fn manage_hovering(&mut self) {
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

    fn manage_menu_interactions(&mut self, events_rw: &mut EventsRw) {
        self.manage_hovering();

        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Released(widget_id) = event {
                    let mut pos = Vector2::default_zero();
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
    fn widget_init(&mut self, _renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_MENU_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .space_between_elements((DEFAULT_WIDGET_SIZE[0] * Screen::get_scale_factor()) as _)
            .fill_type(ContainerFillType::Horizontal)
            .use_space_before_and_after(false)
            .style(WidgetStyle::DefaultBorder);
    }

    fn widget_update(&mut self, renderer: &mut Renderer, events_rw: &mut EventsRw) {
        self.manage_menu_interactions(events_rw);
        let drawing_area_in_px = self.get_data().state.get_clip_area();
        self.entries.iter_mut().for_each(|e| {
            e.submenu.update(drawing_area_in_px, renderer, events_rw);
        });
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
