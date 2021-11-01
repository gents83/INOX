#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeshDataRef {
    pub first_vertex: u32,
    pub last_vertex: u32,
    pub first_index: u32,
    pub last_index: u32,
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct InstanceCommand {
    pub mesh_index: usize,
    pub mesh_data_ref: MeshDataRef,
}
