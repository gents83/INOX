use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, DrawInstance, Mesh, MeshId, Pass, RenderContext, RenderPass,
    RenderPassData, RenderTarget, ShaderStage, StoreOperation, MESH_FLAGS_OPAQUE,
};

use inox_core::ContextRc;
use inox_messenger::Listener;
use inox_resources::{DataTypeResource, HashBuffer, Resource, ResourceEvent, SharedDataRc};
use inox_uid::generate_random_uid;

pub const DEFAULT_PIPELINE: &str = "pipelines/Default.render_pipeline";
pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.render_pipeline";
pub const OPAQUE_PASS_NAME: &str = "OpaquePass";

pub struct OpaquePass {
    listener: Listener,
    shared_data: SharedDataRc,
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    instances: HashBuffer<MeshId, DrawInstance, 0>,
}
unsafe impl Send for OpaquePass {}
unsafe impl Sync for OpaquePass {}

impl Pass for OpaquePass {
    fn name(&self) -> &str {
        OPAQUE_PASS_NAME
    }
    fn static_name() -> &'static str {
        OPAQUE_PASS_NAME
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("opaque_pass::create");

        let data = RenderPassData {
            name: OPAQUE_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipelines: vec![
                PathBuf::from(DEFAULT_PIPELINE),
                PathBuf::from(WIREFRAME_PIPELINE),
            ],
            ..Default::default()
        };
        let listener = Listener::new(context.message_hub());
        listener.register::<ResourceEvent<Mesh>>();
        Self {
            listener,
            shared_data: context.shared_data().clone(),
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
                None,
            ),
            binding_data: BindingData::default(),
            instances: HashBuffer::default(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("opaque_pass::init");

        if self.instances.is_empty()
            || render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.vertices.is_empty()
            || render_context.render_buffers.vertex_positions.is_empty()
            || render_context.render_buffers.vertex_colors.is_empty()
            || render_context.render_buffers.matrix.is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();
        //let render_texture = pass.render_texture_id();
        //let depth_texture = pass.depth_texture_id();

        self.binding_data
            .add_uniform_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.instances,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertices,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_positions,
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_colors,
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.matrix,
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            /*
            .add_textures_data(
                &render_context.texture_handler,
                render_texture,
                depth_texture,
                BindingInfo {
                    group_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )*/;
        self.binding_data.send_to_gpu(render_context);

        pass.init_pipelines(render_context, &self.binding_data);
    }
    fn update(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("opaque_pass::update");

        if self.instances.is_empty() {
            return;
        }

        let pass = self.render_pass.get();

        let mut encoder = render_context.core.new_encoder();
        let render_pass = pass.begin(render_context, &self.binding_data, &mut encoder);
        pass.draw(render_context, render_pass);

        render_context.core.submit(encoder);
    }
    fn handle_events(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("opaque_pass::handle_events");

        self.listener
            .process_messages(|e: &ResourceEvent<Mesh>| match e {
                ResourceEvent::Changed(id) => {
                    if let Some(mesh) = self.shared_data.get_resource::<Mesh>(id) {
                        if mesh.get().has_flags(MESH_FLAGS_OPAQUE) {
                            self.instances.insert(
                                id,
                                DrawInstance {
                                    mesh_index: render_context
                                        .render_buffers
                                        .meshes
                                        .index(id)
                                        .unwrap_or_default()
                                        as _,
                                    matrix_index: render_context
                                        .render_buffers
                                        .matrix
                                        .index(id)
                                        .unwrap_or_default()
                                        as _,
                                    ..Default::default()
                                },
                            );
                        } else {
                            self.instances.remove(id);
                        }
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    self.instances.remove(id);
                }
                _ => {}
            });
    }
}

impl OpaquePass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
