#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RayPackedData {
    pub origin_tmin: [f32; 4],
    pub direction_tmax: [f32; 4],
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RadiancePackedData {
    pub data: [f32; 4],
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ThroughputPackedData {
    pub data: [f32; 4],
}
