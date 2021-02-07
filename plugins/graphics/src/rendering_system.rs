use std::sync::Arc;

use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;
use crate::fonts::font::*;
use crate::api::renderer::*;
use crate::api::pipeline::*;


const VS_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\shader_vert.spv";
const FRAG_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\shader_frag.spv";
const IMAGE_PATH: &str = "C:\\PROJECTS\\NRG\\data\\textures\\Test.jpg";

const FONT_VS_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\text_shader_vert.spv";
const FONT_FRAG_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\text_shader_frag.spv";
const FONT_PATH: & str = "C:\\PROJECTS\\NRG\\data\\fonts\\BasicFont.ttf";


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
                let def_rp = renderer.get_default_render_pass().clone();
                let def_pipeline = Pipeline::create(&renderer.device, VS_PATH, FRAG_PATH, def_rp);
                renderer.add_pipeline(pipeline_id.clone(), def_pipeline);
            }
        }
        /*
        {
            let ui_pipeline_id = String::from("UI");
    
            {
                let read_data = self.shared_data.read().unwrap();
                let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
                let def_rp = renderer.get_default_render_pass().clone();
                let ui_pipeline = Pipeline::create(&renderer.device, FONT_VS_PATH, FONT_FRAG_PATH, def_rp);
                renderer.add_pipeline(ui_pipeline_id.clone(), ui_pipeline);
            }
    
            let font = {
                let read_data = self.shared_data.read().unwrap();
                let renderer = &*read_data.get_unique_resource::<Renderer>();
                let ui_pipeline = renderer.get_pipeline(ui_pipeline_id);
                Font::new(&renderer.device, &ui_pipeline, FONT_PATH)
            };
            
    
            self.shared_data.write().unwrap().add_resource(font);
        }
        */
    }

    fn run(&mut self) -> bool {
        //println!("Executing run() for RenderingSystem[{:?}]", self.id());
        
        let read_data = self.shared_data.read().unwrap();
        let mut renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let mut result = renderer.begin_frame();
        
        renderer.process_pipelines();
        
        /*
        let mut font = &mut *read_data.get_unique_resource_mut::<Font>();

        font.get_material().update_simple();

        let str = String::from("Mauro Gentile aka gents");
        let mut signature_mesh = font.create_text( str.as_str(), [-0.9, 0.9].into(), 0.8);
        signature_mesh.set_vertex_color([0.2, 0.6, 1.0].into())
                      .finalize()
                      .draw();
        */
        result = renderer.end_frame();

        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for RenderingSystem[{:?}]", self.id());
    }
}
