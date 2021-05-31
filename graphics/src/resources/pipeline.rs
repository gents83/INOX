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

    pub fn find_id_from_data(
        shared_data: &SharedDataRw,
        pipeline_data: &PipelineData,
    ) -> PipelineId {
        SharedData::match_resource(shared_data, |p: &PipelineInstance| {
            p.data.fragment_shader == pipeline_data.fragment_shader
                && p.data.vertex_shader == pipeline_data.vertex_shader
        })
    }
    pub fn get_data(&self) -> &PipelineData {
        &self.data
    }
    pub fn init(&mut self) -> &mut Self {
        self.is_initialized = true;
        self
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn create(shared_data: &SharedDataRw, pipeline_data: &PipelineData) -> PipelineId {
        let pipeline_id = PipelineInstance::find_id_from_data(shared_data, pipeline_data);
        if pipeline_id != INVALID_UID {
            return pipeline_id;
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(PipelineInstance {
            id: generate_uid_from_string(pipeline_data.name.as_str()),
            data: pipeline_data.clone(),
            is_initialized: false,
        })
    }
}
