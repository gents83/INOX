use nrg_commands::ExecuteCommand;
use nrg_graphics::{MeshData, Renderer};
use nrg_math::Vector4u;
use nrg_platform::{
    EventsRw, InputHandler, InputState, Key, KeyEvent, KeyTextEvent, MouseButton, MouseEvent,
    MouseState,
};
use nrg_serialize::{Deserialize, Serialize, INVALID_UID, UID};

use crate::{
    implement_widget, AddCharCommand, Indicator, InternalWidget, RemoveCharCommand, Text,
    WidgetData, WidgetEvent, DEFAULT_TEXT_SIZE,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct EditableText {
    #[serde(skip)]
    text_widget: UID,
    #[serde(skip)]
    indicator_widget: UID,
    #[serde(skip)]
    is_focused: bool,
    #[serde(skip)]
    current_char: i32,
    data: WidgetData,
}
implement_widget!(EditableText);

impl Default for EditableText {
    fn default() -> Self {
        Self {
            text_widget: INVALID_UID,
            indicator_widget: INVALID_UID,
            is_focused: false,
            current_char: -1,
            data: WidgetData::default(),
        }
    }
}

impl EditableText {
    pub fn get_text(&mut self) -> String {
        let mut text_result = String::new();
        let text_widget_id = self.text_widget;
        if let Some(text) = self.get_data_mut().node.get_child::<Text>(text_widget_id) {
            text_result = text.get_text().to_string();
        }
        text_result
    }
    fn manage_char_input(&mut self, events_rw: &EventsRw) {
        let mut commands: Vec<AddCharCommand> = Vec::new();
        let mut current_index = self.current_char;
        {
            let events = events_rw.read().unwrap();
            if let Some(key_text_events) = events.read_events::<KeyTextEvent>() {
                for event in key_text_events.iter() {
                    if !event.char.is_control() {
                        commands.push(AddCharCommand::new(
                            self.text_widget,
                            current_index,
                            event.char,
                        ));
                        current_index += 1;
                    }
                }
            }
        }
        self.current_char = current_index;
        let mut events = events_rw.write().unwrap();
        for command in commands.into_iter() {
            events.send_event::<ExecuteCommand>(ExecuteCommand::new(command));
        }
    }

    fn manage_key_pressed(&mut self, events_rw: &mut EventsRw) {
        let mut commands: Vec<RemoveCharCommand> = Vec::new();
        let mut current_index = self.current_char;
        {
            let text_widget_id = self.text_widget;
            let events = events_rw.read().unwrap();
            if let Some(key_events) = events.read_events::<KeyEvent>() {
                for event in key_events.iter() {
                    if event.state == InputState::JustPressed || event.state == InputState::Pressed
                    {
                        match event.code {
                            Key::Backspace => {
                                if let Some(text) =
                                    self.get_data_mut().node.get_child::<Text>(text_widget_id)
                                {
                                    if let Some(c) = text.get_char_at(current_index) {
                                        commands.push(RemoveCharCommand::new(
                                            text_widget_id,
                                            current_index,
                                            c,
                                        ));
                                        current_index -= 1;
                                    }
                                }
                            }
                            Key::Delete => {
                                if let Some(text) =
                                    self.get_data_mut().node.get_child::<Text>(text_widget_id)
                                {
                                    if let Some(c) = text.get_char_at(current_index + 1) {
                                        commands.push(RemoveCharCommand::new(
                                            text_widget_id,
                                            current_index + 1,
                                            c,
                                        ));
                                    }
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
                                if let Some(text) =
                                    self.get_data_mut().node.get_child::<Text>(text_widget_id)
                                {
                                    let length = text.get_text().len() as i32;
                                    if new_index >= length {
                                        new_index = length - 1;
                                    }
                                }
                                current_index = new_index;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        self.current_char = current_index;
        let mut events = events_rw.write().unwrap();
        for command in commands.into_iter() {
            events.send_event::<ExecuteCommand>(ExecuteCommand::new(command));
        }
    }

    fn update_text(&mut self, events_rw: &mut EventsRw) {
        if self.is_focused {
            self.update_indicator_position();
            self.manage_char_input(events_rw);
            self.manage_key_pressed(events_rw);
        }
    }

    fn check_focus(&mut self, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(mouse_events) = events.read_events::<MouseEvent>() {
            for event in mouse_events.iter() {
                if event.button == MouseButton::Left && event.state == MouseState::Down {
                    self.is_focused = false;
                }
            }
        }
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Pressed(widget_id) = event {
                    if self.id() == *widget_id {
                        self.is_focused = true;
                        self.update_character_from_indicator();
                    } else {
                        self.is_focused = false;
                    }
                }
            }
        }
        let focused = self.is_focused;
        let indicator_id = self.indicator_widget;
        if let Some(indicator) = self
            .get_data_mut()
            .node
            .get_child::<Indicator>(indicator_id)
        {
            indicator.visible(focused);
        }
    }

    fn update_character_from_indicator(&mut self) {
        let mut current_char = self.current_char;
        let text_widget_id = self.text_widget;
        if let Some(text) = self.get_data_mut().node.get_child::<Text>(text_widget_id) {
            current_char = text.get_hover_char();
            if current_char < 0 {
                current_char = text.get_text().len() as i32 - 1;
            }
        }
        self.current_char = current_char;
    }

    fn update_indicator_position(&mut self) {
        let mut current_char = self.current_char;
        let text_widget_id = self.text_widget;
        if let Some(text) = self.get_data_mut().node.get_child::<Text>(text_widget_id) {
            let length = text.get_text().len() as i32;
            if current_char < 0 {
                current_char = -1;
            }
            if current_char >= length {
                current_char = length - 1;
            }
            let pos = {
                if current_char >= 0 {
                    let pos_in_screen_space = text.get_char_pos(current_char);
                    Screen::from_screen_space_into_pixels(pos_in_screen_space)
                } else {
                    text.get_data().state.get_position()
                }
            };
            let indicator_id = self.indicator_widget;
            if let Some(indicator) = self
                .get_data_mut()
                .node
                .get_child::<Indicator>(indicator_id)
            {
                indicator.position(pos);
            }
        }
    }
}

impl InternalWidget for EditableText {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        let default_size = DEFAULT_TEXT_SIZE * Screen::get_scale_factor();

        self.size(default_size)
            .horizontal_alignment(HorizontalAlignment::Right);

        let mut text = Text::default();
        text.init(renderer);
        text.vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Left);
        text.set_text("Edit me");
        self.text_widget = self.add_child(Box::new(text));

        let mut indicator = Indicator::default();
        indicator.init(renderer);
        indicator.visible(false);
        self.indicator_widget = self.add_child(Box::new(indicator));
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        events_rw: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        self.check_focus(events_rw);
        self.update_text(events_rw);

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
