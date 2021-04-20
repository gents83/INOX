use std::path::Path;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_math::Matrix4;

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    font_id: FontId,
    left_material_id: MaterialId,
    right_material_id: MaterialId,
    left_mesh_id: MeshId,
    right_mesh_id: MeshId,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            font_id: INVALID_ID,
            left_material_id: INVALID_ID,
            right_material_id: INVALID_ID,
            left_mesh_id: INVALID_ID,
            right_mesh_id: INVALID_ID,
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

        self.left_material_id = renderer.add_material(pipeline_id);
        renderer.add_texture(
            self.left_material_id,
            &Path::new("./data/textures/Test.png"),
        );
        self.right_material_id = renderer.add_material(pipeline_id);
        renderer.add_texture(
            self.right_material_id,
            &Path::new("./data/textures/Test.jpg"),
        );

        let mut mesh = MeshData::default();
        mesh.add_quad_default([0., 0., 1., 1.].into(), 0.);

        self.left_mesh_id = renderer.add_mesh(self.left_material_id, &mesh);

        let mut mesh = MeshData::default();
        mesh.add_quad_default([0., 0., 1., 1.].into(), 0.);
        self.right_mesh_id = renderer.add_mesh(self.right_material_id, &mesh);
    }

    fn run(&mut self) -> bool {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let mut left_matrix = Matrix4::from_nonuniform_scale(400., 400., 1.);
        left_matrix[3][0] = 100.;
        left_matrix[3][1] = 100.;
        renderer.update_mesh(self.left_material_id, self.left_mesh_id, &left_matrix);

        let mut right_matrix = Matrix4::from_nonuniform_scale(400., 600., 1.);
        right_matrix[3][0] = 700.;
        right_matrix[3][1] = 0.;
        renderer.update_mesh(self.right_material_id, self.right_mesh_id, &right_matrix);

        true
    }
    fn uninit(&mut self) {}
}
