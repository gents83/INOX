use std::{
    mem::size_of,
    sync::{Arc, RwLock},
};

use inox_math::{matrix4_to_array, Mat4Ops, Matrix4, Vector2};

use crate::{AsBinding, GpuBuffer, RenderCoreContext};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1 << 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 1 << 2;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 1 << 3;

#[repr(C, align(16))]
#[derive(Default, Debug, Clone, Copy)]
struct Data {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub inverse_view_proj: [[f32; 4]; 4],
    pub screen_width: f32,
    pub screen_height: f32,
    pub frame_index: u32,
    pub flags: u32,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ConstantData {
    is_dirty: bool,
    data: Data,
}

pub type ConstantDataRw = Arc<RwLock<ConstantData>>;

impl AsBinding for ConstantData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        size_of::<Data>() as _
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.data]);
    }
}

impl ConstantData {
    pub fn add_flag(&mut self, flag: u32) -> &mut Self {
        if self.data.flags & flag == 0 {
            self.data.flags |= flag;
            self.set_dirty(true);
        }
        self
    }
    pub fn toggle_flag(&mut self, flag: u32) -> &mut Self {
        self.data.flags ^= flag;
        self.set_dirty(true);
        self
    }
    pub fn remove_flag(&mut self, flag: u32) -> &mut Self {
        if self.data.flags & flag == flag {
            self.data.flags &= !flag;
            self.set_dirty(true);
        }
        self
    }
    pub fn set_flags(&mut self, flags: u32) -> &mut Self {
        if self.data.flags != flags {
            self.data.flags = flags;
            self.set_dirty(true);
        }
        self
    }
    pub fn update(&mut self, view: Matrix4, proj: Matrix4, screen_size: Vector2) -> bool {
        let v = matrix4_to_array(view);
        let p = matrix4_to_array(proj);
        if self.data.view != v
            || self.data.proj != p
            || self.data.screen_width != screen_size.x
            || self.data.screen_height != screen_size.y
        {
            self.data.view = v;
            self.data.proj = p;
            self.data.frame_index = 0;
            self.data.inverse_view_proj = matrix4_to_array((proj * view).inverse());
            self.data.screen_width = screen_size.x;
            self.data.screen_height = screen_size.y;
            self.set_dirty(true);
        }
        self.data.frame_index += 1;
        self.set_dirty(true);
        self.is_dirty()
    }
    pub fn view(&self) -> [[f32; 4]; 4] {
        self.data.view
    }
}
