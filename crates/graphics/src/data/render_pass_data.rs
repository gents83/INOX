use std::path::PathBuf;

use sabi_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum LoadOperation {
    Load,
    Clear,
    DontCare,
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum StoreOperation {
    Store,
    DontCare,
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize", tag = "type")]
pub enum RenderTarget {
    Screen,
    Texture {
        width: u32,
        height: u32,
        read_back: bool,
    },
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum RenderMode {
    Indirect,
    Single,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct RenderPassData {
    pub name: String,
    pub load_color: LoadOperation,
    pub store_color: StoreOperation,
    pub load_depth: LoadOperation,
    pub store_depth: StoreOperation,
    pub render_target: RenderTarget,
    pub render_mode: RenderMode,
    pub pipelines: Vec<PathBuf>,
}

impl SerializeFile for RenderPassData {
    fn extension() -> &'static str {
        "render_pass"
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
            render_mode: RenderMode::Indirect,
            pipelines: Vec::new(),
        }
    }
}
