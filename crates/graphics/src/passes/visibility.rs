use std::{collections::HashMap, path::PathBuf};

use inox_bvh::GPUBVHNode;
use inox_math::Mat4Ops;
use inox_messenger::Listener;
use inox_render::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, ConstantDataRw, DrawCommandType,
    DrawIndexedCommand, GPUBuffer, GPUInstance, GPUMesh, GPUMeshlet, GPUVertexIndices,
    GPUVertexPosition, Mesh, MeshFlags, MeshId, Pass, RenderContext, RenderContextRc, RenderPass,
    RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage, StoreOperation, Texture,
    TextureView, VextexBindingType,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceEvent, ResourceTrait, SharedDataRc};
use inox_scene::{Object, ObjectId};
use inox_uid::generate_random_uid;

pub const VISIBILITY_BUFFER_PIPELINE: &str = "pipelines/VisibilityBuffer.render_pipeline";
pub const VISIBILITY_BUFFER_PASS_NAME: &str = "VisibilityBufferPass";

struct InstanceMapData {
    instance: GPUInstance,
    id: ObjectId,
}

pub struct VisibilityBufferPass {
    render_pass: Resource<RenderPass>,
    listener: Listener,
    shared_data: SharedDataRc,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: GPUBuffer<GPUMesh>,
    indices: GPUBuffer<GPUVertexIndices>,
    bhv: GPUBuffer<GPUBVHNode>,
    instances: Vec<GPUInstance>,
    instance_map: HashMap<MeshId, Vec<InstanceMapData>>,
    is_dirty: bool,
    commands: Vec<DrawIndexedCommand>,
    commands_count: usize,
    vertices_position: GPUBuffer<GPUVertexPosition>,
}
unsafe impl Send for VisibilityBufferPass {}
unsafe impl Sync for VisibilityBufferPass {}

impl Pass for VisibilityBufferPass {
    fn name(&self) -> &str {
        VISIBILITY_BUFFER_PASS_NAME
    }
    fn static_name() -> &'static str {
        VISIBILITY_BUFFER_PASS_NAME
    }
    fn is_active(&self, render_context: &RenderContext) -> bool {
        render_context.has_commands(&self.draw_commands_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Opaque
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("visibility_buffer_pass::create");

        let data = RenderPassData {
            name: VISIBILITY_BUFFER_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(VISIBILITY_BUFFER_PIPELINE),
            ..Default::default()
        };

        let listener = Listener::new(context.message_hub());
        listener.register::<ResourceEvent<Object>>();

        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            shared_data: context.shared_data().clone(),
            listener,
            binding_data: BindingData::new(render_context, VISIBILITY_BUFFER_PASS_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            bhv: render_context.global_buffers().buffer::<GPUBVHNode>(),
            instance_map: HashMap::new(),
            instances: Vec::new(),
            commands: Vec::new(),
            commands_count: 0,
            is_dirty: true,
            vertices_position: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("visibility_buffer_pass::init");

        self.process_messages();
        self.update_instances(render_context);

        if self.commands_count == 0 {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .set_vertex_buffer(
                VextexBindingType::Vertex,
                &mut *self.vertices_position.write().unwrap(),
                Some("Vertices Position"),
            )
            .set_vertex_buffer(
                VextexBindingType::Instance,
                &mut self.instances,
                Some("Instances"),
            )
            .set_index_buffer(&mut *self.indices.write().unwrap(), Some("Indices"))
            .bind_buffer(&mut self.commands, Some("Commands"))
            .bind_buffer(&mut self.commands_count, Some("Commands Count"));

        let vertex_layout = GPUVertexPosition::descriptor(0);
        let instance_layout = GPUInstance::descriptor(vertex_layout.location());
        pass.init(
            render_context,
            &mut self.binding_data,
            Some(vertex_layout),
            Some(instance_layout),
        );
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("visibility_buffer_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        if self.is_dirty || self.commands_count == 0 {
            return;
        }

        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler().render_targets();

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.webgpu,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "visibility_pass",
            );
            pass.indirect_draw(
                render_context,
                &self.commands,
                &self.commands_count,
                render_pass,
            );
        }
    }
}

impl VisibilityBufferPass {
    pub fn add_render_target(&self, texture: &Resource<Texture>) -> &Self {
        self.render_pass.get_mut().add_render_target(texture);
        self
    }
    pub fn add_depth_target(&self, texture: &Resource<Texture>) -> &Self {
        self.render_pass.get_mut().add_depth_target(texture);
        self
    }
    fn update_instances(&mut self, render_context: &RenderContext) {
        if !self.is_dirty {
            return;
        }
        self.commands.clear();
        self.instances.clear();

        let meshes = self.meshes.read().unwrap();
        self.instance_map.iter().for_each(|(mesh_id, data)| {
            let (mesh, _mesh_index) = meshes.get_first_with_index(mesh_id).unwrap();

            let meshlets = render_context.global_buffers().buffer::<GPUMeshlet>();
            let meshlets = meshlets.read().unwrap();
            if let Some(meshlets) = meshlets.get(mesh_id) {
                meshlets.iter().for_each(|meshlet| {
                    let base_instance = self.instances.len() as u32;
                    data.iter().for_each(|mesh_instance| {
                        let mut instance = mesh_instance.instance;
                        instance.meshlet_index = mesh.meshlets_offset;
                        self.instances.push(instance);
                    });
                    let command = DrawIndexedCommand {
                        instance_count: data.len() as u32,
                        base_instance,
                        base_index: meshlet.indices_offset as _,
                        vertex_count: meshlet.indices_count,
                        vertex_offset: mesh.vertices_position_offset as _,
                    };
                    self.commands.push(command);
                });
            }
        });
        //sort in descending order
        self.commands
            .sort_by(|a, b| b.instance_count.partial_cmp(&a.instance_count).unwrap());
        self.commands_count = self.commands.len();
        self.instances.mark_as_dirty(render_context);
        self.commands.mark_as_dirty(render_context);
        self.commands_count.mark_as_dirty(render_context);

        self.is_dirty = false;
    }

    fn process_messages(&mut self) {
        self.listener
            .process_messages(|e: &ResourceEvent<Object>| match e {
                ResourceEvent::Changed(id) => {
                    if let Some(object) = self.shared_data.get_resource::<Object>(id) {
                        object
                            .get()
                            .components_of_type::<Mesh>()
                            .iter()
                            .for_each(|mesh| {
                                let instances = self.instance_map.entry(*mesh.id()).or_default();

                                let meshes = self.meshes.read().unwrap();
                                let (mesh, mesh_index) =
                                    meshes.get_first_with_index(mesh.id()).unwrap();
                                let bhv = self.bhv.read().unwrap();
                                let bhv = bhv.data();
                                let node = &bhv[mesh.blas_index as usize];
                                let matrix = object.get().transform();

                                let result = instances.iter_mut().find(|e| e.id == *object.id());
                                if let Some(data) = result {
                                    data.instance.orientation = matrix.orientation().into();
                                    data.instance.position = matrix.translation().into();
                                    data.instance.scale = matrix.scale().into();
                                } else {
                                    let instance = GPUInstance {
                                        mesh_index,
                                        bb_min: node.min,
                                        bb_max: node.max,
                                        orientation: matrix.orientation().into(),
                                        position: matrix.translation().into(),
                                        scale: matrix.scale().into(),
                                        meshlet_index: 0,
                                        ..Default::default()
                                    };
                                    instances.push(InstanceMapData {
                                        instance,
                                        id: *object.id(),
                                    });
                                }
                            });
                        self.is_dirty = true;
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    self.instance_map.iter_mut().for_each(|(_, instances)| {
                        instances.retain(|e| {
                            if e.id == *id {
                                self.is_dirty = true;
                                true
                            } else {
                                false
                            }
                        });
                    });
                }
                _ => {}
            });
    }
}
