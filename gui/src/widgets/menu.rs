use std::any::TypeId;

use nrg_math::{VecBase, Vector2, Vector4};
use nrg_messenger::{implement_message, Message};
use nrg_platform::{MouseButton, MouseEvent, MouseState};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, Button, InternalWidget, WidgetData, WidgetEvent,
    DEFAULT_BUTTON_WIDTH, DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_SIZE, DEFAULT_WIDGET_WIDTH,
};

const DEFAULT_MENU_LAYER: f32 = 0.5;
const DEFAULT_MENU_SIZE: [f32; 2] = [DEFAULT_WIDGET_WIDTH * 10., DEFAULT_WIDGET_HEIGHT * 5. / 4.];
const DEFAULT_MENU_ITEM_SIZE: [f32; 2] = [DEFAULT_BUTTON_WIDTH * 10., DEFAULT_WIDGET_HEIGHT * 10.];
const DEFAULT_SUBMENU_ITEM_SIZE: [f32; 2] = [DEFAULT_BUTTON_WIDTH * 5., DEFAULT_WIDGET_HEIGHT * 5.];

#[derive(Clone, Copy)]
pub enum MenuEvent {
    Open(Uid),
}
implement_message!(MenuEvent);

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
struct MenuItemPanel {
    uid: Uid,
    submenu: Menu,
    opened: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Menu {
    data: WidgetData,
    entries: Vec<MenuItemPanel>,
}
implement_widget_with_custom_members!(Menu {
    entries: Vec::new()
});

impl Menu {
    pub fn add_menu_item(&mut self, label: &str) -> Uid {
        let is_horizontal = self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch;

        let mut button = Button::new(self.get_shared_data(), self.get_global_messenger());
        button
            .vertical_alignment(VerticalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_width(false)
            .with_text(label)
            .text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left)
            .style(WidgetStyle::DefaultCanvas);

        let size: Vector2 = DEFAULT_SUBMENU_ITEM_SIZE.into();
        let mut submenu = Menu::new(self.get_shared_data(), self.get_global_messenger());
        if is_horizontal {
            submenu.position([button.state().get_position().x, self.state().get_size().y].into());
        } else {
            submenu.position(
                [
                    button.state().get_position().x + self.state().get_size().x,
                    button.state().get_position().y,
                ]
                .into(),
            );
        }
        submenu
            .size(size * Screen::get_scale_factor())
            .visible(false)
            .selectable(false)
            .fill_type(ContainerFillType::Vertical)
            .vertical_alignment(VerticalAlignment::None)
            .horizontal_alignment(HorizontalAlignment::None)
            .keep_fixed_width(false)
            .keep_fixed_height(false)
            .style(WidgetStyle::DefaultBackground)
            .register_to_listen_event::<MouseEvent>();

        submenu.move_to_layer(DEFAULT_MENU_LAYER);

        let uid = self.add_child(Box::new(button));
        self.entries.push(MenuItemPanel {
            uid,
            submenu,
            opened: false,
        });
        uid
    }
    pub fn get_submenu(&mut self, menu_item_id: Uid) -> Option<&mut Menu> {
        if let Some(index) = self
            .entries
            .iter_mut()
            .position(|el| el.uid == menu_item_id)
        {
            return Some(&mut self.entries[index].submenu);
        }
        None
    }
    pub fn add_submenu_entry(&mut self, menu_item_id: Uid, mut widget: Box<dyn Widget>) -> Uid {
        let mut id = INVALID_UID;
        if let Some(index) = self
            .entries
            .iter_mut()
            .position(|el| el.uid == menu_item_id)
        {
            widget.set_visible(false);
            id = self.entries[index].submenu.add_child(widget);
        }
        id
    }
    pub fn add_submenu_entry_default(&mut self, menu_item_id: Uid, label: &str) -> Uid {
        let mut id = INVALID_UID;
        if let Some(index) = self.entries.iter().position(|el| el.uid == menu_item_id) {
            let mut button = Button::new(self.get_shared_data(), self.get_global_messenger());
            button
                .with_text(label)
                .text_alignment(VerticalAlignment::Center, HorizontalAlignment::Left)
                .style(WidgetStyle::DefaultCanvas);

            button.set_visible(false);
            let entry = &mut self.entries[index];
            id = entry.submenu.add_child(Box::new(button));
        }
        id
    }
    pub fn has_entry(&self, entry: Uid) -> bool {
        let mut has_entry = false;
        self.entries.iter().for_each(|e| {
            has_entry |= e.submenu.node().has_child(entry);
            has_entry |= e.submenu.has_entry(entry);
        });
        has_entry
    }
    pub fn is_submenu_opened(&self) -> bool {
        let mut is_opened = false;
        self.entries.iter().for_each(|e| {
            is_opened |= e.opened;
        });
        is_opened
    }

    pub fn is_hover(&self) -> bool {
        let mut is_hover = <dyn Widget>::is_hover(self);
        self.entries.iter().for_each(|e| {
            is_hover |= e.submenu.is_hover();
        });
        is_hover
    }

    fn is_hovering_entry(&mut self, entry_uid: Uid) -> bool {
        let mut is_hover = self.is_hover();
        if let Some(widget) = self.node().get_child_mut::<Button>(entry_uid) {
            if widget.is_hover() {
                is_hover = true;
            }
        }
        if !is_hover {
            if let Some(index) = self.entries.iter().position(|el| el.uid == entry_uid) {
                let item = &self.entries[index];
                if item.opened && item.submenu.is_hover() {
                    is_hover = true;
                }
            }
        }
        is_hover
    }

    pub fn horizontal(&mut self) -> &mut Self {
        self.horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Top)
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_width(false);
        self
    }
    pub fn vertical(&mut self) -> &mut Self {
        self.horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Top)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(false);
        self
    }
}

impl InternalWidget for Menu {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>()
            .register_to_listen_event::<MenuEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_MENU_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .space_between_elements((DEFAULT_WIDGET_SIZE[0] / 2. * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(false)
            .style(WidgetStyle::DefaultCanvas)
            .horizontal()
            .register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>()
            .register_to_listen_event::<MenuEvent>();
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {
        let is_horizontal = self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch;
        let mut buttons: Vec<(Vector2, Vector2)> = Vec::new();
        self.node().propagate_on_children(|w| {
            let pos = w.state().get_position();
            let size = w.state().get_size();
            buttons.push((pos, size));
        });
        let mut is_hover = self.state().is_hover();
        self.entries.iter_mut().enumerate().for_each(|(i, e)| {
            let mut clip_area = Screen::get_draw_area();
            if e.opened {
                if is_horizontal {
                    clip_area.x = buttons[i].0.x;
                    clip_area.y = buttons[i].0.y + buttons[i].1.y;
                } else {
                    clip_area.x = buttons[i].0.x + buttons[i].1.x;
                    clip_area.y = buttons[i].0.y;
                }
                clip_area.z -= clip_area.x;
                clip_area.w -= clip_area.y;
            } else {
                clip_area.z = 0.;
                clip_area.w = 0.;
            }
            e.submenu.update(clip_area, clip_area);
            is_hover |= e.submenu.is_hover();
        });
        self.state_mut().set_hover(is_hover);
    }

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>()
            .unregister_to_listen_event::<MenuEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<MenuEvent>() {
            let event = msg.as_any().downcast_ref::<MenuEvent>().unwrap();
            let MenuEvent::Open(widget_uid) = *event;
            self.entries.iter_mut().for_each(|e| {
                if e.submenu.has_entry(widget_uid) || e.submenu.node().has_child(widget_uid) {
                    e.opened = true;
                    e.submenu.visible(true);
                }
            });
        } else if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            let is_horizontal =
                self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch;
            if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = *event {
                let mut pos = Vector2::default_zero();
                if let Some(button) = self.node().get_child_mut::<Button>(widget_id) {
                    if is_horizontal {
                        pos.x = button.state().get_position().x;
                        pos.y = self.state().get_size().y;
                    } else {
                        pos.x = button.state().get_position().x + button.state().get_size().x;
                        pos.y = self.state().get_position().y;
                    }
                }
                let mut submenu_uid = INVALID_UID;
                self.entries.iter_mut().for_each(|e| {
                    if e.uid == widget_id {
                        submenu_uid = e.uid;
                        e.opened = true;
                        e.submenu.position(pos).visible(e.opened);
                    } else {
                        e.opened = false;
                        e.submenu.visible(false);
                    }
                });
                if !submenu_uid.is_nil() {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(MenuEvent::Open(submenu_uid).as_boxed())
                        .ok();
                }
            }
        } else if msg.type_id() == TypeId::of::<MouseEvent>() {
            let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
            if event.state == MouseState::Down && event.button == MouseButton::Left {
                let count = self.entries.len();
                for i in 0..count {
                    if self.entries[i].opened {
                        let is_hover = self.is_hover();
                        let is_hovering = self.is_hovering_entry(self.entries[i].uid);
                        let item = &mut self.entries[i];
                        if !is_hover && !is_hovering {
                            item.opened = false;
                            item.submenu.visible(item.opened);
                        }
                    }
                }
            }
        }
    }
    fn widget_on_layout_changed(&mut self) {}
}
