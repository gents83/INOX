use crate::api::pipeline::*;
use crate::api::render_pass::*;
use crate::api::renderer::*;

use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;

const VS_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\shader_vert.spv";
const FRAG_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\shader_frag.spv";

pub struct RenderingSystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl RenderingSystem {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
        }
    }
}

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        //println!("Executing init() for RenderingSystem[{:?}]", self.id());
        {
            let renderer = {
                let read_data = self.shared_data.read().unwrap();
                let window = &*read_data.get_unique_resource::<Window>();
                let mut renderer = Renderer::new(window.get_handle(), false);
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
                let def_rp = RenderPass::create_default(&renderer.device);
                let def_pipeline = Pipeline::create(&renderer.device, VS_PATH, FRAG_PATH, def_rp);
                renderer.add_pipeline(pipeline_id, def_pipeline);
            }
        }
    }

    fn run(&mut self) -> bool {
        //println!("Executing run() for RenderingSystem[{:?}]", self.id());
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let mut result = renderer.begin_frame();
        renderer.process_pipelines();
        result = renderer.end_frame();

        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for RenderingSystem[{:?}]", self.id());
            self.shared_data.write().unwrap().remove_resources_of_type::<Renderer>();
    }
}
