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
    shared_data.register_type::<Font>();
    shared_data.register_type::<Material>();
    shared_data.register_type::<Mesh>();
    shared_data.register_type::<Pipeline>();
    shared_data.register_type::<RenderPass>();
    shared_data.register_type::<Texture>();
    shared_data.register_type::<View>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.unregister_type::<Font>();
    shared_data.unregister_type::<Material>();
    shared_data.unregister_type::<Mesh>();
    shared_data.unregister_type::<Pipeline>();
    shared_data.unregister_type::<RenderPass>();
    shared_data.unregister_type::<Texture>();
    shared_data.unregister_type::<View>();
}
