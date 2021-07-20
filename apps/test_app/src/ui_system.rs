use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_graphics::{MaterialInstance, MeshData, MeshInstance, PipelineInstance};
use nrg_math::{MatBase, Matrix4};

use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_scene::{Object, Scene, SceneRc, Transform};

pub struct UISystem {
    id: SystemId,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    ui_scene: SceneRc,
}

impl UISystem {
    pub fn new(shared_data: SharedDataRw, job_handler: JobHandlerRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data,
            job_handler,
            ui_scene: SceneRc::default(),
        }
    }

    fn create_scene(&mut self) -> &mut Self {
        self.ui_scene = SharedData::add_resource::<Scene>(&self.shared_data, Scene::default());
        self
    }

    fn add_2d_quad(&mut self) -> &mut Self {
        let object = SharedData::add_resource::<Object>(&self.shared_data, Object::default());

        let transform = object
            .resource()
            .get_mut()
            .add_default_component::<Transform>(&self.shared_data);
        let mat = Matrix4::from_translation([100., 100., 0.].into())
            * Matrix4::from_nonuniform_scale(1000., 1000., 1.);
        transform.resource().get_mut().set_matrix(mat);

        if let Some(pipeline) =
            SharedData::match_resource(&self.shared_data, |p: &PipelineInstance| {
                p.data().name == "UI"
            })
        {
            let material = MaterialInstance::create_from_pipeline(&self.shared_data, pipeline);
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
            mesh_data.set_vertex_color([1., 0., 0., 1.].into());
            let mesh = MeshInstance::create_from_data(&self.shared_data, mesh_data);
            material.resource().get_mut().add_mesh(mesh);
            object.resource().get_mut().add_component(material);
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
        self.create_scene().add_2d_quad();
    }

    fn run(&mut self) -> bool {
        self.update_scene();
        true
    }

    fn uninit(&mut self) {}
}
