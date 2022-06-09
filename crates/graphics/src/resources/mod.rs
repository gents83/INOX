#![allow(dead_code)]

use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

pub use self::compute_pass::*;
pub use self::compute_pipeline::*;
pub use self::font::*;
pub use self::light::*;
pub use self::material::*;
pub use self::mesh::*;
pub use self::render_pass::*;
pub use self::render_pipeline::*;
pub use self::shader::*;
pub use self::texture::*;
pub use self::view::*;

pub mod compute_pass;
pub mod compute_pipeline;
pub mod font;
pub mod light;
pub mod material;
pub mod mesh;
pub mod render_pass;
pub mod render_pipeline;
pub mod shader;
pub mod texture;
pub mod view;

pub fn register_resource_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.register_type_serializable::<Font>(message_hub);
    shared_data.register_type_serializable::<Material>(message_hub);
    shared_data.register_type_serializable::<Mesh>(message_hub);
    shared_data.register_type_serializable::<ComputePipeline>(message_hub);
    shared_data.register_type_serializable::<RenderPipeline>(message_hub);
    shared_data.register_type_serializable::<Shader>(message_hub);
    shared_data.register_type::<ComputePass>(message_hub);
    shared_data.register_type::<RenderPass>(message_hub);
    shared_data.register_type_serializable::<Texture>(message_hub);
    shared_data.register_type::<View>(message_hub);
    shared_data.register_type_serializable::<Light>(message_hub);
}

pub fn unregister_resource_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.unregister_type_serializable::<Light>(message_hub);
    shared_data.unregister_type::<View>(message_hub);
    shared_data.unregister_type_serializable::<Texture>(message_hub);
    shared_data.unregister_type::<RenderPass>(message_hub);
    shared_data.unregister_type::<ComputePass>(message_hub);
    shared_data.unregister_type_serializable::<Shader>(message_hub);
    shared_data.unregister_type_serializable::<RenderPipeline>(message_hub);
    shared_data.unregister_type_serializable::<ComputePipeline>(message_hub);
    shared_data.unregister_type_serializable::<Mesh>(message_hub);
    shared_data.unregister_type_serializable::<Material>(message_hub);
    shared_data.unregister_type_serializable::<Font>(message_hub);
}
