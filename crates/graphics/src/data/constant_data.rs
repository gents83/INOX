use std::{
    mem::size_of,
    sync::{Arc, RwLock},
};

use inox_math::{matrix4_to_array, Mat4Ops, MatBase, Matrix4, VecBase, Vector2};

use crate::{AsBinding, GpuBuffer, RenderCoreContext, DEFAULT_HEIGHT, DEFAULT_WIDTH};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1 << 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 1 << 2;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_CONE_AXIS: u32 = 1 << 3;
pub const CONSTANT_DATA_FLAGS_DISPLAY_VISIBILITY_BUFFER: u32 = 1 << 4;
pub const CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER: u32 = 1 << 5;
pub const CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER: u32 = 1 << 6;
pub const CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE: u32 = 1 << 7;
pub const CONSTANT_DATA_FLAGS_USE_IBL: u32 = 1 << 8;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Data {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub inverse_view_proj: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub frame_index: u32,
    pub flags: u32,
    pub debug_uv_coords: [f32; 2],
    pub tlas_starting_index: u32,
    pub num_bounces: u32,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            view: Matrix4::default_identity().into(),
            proj: Matrix4::default_identity().into(),
            inverse_view_proj: Matrix4::default_identity().into(),
            screen_size: [DEFAULT_WIDTH as _, DEFAULT_HEIGHT as _],
            frame_index: 0,
            flags: 0,
            debug_uv_coords: [0.; 2],
            tlas_starting_index: 0,
            num_bounces: 5,
        }
    }
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
        self.data.flags = flags;
        self.set_dirty(true);
        self
    }
    pub fn set_frame_index(&mut self, frame_index: u32) -> &mut Self {
        if self.data.frame_index != frame_index {
            self.data.frame_index = frame_index;
            self.set_dirty(true);
        }
        self
    }
    pub fn frame_index(&self) -> u32 {
        self.data.frame_index
    }
    pub fn set_num_bounces(&mut self, n: u32) -> &mut Self {
        if self.data.num_bounces != n {
            self.data.num_bounces = n;
            self.set_dirty(true);
        }
        self
    }
    pub fn num_bounces(&self) -> u32 {
        self.data.num_bounces
    }
    pub fn update(
        &mut self,
        view: Matrix4,
        proj: Matrix4,
        screen_size: Vector2,
        debug_coords: Vector2,
        tlas_starting_index: u32,
    ) -> bool {
        let v = matrix4_to_array(view);
        let p = matrix4_to_array(proj);
        if self.data.view != v
            || self.data.proj != p
            || self.data.screen_size[0] != screen_size.x
            || self.data.screen_size[1] != screen_size.y
        {
            self.data.frame_index = 0;
        }
        self.data.view = v;
        self.data.proj = p;
        self.data.inverse_view_proj = matrix4_to_array((proj * view).inverse());
        self.data.screen_size = screen_size.into();
        self.data.debug_uv_coords = (debug_coords.div(screen_size)).into();
        self.data.tlas_starting_index = tlas_starting_index;
        if self.data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE == 0 {
            self.data.frame_index += 1;
        }
        self.set_dirty(true);
        self.is_dirty()
    }
    pub fn view(&self) -> [[f32; 4]; 4] {
        self.data.view
    }
}
