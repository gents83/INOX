use crate::{events::*, WindowEvent};

use super::mouse::*;

pub struct InputHandler {
    pub mouse: MouseData,
    input_area_width: f64,
    input_area_height: f64,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            mouse: MouseData::default(),
            input_area_width: 0.0,
            input_area_height: 0.0,
        }
    }
}

impl InputHandler {
    pub fn init(&mut self, input_area_width: f64, input_area_height: f64) -> &mut Self {
        self.input_area_width = input_area_width;
        self.input_area_height = input_area_height;
        self
    }
    pub fn update(&mut self, events: &EventsRw) {
        self.manage_window_events(events);
        self.manage_mouse_events(events);
    }

    pub fn get_mouse_data(&self) -> &MouseData {
        &self.mouse
    }
}

impl InputHandler {
    fn manage_window_events(&mut self, events: &EventsRw) {
        let events = events.read().unwrap();
        if let Some(window_events) = events.read_events::<WindowEvent>() {
            for event in window_events.iter() {
                if let WindowEvent::SizeChanged(width, height) = event {
                    self.input_area_width = *width as _;
                    self.input_area_height = *height as _;
                }
            }
        }
    }
    fn manage_mouse_events(&mut self, events: &EventsRw) {
        self.mouse.move_x = 0.0;
        self.mouse.move_y = 0.0;
        let events = events.read().unwrap();
        if let Some(mouse_events) = events.read_events::<MouseEvent>() {
            let mut pos_x = self.mouse.pos_x;
            let mut pos_y = self.mouse.pos_y;
            for event in mouse_events.iter() {
                pos_x = event.x / self.input_area_width;
                pos_y = event.y / self.input_area_height;
                if !self.mouse.is_pressed
                    && event.button == MouseButton::Left
                    && event.state == MouseState::Down
                {
                    self.mouse.is_pressed = true
                } else if event.button == MouseButton::Left && event.state == MouseState::Up {
                    self.mouse.is_pressed = false;
                }
                if self.mouse.is_pressed {
                    self.mouse.move_x = pos_x - self.mouse.pos_x;
                    self.mouse.move_y = pos_y - self.mouse.pos_y;
                }
                self.mouse
                    .buttons
                    .entry(event.button)
                    .and_modify(|e| *e = event.state)
                    .or_insert(event.state);
            }
            self.mouse.pos_x = pos_x;
            self.mouse.pos_y = pos_y;
        }
    }
}
