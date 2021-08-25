use nrg_resources::SharedDataRw;

pub use self::font::*;
pub use self::material::*;
pub use self::mesh::*;
pub use self::pipeline::*;
pub use self::render_pass::*;
pub use self::texture::*;
pub use self::view::*;

pub mod font;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod texture;
pub mod view;

pub fn register_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.register_type::<FontInstance>();
    shared_data.register_type::<MaterialInstance>();
    shared_data.register_type::<MeshInstance>();
    shared_data.register_type::<PipelineInstance>();
    shared_data.register_type::<RenderPassInstance>();
    shared_data.register_type::<TextureInstance>();
    shared_data.register_type::<ViewInstance>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.unregister_type::<FontInstance>();
    shared_data.unregister_type::<MaterialInstance>();
    shared_data.unregister_type::<MeshInstance>();
    shared_data.unregister_type::<PipelineInstance>();
    shared_data.unregister_type::<RenderPassInstance>();
    shared_data.unregister_type::<TextureInstance>();
    shared_data.unregister_type::<ViewInstance>();
}
