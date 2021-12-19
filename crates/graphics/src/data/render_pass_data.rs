use std::path::PathBuf;

use sabi_serialize::*;

#[derive(Serializable, Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum LoadOperation {
    Load,
    Clear,
    DontCare,
}
#[derive(Serializable, Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum StoreOperation {
    Store,
    DontCare,
}
#[derive(Serializable, Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum RenderTarget {
    Screen,
    Texture,
    TextureAndReadback,
}

#[repr(C)]
#[derive(Serializable, Debug, PartialEq, Clone)]
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
            pipeline: PathBuf::new(),
            mesh_category_to_draw: Vec::new(),
        }
    }
}
