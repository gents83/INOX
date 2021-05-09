use nrg_math::Vector2;
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
    fn manage_char_input(&mut self) {
        let mut new_events: Vec<TextEvent> = Vec::new();
        self.current_char = {
            let mut current_index = self.current_char;
            let events = self.get_events().read().unwrap();
            if let Some(key_text_events) = events.read_all_events::<KeyTextEvent>() {
                for event in key_text_events.iter() {
                    if !event.char.is_control() {
                        new_events.push(TextEvent::AddChar(
                            self.editable_text,
                            current_index,
                            event.char,
                        ));
                        current_index += 1;
                    }
                }
            }
            current_index
        };

        let mut events = self.get_events().write().unwrap();
        for e in new_events.into_iter() {
            events.send_event::<TextEvent>(e);
        }
    }

    fn manage_key_pressed(events_rw: &EventsRw, text: &Text, current_char: i32) -> i32 {
        let mut new_events: Vec<TextEvent> = Vec::new();
        let new_char = {
            let mut current_index = current_char;
            let events = events_rw.read().unwrap();
            if let Some(key_events) = events.read_all_events::<KeyEvent>() {
                for event in key_events.iter() {
                    if event.state == InputState::JustPressed || event.state == InputState::Pressed
                    {
                        match event.code {
                            Key::Backspace => {
                                if let Some(c) = text.get_char_at(current_index) {
                                    new_events.push(TextEvent::RemoveChar(
                                        text.id(),
                                        current_index,
                                        c,
                                    ));
                                    current_index -= 1;
                                }
                            }
                            Key::Delete => {
                                if let Some(c) = text.get_char_at(current_index + 1) {
                                    new_events.push(TextEvent::RemoveChar(
                                        text.id(),
                                        current_index + 1,
                                        c,
                                    ));
                                }
                            }
                            Key::ArrowLeft => {
                                let mut new_index = current_index - 1;
                                if new_index < 0 {
                                    new_index = -1;
                                }
                                current_index = new_index;
                            }
                            Key::ArrowRight => {
                                let mut new_index = current_index + 1;
                                let length = text.get_text().len() as i32;
                                if new_index >= length {
                                    new_index = length - 1;
                                }
                                current_index = new_index;
                            }
                            _ => {}
                        }
                    }
                }
            }
            current_index
        };
        let mut events = events_rw.write().unwrap();
        for e in new_events.into_iter() {
            events.send_event::<TextEvent>(e);
        }
        new_char
    }

    fn update_text(&mut self) {
        if self.is_focused {
            self.update_indicator_position();
            self.manage_char_input();
            let text_widget_id = self.editable_text;
            let current_char = self.current_char;
            let events_rw = self.get_events().clone();
            if let Some(text) = self.node_mut().get_child::<Text>(text_widget_id) {
                self.current_char = TextBox::manage_key_pressed(&events_rw, text, current_char);
            }
        }
    }

    fn check_focus(&mut self) {
        let focused = {
            let mut focused = self.is_focused;
            let events = self.get_events().read().unwrap();
            if let Some(mouse_events) = events.read_all_events::<MouseEvent>() {
                for event in mouse_events.iter() {
                    if event.button == MouseButton::Left && event.state == MouseState::Down {
                        focused = false;
                    }
                }
            }
            if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
                for event in widget_events.iter() {
                    if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
                        if self.text_panel == *widget_id {
                            focused = true;
                        } else {
                            focused = false;
                        }
                    }
                }
            }
            focused
        };
        if self.is_focused != focused {
            self.is_focused = focused;
            self.update_character_from_indicator();
            let indicator_id = self.indicator_widget;
            if let Some(indicator) = self.node_mut().get_child::<Indicator>(indicator_id) {
                indicator.visible(focused);
            }
        }
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

        let mut label = Text::new(self.get_shared_data(), self.get_events());
        label
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center);
        label.set_text("Label: ");

        self.label = self.add_child(Box::new(label));

        let mut panel = Panel::new(self.get_shared_data(), self.get_events());
        panel
            .size(size * Screen::get_scale_factor())
            .draggable(false)
            .selectable(true)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .style(WidgetStyle::Default);

        let mut editable_text = Text::new(self.get_shared_data(), self.get_events());
        editable_text
            .size(size * Screen::get_scale_factor())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .set_text("Edit me");

        let mut indicator = Indicator::new(self.get_shared_data(), self.get_events());
        indicator.visible(false);
        self.indicator_widget = editable_text.add_child(Box::new(indicator));

        self.editable_text = panel.add_child(Box::new(editable_text));
        self.text_panel = self.add_child(Box::new(panel));
    }

    fn widget_update(&mut self) {
        if self.is_editable() {
            self.check_focus();
            self.update_text();
        }
    }

    fn widget_uninit(&mut self) {}
}
