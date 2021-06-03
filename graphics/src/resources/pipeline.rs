use std::path::PathBuf;

use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::{generate_uid_from_string, Uid, INVALID_UID};

use crate::PipelineData;

pub type PipelineId = Uid;

pub struct PipelineInstance {
    id: ResourceId,
    data: PipelineData,
    is_initialized: bool,
}

impl ResourceTrait for PipelineInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        PathBuf::from(self.data.name.as_str())
    }
}

impl PipelineInstance {
    pub fn find_id_from_name(shared_data: &SharedDataRw, pipeline_name: &str) -> PipelineId {
        SharedData::match_resource(shared_data, |p: &PipelineInstance| {
            p.data.name == pipeline_name
        })
    }

    fn find_id_from_data(shared_data: &SharedDataRw, pipeline_data: &PipelineData) -> PipelineId {
        SharedData::match_resource(shared_data, |p: &PipelineInstance| {
            pipeline_data.has_same_shaders(&p.data)
        })
    }
    pub fn get_data(&self) -> &PipelineData {
        &self.data
    }
    pub fn init(&mut self) -> &mut Self {
        self.is_initialized = true;
        self
    }

    pub fn invalidate(&mut self) {
        self.is_initialized = false;
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if path_as_string.contains(self.data.vertex_shader.to_str().unwrap())
            && !self.data.vertex_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!("VertexShader {:?} will be reloaded", path_as_string);
        }
        if path_as_string.contains(self.data.fragment_shader.to_str().unwrap())
            && !self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!("FragmentShader {:?} will be reloaded", path_as_string);
        }
        if path_as_string.contains(self.data.tcs_shader.to_str().unwrap())
            && !self.data.tcs_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!(
                "TessellationControlShader {:?} will be reloaded",
                path_as_string
            );
        }
        if path_as_string.contains(self.data.tes_shader.to_str().unwrap())
            && !self.data.tes_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!(
                "TessellationEvaluationShader {:?} will be reloaded",
                path_as_string
            );
        }
        if path_as_string.contains(self.data.geometry_shader.to_str().unwrap())
            && !self.data.geometry_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!("GeometryShader {:?} will be reloaded", path_as_string);
        }
    }

    pub fn create(shared_data: &SharedDataRw, pipeline_data: &PipelineData) -> PipelineId {
        let canonicalized_pipeline_data = pipeline_data.clone().canonicalize_paths();
        let pipeline_id =
            PipelineInstance::find_id_from_data(shared_data, &canonicalized_pipeline_data);
        if pipeline_id != INVALID_UID {
            return pipeline_id;
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(PipelineInstance {
            id: generate_uid_from_string(canonicalized_pipeline_data.name.as_str()),
            data: canonicalized_pipeline_data,
            is_initialized: false,
        })
    }
}
