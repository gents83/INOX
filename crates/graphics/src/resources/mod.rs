use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

pub use self::font::*;
pub use self::light::*;
pub use self::material::*;
pub use self::mesh::*;
pub use self::pipeline::*;
pub use self::render_pass::*;
pub use self::shader::*;
pub use self::texture::*;
pub use self::view::*;

pub mod font;
pub mod light;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod texture;
pub mod view;

pub fn register_resource_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.register_type_serializable::<Font>(message_hub);
    shared_data.register_type_serializable::<Material>(message_hub);
    shared_data.register_type_serializable::<Mesh>(message_hub);
    shared_data.register_type_serializable::<Pipeline>(message_hub);
    shared_data.register_type_serializable::<Shader>(message_hub);
    shared_data.register_type::<RenderPass>();
    shared_data.register_type_serializable::<Texture>(message_hub);
    shared_data.register_type::<View>();
    shared_data.register_type_serializable::<Light>(message_hub);
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_type_serializable::<Font>();
    shared_data.unregister_type_serializable::<Material>();
    shared_data.unregister_type_serializable::<Mesh>();
    shared_data.unregister_type_serializable::<Pipeline>();
    shared_data.unregister_type_serializable::<Shader>();
    shared_data.unregister_type::<RenderPass>();
    shared_data.unregister_type_serializable::<Texture>();
    shared_data.unregister_type::<View>();
    shared_data.unregister_type_serializable::<Light>();
}