use std::path::PathBuf;

use nrg_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "nrg_serialize")]
pub enum LoadOperation {
    Load,
    Clear,
    DontCare,
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "nrg_serialize")]
pub enum StoreOperation {
    Store,
    DontCare,
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "nrg_serialize")]
pub enum RenderTarget {
    Screen,
    Texture,
    TextureAndReadback,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct RenderPassData {
    pub name: String,
    pub load_color: LoadOperation,
    pub store_color: StoreOperation,
    pub load_depth: LoadOperation,
    pub store_depth: StoreOperation,
    pub render_target: RenderTarget,
    pub pipeline: PathBuf,
    pub mesh_category_to_draw: Vec<String>,
}

impl SerializeFile for RenderPassData {
    fn extension() -> &'static str {
        "render_pass_data"
    }
}

unsafe impl Send for RenderPassData {}
unsafe impl Sync for RenderPassData {}

impl Default for RenderPassData {
    fn default() -> Self {
        Self {
            name: String::new(),
            load_color: LoadOperation::Clear,
            store_color: StoreOperation::DontCare,
            load_depth: LoadOperation::Clear,
            store_depth: StoreOperation::DontCare,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::new(),
            mesh_category_to_draw: Vec::new(),
        }
    }
}
