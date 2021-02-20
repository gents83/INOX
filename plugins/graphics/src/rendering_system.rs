use crate::api::renderer::*;
use crate::config::*;

use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;

pub struct RenderingSystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
}

impl RenderingSystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
        }
    }
}

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        {
            let renderer = {
                let read_data = self.shared_data.read().unwrap();
                let window = &*read_data.get_unique_resource::<Window>();
                let mut renderer = Renderer::new(window.get_handle(), &self.config);
                let size = Vector2u::new(window.get_width(), window.get_heigth());
                renderer.set_viewport_size(size);
                renderer
            };
            self.shared_data.write().unwrap().add_resource(renderer);
        }

        {
            let pipeline_id = String::from("Default");
            {
                let read_data = self.shared_data.read().unwrap();
                let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
                let pipeline_data = self.config.get_pipeline_data(pipeline_id).unwrap();
                renderer.add_pipeline(pipeline_data);
            }
        }
    }

    fn run(&mut self) -> bool {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let mut _result = renderer.begin_frame();
        renderer.draw();
        _result = renderer.end_frame();

        true
    }
    fn uninit(&mut self) {
        self.shared_data
            .write()
            .unwrap()
            .remove_resources_of_type::<Renderer>();
    }
}
