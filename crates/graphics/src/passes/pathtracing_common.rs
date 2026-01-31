use inox_serialize::{Deserialize, Serialize};
use inox_uid::{generate_static_uid_from_string, Uid};

pub const NEXT_RAYS_UID: Uid = generate_static_uid_from_string("NEXT_RAYS_BUFFER");

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct Ray {
    pub origin: [f32; 3],
    pub t_min: f32,
    pub direction: [f32; 3],
    pub t_max: f32,
    pub throughput: [f32; 3],
    pub pixel_index: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct RayHit {
    pub instance_id: u32,
    pub primitive_index: u32,
    pub barycentrics: [f32; 2],
    pub t: f32,
    pub pixel_index: u32,
    pub padding: [u32; 2],
    pub direction: [f32; 3],
    pub pad_dir: f32,
    pub throughput: [f32; 3],
    pub pad_thr: f32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct ShadowRay {
    pub origin: [f32; 3],
    pub t_max: f32,
    pub direction: [f32; 3],
    pub radiance: u32, // Packed RGBA or similar? Or just throughput?
                       // Actually we usually accumulate radiance in the Shade pass if we know the light.
                       // But for "Shadow Ray", we check visibility. If visible, we add contribution.
                       // So we need to store the "Potential Contribution" (throughput * light_radiance * brdf * ndotl).
                       // Let's store it as vec3.
    pub contribution: [f32; 3],
    pub pixel_index: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct PathTracingCounters {
    pub ray_count: u32,
    pub hit_count: u32,
    pub shadow_ray_count: u32,
    pub extension_ray_count: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RadiancePackedData(pub f32);
