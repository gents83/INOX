use std::{
    mem::size_of,
    sync::{Arc, RwLock},
};

use inox_math::{matrix4_to_array, Mat4Ops, MatBase, Matrix4, VecBase, Vector2};
use inox_uid::Uid;

use crate::{
    AsBinding, GpuBuffer, RenderContext, DEFAULT_HEIGHT, DEFAULT_WIDTH, ENV_MAP_UID,
    LUT_PBR_CHARLIE_UID, LUT_PBR_GGX_UID,
};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_USE_IBL: u32 = 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1 << 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 1 << 2;
pub const CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER: u32 = 1 << 3;
pub const CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER: u32 = 1 << 4;
pub const CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE: u32 = 1 << 5;
pub const CONSTANT_DATA_FLAGS_DISPLAY_NORMALS: u32 = 1 << 6;
pub const CONSTANT_DATA_FLAGS_DISPLAY_TANGENT: u32 = 1 << 7;
pub const CONSTANT_DATA_FLAGS_DISPLAY_BITANGENT: u32 = 1 << 8;
pub const CONSTANT_DATA_FLAGS_DISPLAY_BASE_COLOR: u32 = 1 << 9;
pub const CONSTANT_DATA_FLAGS_DISPLAY_METALLIC: u32 = 1 << 10;
pub const CONSTANT_DATA_FLAGS_DISPLAY_ROUGHNESS: u32 = 1 << 11;
pub const CONSTANT_DATA_FLAGS_DISPLAY_UV_0: u32 = 1 << 12;
pub const CONSTANT_DATA_FLAGS_DISPLAY_UV_1: u32 = 1 << 13;
pub const CONSTANT_DATA_FLAGS_DISPLAY_UV_2: u32 = 1 << 14;
pub const CONSTANT_DATA_FLAGS_DISPLAY_UV_3: u32 = 1 << 15;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Data {
    pub view: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub view_proj: [[f32; 4]; 4],
    pub inverse_view_proj: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub frame_index: u32,
    pub flags: u32,
    pub debug_uv_coords: [f32; 2],
    pub tlas_starting_index: u32,
    pub num_bounces: u32,
    pub lut_pbr_charlie_texture_index: u32,
    pub lut_pbr_ggx_texture_index: u32,
    pub env_map_texture_index: u32,
    pub num_lights: u32,
    pub forced_lod_level: i32,
    pub camera_near: f32,
    pub camera_far: f32,
    pub _empty3: u32,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            view: Matrix4::default_identity().into(),
            inv_view: Matrix4::default_identity().into(),
            proj: Matrix4::default_identity().into(),
            view_proj: Matrix4::default_identity().into(),
            inverse_view_proj: Matrix4::default_identity().into(),
            screen_size: [DEFAULT_WIDTH as _, DEFAULT_HEIGHT as _],
            frame_index: 0,
            flags: 0,
            debug_uv_coords: [0.; 2],
            tlas_starting_index: 0,
            num_bounces: 0,
            lut_pbr_charlie_texture_index: 0,
            lut_pbr_ggx_texture_index: 0,
            env_map_texture_index: 0,
            num_lights: 0,
            forced_lod_level: -1,
            camera_near: 0.,
            camera_far: 0.,
            _empty3: 0,
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
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_context, &[self.data]);
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
    pub fn set_num_lights(&mut self, n: u32) -> &mut Self {
        if self.data.num_lights != n {
            self.data.num_lights = n;
            self.set_dirty(true);
        }
        self
    }
    pub fn num_lights(&self) -> u32 {
        self.data.num_lights
    }
    pub fn set_forced_lod_level(&mut self, n: i32) -> &mut Self {
        if self.data.forced_lod_level != n {
            self.data.forced_lod_level = n;
            self.set_dirty(true);
        }
        self
    }
    pub fn forced_lod_level(&self) -> i32 {
        self.data.forced_lod_level
    }
    #[allow(non_snake_case)]
    pub fn set_LUT(&mut self, lut_id: &Uid, texture_index: u32) -> &mut Self {
        if *lut_id == LUT_PBR_CHARLIE_UID {
            self.data.lut_pbr_charlie_texture_index = texture_index;
            self.set_dirty(true);
        } else if *lut_id == LUT_PBR_GGX_UID {
            self.data.lut_pbr_ggx_texture_index = texture_index;
            self.set_dirty(true);
        } else if *lut_id == ENV_MAP_UID {
            self.data.env_map_texture_index = texture_index;
            self.set_dirty(true);
        }
        self
    }
    pub fn update(
        &mut self,
        view_proj_near_far: (Matrix4, Matrix4, f32, f32),
        screen_size_and_debug_coords: (Vector2, Vector2),
        tlas_starting_index: u32,
    ) -> bool {
        let v = matrix4_to_array(view_proj_near_far.0);
        let p = matrix4_to_array(view_proj_near_far.1);
        if self.data.view != v
            || self.data.proj != p
            || self.data.screen_size[0] != screen_size_and_debug_coords.0.x
            || self.data.screen_size[1] != screen_size_and_debug_coords.0.y
        {
            self.data.frame_index = 0;
        }
        self.data.view = v;
        self.data.proj = p;
        self.data.camera_near = view_proj_near_far.2;
        self.data.camera_far = view_proj_near_far.3;
        self.data.view_proj = matrix4_to_array(view_proj_near_far.1 * view_proj_near_far.0);
        self.data.inverse_view_proj =
            matrix4_to_array((view_proj_near_far.1 * view_proj_near_far.0).inverse());
        self.data.screen_size = screen_size_and_debug_coords.0.into();
        self.data.debug_uv_coords = (screen_size_and_debug_coords
            .1
            .div(screen_size_and_debug_coords.0))
        .into();
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
    pub fn proj(&self) -> [[f32; 4]; 4] {
        self.data.proj
    }
    pub fn inverse_view_proj(&self) -> [[f32; 4]; 4] {
        self.data.inverse_view_proj
    }
}
