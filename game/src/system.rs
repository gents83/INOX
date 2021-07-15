#![allow(dead_code)]
use std::path::Path;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_math::{Matrix4, Vector3};
use nrg_resources::{DataTypeResource, FileResource, SharedDataRw};

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    font: FontRc,
    left_material: MaterialRc,
    right_material: MaterialRc,
    left_texture: TextureRc,
    right_texture: TextureRc,
    left_mesh: MeshRc,
    right_mesh: MeshRc,
    angle: f32,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        let pipeline = PipelineInstance::find_from_name(shared_data, "UI");

        let mut mesh = MeshData::default();
        mesh.add_quad_default([0., 0., 1., 1.].into(), 0.);
        let left_mesh = MeshInstance::create_from_data(shared_data, mesh);

        let mut mesh = MeshData::default();
        mesh.add_quad(
            [-0.5, -0.5, 0.5, 0.5].into(),
            1.,
            [0.0, 0.0, 1.0, 1.0].into(),
            None,
        );
        let right_mesh = MeshInstance::create_from_data(shared_data, mesh);

        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            font: FontInstance::create_from_file(shared_data, config.fonts.first().unwrap()),
            left_material: MaterialInstance::create_from_pipeline(shared_data, pipeline.clone()),
            right_material: MaterialInstance::create_from_pipeline(shared_data, pipeline),
            left_texture: TextureInstance::create_from_file(
                shared_data,
                &Path::new("./data/textures/Test.png"),
            ),
            right_texture: TextureInstance::create_from_file(
                shared_data,
                &Path::new("./data/textures/Test.jpg"),
            ),
            left_mesh,
            right_mesh,
            angle: 0.,
        }
    }
}

impl System for MySystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.left_material
            .resource()
            .get_mut()
            .add_texture(self.left_texture.clone());
        self.left_material
            .resource()
            .get_mut()
            .add_mesh(self.left_mesh.clone());

        self.right_material
            .resource()
            .get_mut()
            .add_texture(self.right_texture.clone());
        self.right_material
            .resource()
            .get_mut()
            .add_mesh(self.right_mesh.clone());
    }

    fn run(&mut self) -> bool {
        let mut left_matrix = Matrix4::from_nonuniform_scale(400., 400., 1.);
        left_matrix[3][0] = 100.;
        left_matrix[3][1] = 100.;

        self.left_mesh
            .resource()
            .get_mut()
            .set_transform(left_matrix);

        self.angle += 0.1;
        let right_matrix = Matrix4::from_translation(Vector3::new(1000., 800., 0.))
            * Matrix4::from_angle_z(nrg_math::Rad::from(nrg_math::Deg(self.angle)))
            * Matrix4::from_nonuniform_scale(400., 600., 1.);

        self.right_mesh
            .resource()
            .get_mut()
            .set_transform(right_matrix);
        self.right_material
            .resource()
            .get_mut()
            .set_outline_color([1., 1., 0., 2.].into());

        true
    }
    fn uninit(&mut self) {}
}
