use super::*;
use nrg_graphics::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct EditableText {
    text_widget: UID,
    indicator_widget: UID,
    is_focused: bool,
    current_char: i32,
}

unsafe impl Send for EditableText {}
unsafe impl Sync for EditableText {}

impl Default for EditableText {
    fn default() -> Self {
        Self {
            text_widget: INVALID_ID,
            indicator_widget: INVALID_ID,
            is_focused: false,
            current_char: -1,
        }
    }
}

impl EditableText {
    fn manage_key_pressed(event: &KeyEvent, current_index: i32, text: &mut Text) -> i32 {
        match event.code {
            Key::Enter => {
                if text.is_multiline() {}
                current_index
            }
            Key::Backspace => text.remove_char(current_index),
            Key::Delete => text.remove_char(current_index + 1),
            Key::ArrowLeft => {
                let mut new_index = current_index - 1;
                if new_index < 0 {
                    new_index = 0;
                }
                new_index
            }
            Key::ArrowRight => {
                let mut new_index = current_index + 1;
                let length = text.get_text().len() as i32;
                if new_index >= length {
                    new_index = length - 1;
                }
                new_index
            }
            _ => {
                if event.char.is_ascii() {
                    text.add_char(current_index, event.char)
                } else {
                    current_index
                }
            }
        }
    }

    fn update_text(widget: &mut Widget<Self>, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(key_events) = events.read_events::<KeyEvent>() {
            for event in key_events.iter() {
                if event.state == InputState::JustPressed || event.state == InputState::Pressed {
                    let character = widget.get().current_char;
                    if let Some(text) = widget.get_child::<Text>(widget.get().text_widget) {
                        widget.get_mut().current_char =
                            Self::manage_key_pressed(*event, character, text.get_mut());
                    }
                }
            }
        }
    }

    fn check_focus(widget: &mut Widget<Self>, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(mouse_events) = events.read_events::<MouseEvent>() {
            for event in mouse_events.iter() {
                if event.button == MouseButton::Left && event.state == MouseState::Down {
                    widget.get_mut().is_focused = false;
                }
            }
        }
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Pressed(widget_id) = event {
                    if widget.id() == *widget_id {
                        widget.get_mut().is_focused = true;
                        Self::update_character_from_indicator(widget);
                    } else {
                        widget.get_mut().is_focused = false;
                    }
                }
            }
        }
        let focused = widget.get_mut().is_focused;
        if let Some(indicator) = widget.get_child::<Indicator>(widget.get().indicator_widget) {
            indicator.get_mut().set_active(focused);
        }
    }

    fn update_character_from_indicator(widget: &mut Widget<Self>) {
        let mut current_char = widget.get().current_char;
        if let Some(text) = widget.get_child::<Text>(widget.get().text_widget) {
            current_char = text.get().get_hover_char();
            if current_char < 0 {
                current_char = text.get().get_text().len() as i32 - 1;
            }
        }
        widget.get_mut().current_char = current_char;
    }

    fn update_indicator_position(widget: &mut Widget<Self>) {
        let mut current_char = widget.get().current_char;
        if let Some(text) = widget.get_child::<Text>(widget.get().text_widget) {
            let length = text.get().get_text().len() as i32;
            if current_char < 0 || current_char >= length {
                current_char = length - 1;
            }
            let pos = {
                if current_char >= 0 {
                    let pos_in_screen_space = text.get().get_char_pos(current_char);
                    widget
                        .get_screen()
                        .from_screen_space_into_pixels(pos_in_screen_space)
                } else {
                    text.get_data().state.get_position()
                }
            };
            if let Some(indicator) = widget.get_child::<Indicator>(widget.get().indicator_widget) {
                indicator.position(pos);
            }
        }
    }
}

impl WidgetTrait for EditableText {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let size = DEFAULT_WIDGET_SIZE * screen.get_scale_factor();

        data.graphics
            .init(renderer, "UI")
            .set_style(WidgetStyle::default());
        widget
            .size(size)
            .draggable(false)
            .selectable(true)
            .stroke(2)
            .horizontal_alignment(HorizontalAlignment::Stretch);

        let mut text = Widget::<Text>::new(screen.clone());
        text.init(renderer)
            .draggable(false)
            .size(size)
            .vertical_alignment(VerticalAlignment::Stretch)
            .horizontal_alignment(HorizontalAlignment::Left);
        text.get_mut().set_text("Edit me");
        widget.get_mut().text_widget = widget.add_child(text);

        let mut indicator = Widget::<Indicator>::new(screen);
        indicator.init(renderer).get_mut().set_active(false);
        widget.get_mut().indicator_widget = widget.add_child(indicator);
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        events_rw: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        Self::check_focus(widget, events_rw);
        Self::update_indicator_position(widget);
        Self::update_text(widget, events_rw);

        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let pos = screen.convert_from_pixels_into_screen_space(data.state.get_position());
        let size = screen.convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
