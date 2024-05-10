use std::collections::HashMap;

use inox_math::Mat4Ops;
use inox_messenger::Listener;
use inox_render::{
    AsBinding, BindingData, CommandBuffer, DrawIndexedCommand, GPUBuffer, GPUInstance, GPUMesh,
    GPUMeshlet, GPUVector, Mesh, MeshId, Pass, RenderContext, RenderContextRc, TextureView,
    MAX_LOD_LEVELS,
};

use inox_core::ContextRc;
use inox_resources::{ResourceEvent, SharedDataRc};
use inox_scene::{Object, ObjectId};

pub const COMPUTE_INSTANCES_NAME: &str = "ComputeInstancesPass";

const DRAW_MESHLET_COMMANDS: bool = true;

struct InstanceMapData {
    instance: GPUInstance,
    id: ObjectId,
}

pub struct ComputeInstancesPass {
    listener: Listener,
    shared_data: SharedDataRc,
    binding_data: BindingData,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    instances: GPUVector<GPUInstance>,
    commands: GPUVector<DrawIndexedCommand>,
    instance_map: HashMap<MeshId, Vec<InstanceMapData>>,
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
            instances: render_context.global_buffers().vector::<GPUInstance>(),
            commands: render_context
                .global_buffers()
                .vector::<DrawIndexedCommand>(),
            instance_map: HashMap::new(),
            is_dirty: true,
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            binding_data: BindingData::new(render_context, COMPUTE_INSTANCES_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_instances_pass::init");

        self.process_messages();
        self.update_instances(render_context);

        if self.commands.read().unwrap().is_empty() || self.instances.read().unwrap().is_empty() {
            return;
        }
        self.binding_data
            .bind_buffer(
                &mut *self.instances.write().unwrap(),
                false,
                Some("Instances"),
            )
            .bind_buffer(&mut *self.commands.write().unwrap(), true, Some("Commands"));
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
        let mut instances = self.instances.write().unwrap();
        let mut commands = self.commands.write().unwrap();

        commands.clear();
        instances.clear();

        let meshes = self.meshes.read().unwrap();
        if DRAW_MESHLET_COMMANDS {
            self.instance_map.iter().for_each(|(mesh_id, data)| {
                if let Some((mesh, _mesh_index)) = meshes.get_first_with_index(mesh_id) {
                    let meshlets = self.meshlets.read().unwrap();
                    if let Some(meshlets) = meshlets.get(mesh_id) {
                        meshlets.iter().enumerate().for_each(|(i, meshlet)| {
                            let max_lod_level = MAX_LOD_LEVELS as u32 - 1;
                            let meshlet_lod_level =
                                max_lod_level - (meshlet.mesh_index_and_lod_level & max_lod_level);
                            if meshlet_lod_level != 0 {
                                return;
                            }
                            let base_instance = instances.len() as u32;
                            data.iter().for_each(|mesh_instance| {
                                let mut instance = mesh_instance.instance;
                                instance.meshlet_index = mesh.meshlets_offset + i as u32;
                                instances.push(instance);
                            });
                            let command = DrawIndexedCommand {
                                instance_count: data.len() as u32,
                                base_instance,
                                base_index: meshlet.indices_offset as _,
                                vertex_count: meshlet.indices_count,
                                vertex_offset: mesh.vertices_position_offset as _,
                            };
                            commands.push(command);
                        });
                    }
                }
            });
        } else {
            self.instance_map.iter().for_each(|(mesh_id, data)| {
                let base_instance = instances.len() as u32;
                instances.extend(data.iter().map(|e| e.instance));
                if let Some((mesh, _mesh_index)) = meshes.get_first_with_index(mesh_id) {
                    let command = DrawIndexedCommand {
                        instance_count: data.len() as u32,
                        base_instance,
                        base_index: mesh.indices_offset as _,
                        vertex_count: mesh.indices_count,
                        vertex_offset: mesh.vertices_position_offset as _,
                    };
                    commands.push(command);
                }
            });
        }

        if commands.is_empty() {
            return;
        }
        //sort in descending order
        commands.sort_by(|a, b| b.instance_count.partial_cmp(&a.instance_count).unwrap());
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
                                let instances = self.instance_map.entry(*mesh.id()).or_default();
                                let matrix = object.get().transform();

                                let result = instances.iter_mut().find(|e| e.id == *object.id());
                                if let Some(data) = result {
                                    data.instance.orientation = matrix.orientation().into();
                                    data.instance.position = matrix.translation().into();
                                    data.instance.scale = matrix.scale().into();
                                    data.instance.mesh_index = mesh.get().mesh_index() as _;
                                    data.instance.bb_min = (*mesh.get().min()).into();
                                    data.instance.bb_max = (*mesh.get().max()).into();
                                } else {
                                    let instance = GPUInstance {
                                        mesh_index: mesh.get().mesh_index() as _,
                                        bb_min: (*mesh.get().min()).into(),
                                        bb_max: (*mesh.get().max()).into(),
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
            })
            .process_messages(|e: &ResourceEvent<Mesh>| match e {
                ResourceEvent::Created(mesh) => {
                    if let Some(instances) = self.instance_map.get_mut(mesh.id()) {
                        instances.iter_mut().for_each(|e| {
                            e.instance.mesh_index = mesh.get().mesh_index() as _;
                            e.instance.bb_min = (*mesh.get().min()).into();
                            e.instance.bb_max = (*mesh.get().max()).into();
                        });
                        self.is_dirty = true;
                    }
                }
                ResourceEvent::Changed(id) => {
                    if let Some(mesh) = self.shared_data.get_resource::<Mesh>(id) {
                        if let Some(instances) = self.instance_map.get_mut(id) {
                            instances.iter_mut().for_each(|e| {
                                e.instance.mesh_index = mesh.get().mesh_index() as _;
                                e.instance.bb_min = (*mesh.get().min()).into();
                                e.instance.bb_max = (*mesh.get().max()).into();
                            });
                            self.is_dirty = true;
                        }
                    }
                }
                _ => {}
            });
    }
}
