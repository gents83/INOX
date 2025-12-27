use std::sync::{Arc, RwLock};

use inox_math::{matrix4_to_array, Mat4Ops, MatBase, Matrix4, VecBase, Vector2};
use inox_uid::Uid;

use crate::{
    declare_as_binding, AsBinding, RenderContext, DEFAULT_FOV, DEFAULT_HEIGHT, DEFAULT_WIDTH,
    ENV_MAP_UID, LUT_PBR_CHARLIE_UID, LUT_PBR_GGX_UID,
};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_USE_IBL: u32 = 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1 << 1;
pub const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_LOD_LEVEL: u32 = 1 << 2;
pub const CONSTANT_DATA_FLAGS_DISPLAY_SHADOW: u32 = 1 << 3;
pub const CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER: u32 = 1 << 4;
pub const CONSTANT_DATA_FLAGS_DISPLAY_AO: u32 = 1 << 5;
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
pub const CONSTANT_DATA_FLAGS_DISPLAY_INDIRECT_DIFFUSE: u32 = 1 << 16;
pub const CONSTANT_DATA_FLAGS_DISPLAY_INDIRECT_SPECULAR: u32 = 1 << 17;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ConstantData {
    view: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    inverse_view_proj: [[f32; 4]; 4],
    screen_size: [f32; 2],
    frame_index: u32,
    flags: u32,
    debug_uv_coords: [f32; 2],
    tlas_starting_index: u32,
    num_bounces: u32,
    lut_pbr_charlie_texture_index: u32,
    lut_pbr_ggx_texture_index: u32,
    env_map_texture_index: u32,
    num_lights: u32,
    forced_lod_level: i32,
    camera_near: f32,
    camera_far: f32,
    camera_fov: f32,
}

impl Default for ConstantData {
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
            camera_fov: DEFAULT_FOV,
        }
    }
}
declare_as_binding!(ConstantData);

pub type ConstantDataRw = Arc<RwLock<ConstantData>>;

impl ConstantData {
    pub fn tlas_starting_index(&self) -> u32 {
        self.tlas_starting_index
    }
    pub fn add_flag(&mut self, render_context: &RenderContext, flag: u32) -> &mut Self {
        if self.flags & flag == 0 {
            self.flags |= flag;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn toggle_flag(&mut self, render_context: &RenderContext, flag: u32) -> &mut Self {
        self.flags ^= flag;
        self.mark_as_dirty(render_context);
        self
    }
    pub fn remove_flag(&mut self, render_context: &RenderContext, flag: u32) -> &mut Self {
        if self.flags & flag == flag {
            self.flags &= !flag;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn set_flags(&mut self, render_context: &RenderContext, flags: u32) -> &mut Self {
        self.flags = flags;
        self.mark_as_dirty(render_context);
        self
    }
    pub fn set_frame_index(
        &mut self,
        render_context: &RenderContext,
        frame_index: u32,
    ) -> &mut Self {
        if self.frame_index != frame_index {
            self.frame_index = frame_index;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn frame_index(&self) -> u32 {
        self.frame_index
    }
    pub fn set_num_bounces(&mut self, render_context: &RenderContext, n: u32) -> &mut Self {
        if self.num_bounces != n {
            self.num_bounces = n;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn num_bounces(&self) -> u32 {
        self.num_bounces
    }
    pub fn set_num_lights(&mut self, render_context: &RenderContext, n: u32) -> &mut Self {
        if self.num_lights != n {
            self.num_lights = n;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn num_lights(&self) -> u32 {
        self.num_lights
    }
    pub fn set_forced_lod_level(&mut self, render_context: &RenderContext, n: i32) -> &mut Self {
        if self.forced_lod_level != n {
            self.forced_lod_level = n;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn forced_lod_level(&self) -> i32 {
        self.forced_lod_level
    }
    #[allow(non_snake_case)]
    pub fn set_LUT(
        &mut self,
        render_context: &RenderContext,
        lut_id: &Uid,
        texture_index: u32,
    ) -> &mut Self {
        if *lut_id == LUT_PBR_CHARLIE_UID {
            self.lut_pbr_charlie_texture_index = texture_index;
            self.mark_as_dirty(render_context);
        } else if *lut_id == LUT_PBR_GGX_UID {
            self.lut_pbr_ggx_texture_index = texture_index;
            self.mark_as_dirty(render_context);
        } else if *lut_id == ENV_MAP_UID {
            self.env_map_texture_index = texture_index;
            self.mark_as_dirty(render_context);
        }
        self
    }
    pub fn update(
        &mut self,
        render_context: &RenderContext,
        view_proj_near_far_fov: (Matrix4, Matrix4, f32, f32, f32),
        screen_size_and_debug_coords: (Vector2, Vector2),
        tlas_starting_index: u32,
    ) -> bool {
        let v = matrix4_to_array(view_proj_near_far_fov.0);
        let p = matrix4_to_array(view_proj_near_far_fov.1);
        if self.view != v
            || self.proj != p
            || self.screen_size[0] != screen_size_and_debug_coords.0.x
            || self.screen_size[1] != screen_size_and_debug_coords.0.y
        {
            self.frame_index = 0;
        }
        self.view = v;
        self.proj = p;
        self.camera_near = view_proj_near_far_fov.2;
        self.camera_far = view_proj_near_far_fov.3;
        self.camera_fov = view_proj_near_far_fov.4;
        self.view_proj = matrix4_to_array(view_proj_near_far_fov.1 * view_proj_near_far_fov.0);
        self.inverse_view_proj =
            matrix4_to_array((view_proj_near_far_fov.1 * view_proj_near_far_fov.0).inverse());
        self.screen_size = screen_size_and_debug_coords.0.into();
        self.debug_uv_coords = (screen_size_and_debug_coords
            .1
            .div(screen_size_and_debug_coords.0))
        .into();
        self.tlas_starting_index = tlas_starting_index;
        self.mark_as_dirty(render_context);
        true
    }
    pub fn view(&self) -> [[f32; 4]; 4] {
        self.view
    }
    pub fn proj(&self) -> [[f32; 4]; 4] {
        self.proj
    }
    pub fn inverse_view_proj(&self) -> [[f32; 4]; 4] {
        self.inverse_view_proj
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
}
