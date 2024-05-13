pub const INVALID_INDEX: i32 = -1;

pub use binding_data::*;
pub use compute_pass_data::*;
pub use compute_pipeline_data::*;
pub use constant_data::*;
pub use gpu_data::*;
use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;
pub use light_data::*;
pub use material_data::*;
pub use mesh_data::*;
pub use render_pass_data::*;
pub use render_pipeline_data::*;
pub use shader_data::*;
pub use texture_data::*;
pub use vertex_data::*;

pub mod binding_data;
pub mod compute_pass_data;
pub mod compute_pipeline_data;
pub mod constant_data;
pub mod gpu_data;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod render_pass_data;
pub mod render_pipeline_data;
pub mod shader_data;
pub mod texture_data;
pub mod vertex_data;

pub fn register_gpu_data_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.register_type::<GPUTexture>(message_hub);
}

pub fn unregister_gpu_data_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.unregister_type::<GPUTexture>(message_hub);
}

#[macro_export]
macro_rules! print_field_size {
    ($Expected_offset:expr, $Field:ident, $Field_type:ty, $Number:expr) => {
        let offset: usize = unsafe { &(*(::std::ptr::null::<Self>())).$Field as *const _ as usize };
        let size: usize = std::mem::size_of::<$Field_type>();
        let typename = std::any::type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        println!(
            "{}.{} | offset [{}] | size {}x{}=[{}] | next_expected_offset = [{}]",
            typename,
            stringify!($Field),
            offset,
            size,
            $Number,
            size * $Number,
            offset + (size * $Number)
        );
        $Expected_offset += (size * $Number)
    };
}
