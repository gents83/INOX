use std::collections::HashMap;

use inox_math::{Mat4Ops, Vector4};
use inox_messenger::Listener;
use inox_render::{
    AsBinding, BindingData, CommandBuffer, DrawIndexedCommand, GPUBuffer, GPUInstance, GPUMesh,
    GPUMeshlet, GPUTransform, GPUVector, Mesh, MeshId, Pass, RenderContext, RenderContextRc,
    TextureView,
};

use inox_core::ContextRc;
use inox_resources::{ResourceEvent, SharedDataRc};
use inox_scene::{Object, ObjectId};
use inox_uid::{generate_static_uid_from_string, Uid};

pub const COMPUTE_INSTANCES_NAME: &str = "ComputeInstancesPass";

pub const INSTANCE_DATA_ID: Uid = generate_static_uid_from_string("INSTANCE_DATA_ID");
pub const COMMANDS_DATA_ID: Uid = generate_static_uid_from_string("COMMANDS_DATA_ID");

struct TransformMapData {
    transform: GPUTransform,
    id: ObjectId,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CommandsData(pub i32);
impl AsBinding for CommandsData {
    fn count(&self) -> usize {
        1
    }

    fn size(&self) -> u64 {
        std::mem::size_of::<i32>() as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut inox_render::BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}

pub struct ComputeInstancesPass {
    listener: Listener,
    shared_data: SharedDataRc,
    binding_data: BindingData,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    transforms: GPUVector<GPUTransform>,
    commands: GPUVector<DrawIndexedCommand>,
    commands_data: GPUVector<CommandsData>,
    instances: GPUVector<GPUInstance>,
    transform_map: HashMap<MeshId, Vec<TransformMapData>>,
    is_dirty: bool,
}
unsafe impl Send for ComputeInstancesPass {}
unsafe impl Sync for ComputeInstancesPass {}

impl Pass for ComputeInstancesPass {
    fn name(&self) -> &str {
        COMPUTE_INSTANCES_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_INSTANCES_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let listener = Listener::new(context.message_hub());
        listener
            .register::<ResourceEvent<Object>>()
            .register::<ResourceEvent<Mesh>>();

        Self {
            listener,
            shared_data: context.shared_data().clone(),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            transform_map: HashMap::new(),
            is_dirty: true,
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            commands: render_context
                .global_buffers()
                .vector::<DrawIndexedCommand>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            commands_data: render_context
                .global_buffers()
                .vector_with_id::<CommandsData>(COMMANDS_DATA_ID),
            binding_data: BindingData::new(render_context, COMPUTE_INSTANCES_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_instances_pass::init");

        self.process_messages();
        self.update_instances(render_context);

        if self.commands_data.read().unwrap().is_empty()
            || self.transforms.read().unwrap().is_empty()
        {
            return;
        }
        self.binding_data
            .bind_buffer(
                &mut *self.transforms.write().unwrap(),
                false,
                Some("Instances"),
            )
            .bind_buffer(&mut *self.commands.write().unwrap(), true, Some("Commands"))
            .bind_buffer(
                &mut *self.commands_data.write().unwrap(),
                false,
                Some("CommandsData"),
            )
            .bind_buffer(
                &mut *self.instances.write().unwrap(),
                false,
                Some("InstanceData"),
            );
    }

    fn update(
        &mut self,
        _render_context: &RenderContext,
        _surface_view: &TextureView,
        _command_buffer: &mut CommandBuffer,
    ) {
    }
}

impl ComputeInstancesPass {
    fn update_instances(&mut self, render_context: &RenderContext) {
        if !self.is_dirty {
            return;
        }
        self.is_dirty = false;
        let mut transforms = self.transforms.write().unwrap();
        let mut commands = self.commands.write().unwrap();
        let mut commands_data = self.commands_data.write().unwrap();
        let mut instances = self.instances.write().unwrap();

        commands_data.clear();
        transforms.clear();
        instances.clear();
        commands.clear();

        let meshes = self.meshes.read().unwrap();
        self.transform_map.iter().for_each(|(mesh_id, data)| {
            if let Some((mesh, _mesh_index)) = meshes.get_first_with_index(mesh_id) {
                let base_instance = transforms.len() as u32;
                let meshlets = self.meshlets.read().unwrap();
                if let Some(meshlets) = meshlets.get(mesh_id) {
                    let current_len = instances.len();
                    instances.reserve(current_len + meshlets.len() * data.len());
                    let current_len = commands_data.len();
                    commands_data
                        .resize(current_len + meshlets.len() * data.len(), CommandsData(-1));
                    data.iter().enumerate().for_each(|(j, mesh_instance)| {
                        transforms.push(mesh_instance.transform);
                        meshlets.iter().enumerate().for_each(|(i, meshlet)| {
                            instances.push(GPUInstance {
                                transform_id: base_instance + j as u32,
                                mesh_id: meshlet.mesh_index_and_lod_level >> 3,
                                meshlet_id: mesh.meshlets_offset + i as u32,
                                command_id: -1,
                            });
                        });
                    });
                }
            }
        });

        if commands_data.is_empty() {
            return;
        }
        //sort in descending order
        commands.resize(commands_data.len(), DrawIndexedCommand::default());
        transforms.mark_as_dirty(render_context);
        commands_data.mark_as_dirty(render_context);
        instances.mark_as_dirty(render_context);
        commands.mark_as_dirty(render_context);
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
                                let instances = self.transform_map.entry(*mesh.id()).or_default();
                                let matrix = object.get().transform();
                                let position = matrix.translation();
                                let scale = matrix.scale();
                                let bb_min = *mesh.get().min();
                                let bb_max = *mesh.get().max();

                                let result = instances.iter_mut().find(|e| e.id == *object.id());
                                if let Some(data) = result {
                                    data.transform.orientation = matrix.orientation().into();
                                    data.transform.position_scale_x =
                                        Vector4::new(position.x, position.y, position.z, scale.x)
                                            .into();
                                    data.transform.bb_min_scale_y =
                                        Vector4::new(bb_min.x, bb_min.y, bb_min.z, scale.y).into();
                                    data.transform.bb_max_scale_z =
                                        Vector4::new(bb_max.x, bb_max.y, bb_max.z, scale.z).into();
                                } else {
                                    let transform = GPUTransform {
                                        orientation: matrix.orientation().into(),
                                        position_scale_x: Vector4::new(
                                            position.x, position.y, position.z, scale.x,
                                        )
                                        .into(),
                                        bb_min_scale_y: Vector4::new(
                                            bb_min.x, bb_min.y, bb_min.z, scale.y,
                                        )
                                        .into(),
                                        bb_max_scale_z: Vector4::new(
                                            bb_max.x, bb_max.y, bb_max.z, scale.z,
                                        )
                                        .into(),
                                    };
                                    instances.push(TransformMapData {
                                        transform,
                                        id: *object.id(),
                                    });
                                }
                            });
                        self.is_dirty = true;
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    self.transform_map.iter_mut().for_each(|(_, instances)| {
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
            })
            .process_messages(|e: &ResourceEvent<Mesh>| match e {
                ResourceEvent::Created(mesh) => {
                    if let Some(instances) = self.transform_map.get_mut(mesh.id()) {
                        instances.iter_mut().for_each(|e| {
                            let bb_min = *mesh.get().min();
                            let bb_max = *mesh.get().max();
                            e.transform.bb_min_scale_y = Vector4::new(
                                bb_min.x,
                                bb_min.y,
                                bb_min.z,
                                e.transform.bb_min_scale_y[3],
                            )
                            .into();
                            e.transform.bb_max_scale_z = Vector4::new(
                                bb_max.x,
                                bb_max.y,
                                bb_max.z,
                                e.transform.bb_max_scale_z[3],
                            )
                            .into();
                        });
                        self.is_dirty = true;
                    }
                }
                ResourceEvent::Changed(id) => {
                    if let Some(mesh) = self.shared_data.get_resource::<Mesh>(id) {
                        if let Some(instances) = self.transform_map.get_mut(id) {
                            instances.iter_mut().for_each(|e| {
                                let bb_min = *mesh.get().min();
                                let bb_max = *mesh.get().max();
                                e.transform.bb_min_scale_y = Vector4::new(
                                    bb_min.x,
                                    bb_min.y,
                                    bb_min.z,
                                    e.transform.bb_min_scale_y[3],
                                )
                                .into();
                                e.transform.bb_max_scale_z = Vector4::new(
                                    bb_max.x,
                                    bb_max.y,
                                    bb_max.z,
                                    e.transform.bb_max_scale_z[3],
                                )
                                .into();
                            });
                            self.is_dirty = true;
                        }
                    }
                }
                _ => {}
            });
    }
}
