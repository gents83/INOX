use nrg_core::*;
use nrg_graphics::*;
use std::path::PathBuf;

const FONT_PATH: &str = "C:\\PROJECTS\\NRG\\data\\fonts\\BasicFont.ttf";

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
        }
    }
}

impl System for MySystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        //println!("Executing init() for MySystem[{:?}]", self.id());
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for MySystem[{:?}]", self.id());
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
        if renderer.get_fonts_count() < 1 {
            renderer.request_font(PathBuf::from(FONT_PATH));
        }
        if let Some(ref mut font) = renderer.get_default_font() {
            font.add_text(
                String::from("Hi, GENTS!\n\nThis is new\n\nNRG\n\nplugin architecture").as_str(),
                [-0.9, -0.7].into(),
                1.0,
                [0.0, 0.8, 1.0].into(),
            );
        }
        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for MySystem[{:?}]", self.id());
    }
}
