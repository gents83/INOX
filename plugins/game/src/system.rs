use super::config::*;

use nrg_core::*;
use nrg_graphics::*;

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
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
        if renderer.get_fonts_count() < 1 && !self.config.fonts.is_empty() {
            renderer.request_font(self.config.fonts.first().unwrap());
        }
        if let Some(ref mut font) = renderer.get_default_font() {
            font.add_text(
                String::from("Hi, GENTS!\n\nThis is new\n\nNRG\n\nengine plugin architecture")
                    .as_str(),
                [-0.9, -0.7].into(),
                1.0,
                [0.0, 0.8, 1.0].into(),
            );
        }
        true
    }
    fn uninit(&mut self) {}
}
