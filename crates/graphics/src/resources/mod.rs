use sabi_resources::SharedDataRc;

use crate::*;

pub use self::font::*;
pub use self::light::*;
pub use self::material::*;
pub use self::mesh::*;
pub use self::pipeline::*;
pub use self::render_pass::*;
pub use self::texture::*;
pub use self::view::*;

pub mod font;
pub mod light;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod texture;
pub mod view;

pub fn register_resource_types(shared_data: &SharedDataRc) {
    shared_data.register_serializable_type::<LightType>();
    shared_data.register_serializable_type::<LightData>();
    shared_data.register_serializable_type::<MaterialAlphaMode>();
    shared_data.register_serializable_type::<MaterialData>();
    shared_data.register_serializable_type::<MeshCategoryId>();
    shared_data.register_serializable_type::<MeshData>();
    shared_data.register_serializable_type::<PolygonModeType>();
    shared_data.register_serializable_type::<CullingModeType>();
    shared_data.register_serializable_type::<BlendFactor>();
    shared_data.register_serializable_type::<DrawMode>();
    shared_data.register_serializable_type::<PipelineType>();
    shared_data.register_serializable_type::<PipelineData>();
    shared_data.register_serializable_type::<LoadOperation>();
    shared_data.register_serializable_type::<StoreOperation>();
    shared_data.register_serializable_type::<RenderTarget>();
    shared_data.register_serializable_type::<RenderPassData>();
    shared_data.register_serializable_type::<TextureType>();
    shared_data.register_serializable_type::<VertexData>();
    shared_data.register_serializable_type::<FontData>();
    shared_data.register_serializable_type::<Line>();
    shared_data.register_serializable_type::<Metrics>();
    shared_data.register_serializable_type::<Glyph>();

    shared_data.register_serializable_resource_type::<Font>();
    shared_data.register_serializable_resource_type::<Material>();
    shared_data.register_serializable_resource_type::<Mesh>();
    shared_data.register_serializable_resource_type::<Pipeline>();
    shared_data.register_resource_type::<RenderPass>();
    shared_data.register_serializable_resource_type::<Texture>();
    shared_data.register_resource_type::<View>();
    shared_data.register_serializable_resource_type::<Light>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_serializable_resource_type::<Font>();
    shared_data.unregister_serializable_resource_type::<Material>();
    shared_data.unregister_serializable_resource_type::<Mesh>();
    shared_data.unregister_serializable_resource_type::<Pipeline>();
    shared_data.unregister_resource_type::<RenderPass>();
    shared_data.unregister_serializable_resource_type::<Texture>();
    shared_data.unregister_resource_type::<View>();
    shared_data.unregister_serializable_resource_type::<Light>();

    shared_data.unregister_serializable_type::<LightType>();
    shared_data.unregister_serializable_type::<LightData>();
    shared_data.unregister_serializable_type::<MaterialAlphaMode>();
    shared_data.unregister_serializable_type::<MaterialData>();
    shared_data.unregister_serializable_type::<MeshCategoryId>();
    shared_data.unregister_serializable_type::<MeshData>();
    shared_data.unregister_serializable_type::<PolygonModeType>();
    shared_data.unregister_serializable_type::<CullingModeType>();
    shared_data.unregister_serializable_type::<BlendFactor>();
    shared_data.unregister_serializable_type::<DrawMode>();
    shared_data.unregister_serializable_type::<PipelineType>();
    shared_data.unregister_serializable_type::<PipelineData>();
    shared_data.unregister_serializable_type::<LoadOperation>();
    shared_data.unregister_serializable_type::<StoreOperation>();
    shared_data.unregister_serializable_type::<RenderTarget>();
    shared_data.unregister_serializable_type::<RenderPassData>();
    shared_data.unregister_serializable_type::<TextureType>();
    shared_data.unregister_serializable_type::<VertexData>();
    shared_data.unregister_serializable_type::<FontData>();
    shared_data.unregister_serializable_type::<Line>();
    shared_data.unregister_serializable_type::<Metrics>();
    shared_data.unregister_serializable_type::<Glyph>();
}
