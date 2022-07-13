use std::path::PathBuf;

use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::TextureFormat;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum IndexFormat {
    U32,
    U16,
}
impl From<IndexFormat> for wgpu::IndexFormat {
    fn from(val: IndexFormat) -> Self {
        match val {
            IndexFormat::U32 => wgpu::IndexFormat::Uint32,
            IndexFormat::U16 => wgpu::IndexFormat::Uint16,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum LoadOperation {
    Load,
    Clear,
    DontCare,
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum StoreOperation {
    Store,
    DontCare,
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "inox_serialize", tag = "type")]
pub enum RenderTarget {
    None,
    Screen,
    Texture {
        width: u32,
        height: u32,
        format: TextureFormat,
        read_back: bool,
    },
}
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum RenderMode {
    Indirect,
    Single,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct RenderPassData {
    pub name: String,
    pub load_color: LoadOperation,
    pub store_color: StoreOperation,
    pub load_depth: LoadOperation,
    pub store_depth: StoreOperation,
    pub render_target: RenderTarget,
    pub depth_target: RenderTarget,
    pub render_mode: RenderMode,
    pub pipeline: PathBuf,
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
            depth_target: RenderTarget::None,
            render_mode: RenderMode::Indirect,
            pipeline: PathBuf::new(),
        }
    }
}
