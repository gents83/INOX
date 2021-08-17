use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_uid_from_string, Uid, INVALID_UID};

use crate::{RenderPassData, TextureRc};

pub type RenderPassId = Uid;
pub type RenderPassRc = ResourceRef<RenderPassInstance>;

pub struct RenderPassInstance {
    id: ResourceId,
    data: RenderPassData,
    color_texture: Option<TextureRc>,
    depth_texture: Option<TextureRc>,
    is_initialized: bool,
}

impl Default for RenderPassInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            data: RenderPassData::default(),
            color_texture: None,
            depth_texture: None,
            is_initialized: false,
        }
    }
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
        if let Some(render_pass) =
            RenderPassInstance::find_from_name(shared_data, render_pass_data.name.as_str())
        {
            return render_pass;
        }

        SharedData::add_resource(
            shared_data,
            RenderPassInstance {
                id: generate_uid_from_string(render_pass_data.name.as_str()),
                data: render_pass_data.clone(),
                ..Default::default()
            },
        )
    }
}

impl RenderPassInstance {
    pub fn find_from_name(
        shared_data: &SharedDataRw,
        render_pass_name: &str,
    ) -> Option<RenderPassRc> {
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
    pub fn color_texture(&self) -> Option<TextureRc> {
        self.color_texture.clone()
    }
    pub fn depth_texture(&self) -> Option<TextureRc> {
        self.depth_texture.clone()
    }
    pub fn reset_color_texture(&mut self) {
        self.color_texture = None;
    }
    pub fn reset_depth_texture(&mut self) {
        self.depth_texture = None;
    }
    pub fn set_color_texture(&mut self, color_texture: TextureRc) {
        self.color_texture = Some(color_texture);
        self.invalidate();
    }
    pub fn set_depth_texture(&mut self, depth_texture: TextureRc) {
        self.depth_texture = Some(depth_texture);
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.is_initialized = false;
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
