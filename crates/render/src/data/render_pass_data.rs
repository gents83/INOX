use std::path::PathBuf;

use crate::TextureFormat;

#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
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

#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
pub enum LoadOperation {
    Load,
    Clear,
    DontCare,
}
#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
pub enum StoreOperation {
    Store,
    DontCare,
}
#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
pub enum RenderTarget {
    None,
    Screen,
    Texture {
        width: u32,
        height: u32,
        format: TextureFormat,
        sample_count: u32,
        mips_count: u32,
    },
    RWTexture {
        width: u32,
        height: u32,
        format: TextureFormat,
    },
}
#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
pub enum RenderMode {
    Indirect,
    Single,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
