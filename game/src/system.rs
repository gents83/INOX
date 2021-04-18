use std::path::Path;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    font_id: FontId,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            font_id: INVALID_ID,
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

        let material_id = renderer.add_material(pipeline_id);
        renderer.add_texture(material_id, &Path::new("./data/textures/Test.png"));
        let mut mesh = MeshData::default();
        mesh.add_quad_default([-0.75, -0.75, -0.25, 0.75].into(), 0.);
        renderer.add_mesh(material_id, &mesh);

        let material_id = renderer.add_material(pipeline_id);
        renderer.add_texture(material_id, &Path::new("./data/textures/Test.jpg"));
        let mut mesh = MeshData::default();
        mesh.add_quad_default([0.25, -0.75, 0.75, 0.75].into(), 0.);
        renderer.add_mesh(material_id, &mesh);
    }

    fn run(&mut self) -> bool {
        true
    }
    fn uninit(&mut self) {}
}
