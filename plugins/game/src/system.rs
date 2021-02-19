use std::collections::HashMap;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_platform::*;

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    keys: HashMap<Key, InputState>,
    mouse: MouseEvent,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            keys: HashMap::new(),
            mouse: MouseEvent::default(),
        }
    }
}

impl System for MySystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
        let window = &*read_data.get_unique_resource::<Window>();

        let key_events = window.get_events().read_events::<KeyEvent>();
        let mut line = 0.05;
        for event in key_events.iter() {
            let entry = self.keys.entry(event.code).or_insert(InputState::Released);
            *entry = event.state;
        }

        if renderer.get_fonts_count() < 1 && !self.config.fonts.is_empty() {
            renderer.request_font(self.config.fonts.first().unwrap());
        }
        if let Some(ref mut font) = renderer.get_default_font() {
            let mouse_events = window.get_events().read_events::<MouseEvent>();
            if let Some(event) = mouse_events.last() {
                self.mouse = (*event).clone();
            }
            let string = format!(
                "Mouse [{:?}, {:?}], {:?}, , {:?}",
                self.mouse.x, self.mouse.y, self.mouse.button, self.mouse.state
            );
            font.add_text(
                string.as_str(),
                [-0.9, -0.9 + line].into(),
                1.0,
                [0.0, 0.8, 1.0].into(),
            );
            line += 0.05;

            for (key, state) in self.keys.iter_mut() {
                let string = format!("{:?} = {:?}", key, state);
                font.add_text(
                    string.as_str(),
                    [-0.9, -0.9 + line].into(),
                    1.0,
                    [0.0, 0.8, 1.0].into(),
                );
                line += 0.05;
            }
        }
        true
    }
    fn uninit(&mut self) {}
}
