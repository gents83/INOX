use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};

use crate::PipelineData;

pub type PipelineId = ResourceId;
pub type PipelineRc = ResourceRef<PipelineInstance>;

pub struct PipelineInstance {
    id: ResourceId,
    data: PipelineData,
    is_initialized: bool,
}

impl Default for PipelineInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            data: PipelineData::default(),
            is_initialized: false,
        }
    }
}

impl ResourceData for PipelineInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for PipelineInstance {
    type DataType = PipelineData;
    fn create_from_data(shared_data: &SharedDataRw, pipeline_data: Self::DataType) -> PipelineRc {
        let canonicalized_pipeline_data = pipeline_data.canonicalize_paths();
        if let Some(pipeline) =
            PipelineInstance::find_from_data(shared_data, &canonicalized_pipeline_data)
        {
            return pipeline;
        }
        SharedData::add_resource(
            shared_data,
            PipelineInstance {
                id: generate_uid_from_string(canonicalized_pipeline_data.name.as_str()),
                data: canonicalized_pipeline_data,
                ..Default::default()
            },
        )
    }
}

impl PipelineInstance {
    pub fn find_from_name(shared_data: &SharedDataRw, pipeline_name: &str) -> Option<PipelineRc> {
        SharedData::match_resource(shared_data, |p: &PipelineInstance| {
            p.data.name == pipeline_name
        })
    }

    fn find_from_data(
        shared_data: &SharedDataRw,
        pipeline_data: &PipelineData,
    ) -> Option<PipelineRc> {
        SharedData::match_resource(shared_data, |p: &PipelineInstance| {
            pipeline_data.has_same_shaders(&p.data) && p.data.name == pipeline_data.name
        })
    }
    pub fn data(&self) -> &PipelineData {
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

    pub fn should_draw_in_render_pass(&self, render_pass: &str) -> bool {
        for name in self.data.render_in_passes.iter() {
            if name == render_pass {
                return true;
            }
        }
        false
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
}
