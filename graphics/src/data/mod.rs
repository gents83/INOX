pub const INVALID_INDEX: i32 = -1;

pub use constant_data::*;
pub use instance_command::*;
pub use instance_data::*;
pub use light_data::*;
pub use material_data::*;
pub use mesh_data::*;
pub use pipeline_data::*;
pub use render_pass_data::*;
pub use shader_data::*;
pub use shader_material_data::*;
pub use shader_texture_data::*;
pub use texture_data::*;
pub use vertex_data::*;

pub mod constant_data;
pub mod instance_command;
pub mod instance_data;
pub mod light_data;
pub mod material_data;
pub mod mesh_data;
pub mod pipeline_data;
pub mod render_pass_data;
pub mod shader_data;
pub mod shader_material_data;
pub mod shader_texture_data;
pub mod texture_data;
pub mod vertex_data;

#[macro_export]
macro_rules! print_field_size {
    ($Expected_offset:expr, $Field:ident, $Field_type:ty, $Number:expr) => {
        let offset: usize = unsafe { &(*(::std::ptr::null::<Self>())).$Field as *const _ as usize };
        let size: usize = std::mem::size_of::<$Field_type>();
        let result: &str = if $Expected_offset == offset && offset % 4 == 0 {
            "OK"
        } else {
            "TO ALIGN"
        };
        let typename = std::any::type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        println!(
            "{}.{} | offset {} [{}] | size {}x{}=[{}] | next_expected_offset = [{}]",
            typename,
            stringify!($Field),
            result,
            offset,
            size,
            $Number,
            size * $Number,
            offset + (size * $Number)
        );
        $Expected_offset = $Expected_offset + (size * $Number)
    };
}
