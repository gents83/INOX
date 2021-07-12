use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_uid_from_string, Uid, INVALID_UID};

use crate::RenderPassData;

pub type RenderPassId = Uid;
pub type RenderPassRc = ResourceRef<RenderPassInstance>;

pub struct RenderPassInstance {
    id: ResourceId,
    data: RenderPassData,
    is_initialized: bool,
}

impl ResourceData for RenderPassInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for RenderPassInstance {
    type DataType = RenderPassData;
    fn create_from_data(
        shared_data: &SharedDataRw,
        render_pass_data: Self::DataType,
    ) -> RenderPassRc {
        let render_pass_id =
            RenderPassInstance::find_id_from_name(shared_data, render_pass_data.name.as_str());
        if render_pass_id != INVALID_UID {
            return SharedData::get_resource::<Self>(shared_data, render_pass_id);
        }
        SharedData::add_resource(
            shared_data,
            RenderPassInstance {
                id: generate_uid_from_string(render_pass_data.name.as_str()),
                data: render_pass_data.clone(),
                is_initialized: false,
            },
        )
    }
}

impl RenderPassInstance {
    pub fn find_id_from_name(shared_data: &SharedDataRw, render_pass_name: &str) -> RenderPassId {
        SharedData::match_resource(shared_data, |r: &RenderPassInstance| {
            r.data.name == render_pass_name
        })
    }
    pub fn data(&self) -> &RenderPassData {
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
}
