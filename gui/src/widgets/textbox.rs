use std::any::TypeId;

use nrg_math::Vector2;
use nrg_messenger::Message;
use nrg_platform::{InputState, Key, KeyEvent, KeyTextEvent, MouseButton, MouseEvent, MouseState};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, Indicator, InternalWidget, Panel, Text, TextEvent,
    WidgetData, WidgetEvent, DEFAULT_TEXT_SIZE, DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
};
pub const DEFAULT_ICON_SIZE: [f32; 2] = [
    DEFAULT_WIDGET_WIDTH * 2. / 3.,
    DEFAULT_WIDGET_HEIGHT * 2. / 3.,
];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct TextBox {
    data: WidgetData,
    is_editable: bool,
    #[serde(skip)]
    label: Uid,
    #[serde(skip)]
    editable_text: Uid,
    #[serde(skip)]
    text_panel: Uid,
    #[serde(skip)]
    indicator_widget: Uid,
    #[serde(skip)]
    is_focused: bool,
    #[serde(skip)]
    current_char: i32,
}
implement_widget_with_custom_members!(TextBox {
    is_editable: true,
    label: INVALID_UID,
    editable_text: INVALID_UID,
    text_panel: INVALID_UID,
    indicator_widget: INVALID_UID,
    is_focused: false,
    current_char: -1
});

impl TextBox {
    pub fn editable(&mut self, is_editable: bool) -> &mut Self {
        self.is_editable = is_editable;
        let uid = self.text_panel;
        if let Some(text_panel) = self.node_mut().get_child::<Panel>(uid) {
            text_panel.selectable(is_editable);
        }
        self
    }
    pub fn is_editable(&self) -> bool {
        self.is_editable
    }
    pub fn with_label(&mut self, text: &str) -> &mut Self {
        let uid = self.label;
        if let Some(label) = self.node_mut().get_child::<Text>(uid) {
            label.set_text(text);
        }
        self
    }
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        let uid = self.editable_text;
        if let Some(editable_text) = self.node_mut().get_child::<Text>(uid) {
            editable_text.set_text(text);
        }
        self
    }

    pub fn get_text(&mut self) -> String {
        let mut str = String::new();
        let uid = self.editable_text;
        if let Some(editable_text) = self.node_mut().get_child::<Text>(uid) {
            str = String::from(editable_text.get_text());
        }
        str
    }

    fn update_character_from_indicator(&mut self) {
        let mut current_char = self.current_char;
        let text_widget_id = self.editable_text;
        if let Some(text) = self.node_mut().get_child::<Text>(text_widget_id) {
            current_char = text.get_hover_char() - 1;
            if current_char < 0 {
                current_char = text.get_text().len() as i32 - 1;
            }
        }
        self.current_char = current_char;
    }

    fn update_indicator_position(&mut self) {
        let mut current_char = self.current_char;
        let text_widget_id = self.editable_text;
        if let Some(text) = self.node_mut().get_child::<Text>(text_widget_id) {
            let length = text.get_text().len() as i32;
            if current_char < 0 {
                current_char = -1;
            }
            if current_char >= length {
                current_char = length - 1;
            }
            let pos = {
                if current_char >= 0 {
                    text.get_char_pos(current_char)
                } else {
                    text.state().get_position()
                }
            };
            let indicator_id = self.indicator_widget;
            if let Some(indicator) = self.node_mut().get_child::<Indicator>(indicator_id) {
                indicator.position(pos);
            }
        }
    }
}

impl InternalWidget for TextBox {
    fn widget_init(&mut self) {
        self.get_global_messenger()
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(self.get_messagebox());
        self.get_global_messenger()
            .write()
            .unwrap()
            .register_messagebox::<KeyTextEvent>(self.get_messagebox());

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [400., DEFAULT_TEXT_SIZE[1]].into();

        self.size(size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_width(false)
            .keep_fixed_height(false)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .space_between_elements(10)
            .use_space_before_and_after(false)
            .selectable(false)
            .style(WidgetStyle::Invisible);

        let mut label = Text::new(self.get_shared_data(), self.get_global_messenger());
        label
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center);
        label.set_text("Label: ");

        self.label = self.add_child(Box::new(label));

        let mut panel = Panel::new(self.get_shared_data(), self.get_global_messenger());
        panel
            .size(size * Screen::get_scale_factor())
            .draggable(false)
            .selectable(true)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .style(WidgetStyle::Default);

        let mut editable_text = Text::new(self.get_shared_data(), self.get_global_messenger());
        editable_text
            .size(size * Screen::get_scale_factor())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .set_text("Edit me");

        let mut indicator = Indicator::new(self.get_shared_data(), self.get_global_messenger());
        indicator.visible(false);
        self.indicator_widget = editable_text.add_child(Box::new(indicator));

        self.editable_text = panel.add_child(Box::new(editable_text));
        self.text_panel = self.add_child(Box::new(panel));
    }

    fn widget_update(&mut self) {
        if self.is_editable() && self.is_focused {
            self.update_indicator_position();
        }
    }

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<MouseEvent>() {
            let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
            if event.button == MouseButton::Left && event.state == MouseState::Down {
                self.is_focused = false;
            }
        } else if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = *event {
                if self.text_panel == widget_id {
                    self.is_focused = true;
                } else {
                    self.is_focused = false;
                }
                self.update_character_from_indicator();
                let indicator_id = self.indicator_widget;
                let focused = self.is_focused;
                if let Some(indicator) = self.node_mut().get_child::<Indicator>(indicator_id) {
                    indicator.visible(focused);
                }
            }
        } else if msg.type_id() == TypeId::of::<KeyTextEvent>() {
            let event = msg.as_any().downcast_ref::<KeyTextEvent>().unwrap();
            if !self.is_focused {
                return;
            }
            let events_dispatcher = self.get_global_dispatcher();
            if !event.char.is_control() {
                let text_event =
                    TextEvent::AddChar(self.editable_text, self.current_char, event.char);
                self.current_char += 1;

                events_dispatcher
                    .write()
                    .unwrap()
                    .send(text_event.as_boxed())
                    .ok();
            }
        } else if msg.type_id() == TypeId::of::<KeyEvent>() {
            let event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
            if !self.is_focused {
                return;
            }
            let events_dispatcher = self.get_global_dispatcher();
            let text_id = self.editable_text;
            let mut current_char = self.current_char;
            if let Some(text) = self.node_mut().get_child::<Text>(text_id) {
                if event.state == InputState::JustPressed || event.state == InputState::Pressed {
                    match event.code {
                        Key::Backspace => {
                            if let Some(c) = text.get_char_at(current_char) {
                                let text_event = TextEvent::RemoveChar(text.id(), current_char, c);
                                current_char -= 1;
                                events_dispatcher
                                    .write()
                                    .unwrap()
                                    .send(text_event.as_boxed())
                                    .ok();
                            }
                        }
                        Key::Delete => {
                            if let Some(c) = text.get_char_at(current_char + 1) {
                                let text_event =
                                    TextEvent::RemoveChar(text.id(), current_char + 1, c);
                                events_dispatcher
                                    .write()
                                    .unwrap()
                                    .send(text_event.as_boxed())
                                    .ok();
                            }
                        }
                        Key::ArrowLeft => {
                            let mut new_index = current_char - 1;
                            if new_index < 0 {
                                new_index = -1;
                            }
                            current_char = new_index;
                        }
                        Key::ArrowRight => {
                            let mut new_index = current_char + 1;
                            let length = text.get_text().len() as i32;
                            if new_index >= length {
                                new_index = length - 1;
                            }
                            current_char = new_index;
                        }
                        _ => {}
                    }
                }
            }
            self.current_char = current_char;
        }
    }
}
