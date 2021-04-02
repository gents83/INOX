use std::collections::HashMap;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    font_id: FontId,
    keys: HashMap<Key, InputState>,
    mouse: MouseEvent,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            font_id: INVALID_ID,
            keys: HashMap::new(),
            mouse: MouseEvent::default(),
        }
    }
}

impl System for MySystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let pipeline_id = renderer.get_pipeline_id("UI");
        self.font_id = renderer.add_font(pipeline_id, self.config.fonts.first().unwrap());
    }

    fn run(&mut self) -> bool {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
        let window = &*read_data.get_unique_resource::<Window>();
        let window_events = window.get_events();
        let events = window_events.read().unwrap();
        if let Some(key_events) = events.read_events::<KeyEvent>() {
            for event in key_events.iter() {
                let entry = self.keys.entry(event.code).or_insert(InputState::Released);
                *entry = event.state;
            }
        }

        if let Some(mouse_events) = events.read_events::<MouseEvent>() {
            if let Some(&event) = mouse_events.last() {
                self.mouse = *event;
            }
        }
        let mut line = 0.05;
        let string = format!(
            "Mouse [{:?}, {:?}], {:?}, {:?}",
            self.mouse.x, self.mouse.y, self.mouse.button, self.mouse.state
        );
        renderer.add_text(
            self.font_id,
            string.as_str(),
            [-0.9, -0.9 + line].into(),
            25. * window.get_scale_factor(),
            [0., 0.8, 1., 1.].into(),
            Vector2f { x: 0., y: 0. } * window.get_scale_factor(),
        );
        line += 0.05;

        for (key, state) in self.keys.iter_mut() {
            let string = format!("{:?} = {:?}", key, state);
            renderer.add_text(
                self.font_id,
                string.as_str(),
                [-0.9, -0.9 + line].into(),
                25. * window.get_scale_factor(),
                [0., 0.8, 1., 1.].into(),
                Vector2f { x: 0., y: 0. } * window.get_scale_factor(),
            );
            line += 0.05;
        }
        true
    }
    fn uninit(&mut self) {}
}
