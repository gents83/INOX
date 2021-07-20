use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_graphics::{MaterialInstance, MaterialRc, MeshData, MeshInstance, PipelineInstance};
use nrg_math::{get_random_f32, MatBase, Matrix4};

use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_scene::{Object, Scene, SceneRc, Transform};

pub struct UISystem {
    id: SystemId,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    ui_scene: SceneRc,
    ui_default_material: MaterialRc,
}

impl UISystem {
    pub fn new(shared_data: SharedDataRw, job_handler: JobHandlerRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data,
            job_handler,
            ui_scene: SceneRc::default(),
            ui_default_material: MaterialRc::default(),
        }
    }

    fn create_scene(&mut self) -> &mut Self {
        self.ui_scene = SharedData::add_resource::<Scene>(&self.shared_data, Scene::default());
        self
    }

    fn create_default_material(&mut self) -> &mut Self {
        if let Some(pipeline) =
            SharedData::match_resource(&self.shared_data, |p: &PipelineInstance| {
                p.data().name == "UI"
            })
        {
            self.ui_default_material =
                MaterialInstance::create_from_pipeline(&self.shared_data, pipeline);
        } else {
            panic!("No pipeline with name UI has been loaded");
        }
        self
    }

    fn add_2d_quad(&mut self, x: f32, y: f32) -> &mut Self {
        let object = Object::generate_empty(&self.shared_data);

        let transform = object
            .resource()
            .get_mut()
            .add_default_component::<Transform>(&self.shared_data);
        let mat = Matrix4::from_translation([x, y, 0.].into())
            * Matrix4::from_nonuniform_scale(100., 100., 1.);
        transform.resource().get_mut().set_matrix(mat);

        {
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
            mesh_data.set_vertex_color(
                [
                    get_random_f32(0., 1.),
                    get_random_f32(0., 1.),
                    get_random_f32(0., 1.),
                    1.,
                ]
                .into(),
            );
            let mesh = MeshInstance::create_from_data(&self.shared_data, mesh_data);
            self.ui_default_material
                .resource()
                .get_mut()
                .add_mesh(mesh.clone());
            object
                .resource()
                .get_mut()
                .add_component(mesh)
                .add_component(self.ui_default_material.clone());
        }

        self.ui_scene.resource().get_mut().add_object(object);

        self
    }

    fn update_scene(&mut self) -> &mut Self {
        let objects = self.ui_scene.resource().get().get_objects();
        for (i, obj) in objects.iter().enumerate() {
            let job_name = format!("Object[{}]", i);
            let obj = obj.clone();
            let shared_data = self.shared_data.clone();
            self.job_handler
                .write()
                .unwrap()
                .add_job(job_name.as_str(), move || {
                    obj.resource()
                        .get_mut()
                        .update_from_parent(&shared_data, Matrix4::default_identity());
                });
        }

        self
    }
}

impl System for UISystem {
    fn id(&self) -> nrg_core::SystemId {
        self.id
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.create_scene().create_default_material();
        for i in 0..15 {
            for j in 0..10 {
                self.add_2d_quad(i as f32 * 150., j as f32 * 150.);
            }
        }
    }

    fn run(&mut self) -> bool {
        self.update_scene();
        true
    }

    fn uninit(&mut self) {}
}
