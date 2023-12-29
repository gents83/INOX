use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock,
    },
};

use inox_bhv::{BHVTree, AABB};
use inox_math::{quantize_snorm, InnerSpace, Mat4Ops, Matrix4, VecBase};
use inox_resources::{to_slice, Buffer, HashBuffer, ResourceId};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{
    declare_as_binding_vector, utils::create_linearized_bhv, AsBinding, BindingDataBuffer,
    GPUBHVNode, GPUMaterial, GPUMesh, GPUMeshlet, GPURay, GPURuntimeVertexData, Light, LightData,
    LightId, Material, MaterialData, MaterialFlags, MaterialId, Mesh, MeshData, MeshFlags, MeshId,
    RenderCommandsPerType, RenderCoreContext, TextureId, TextureInfo, TextureType,
};

declare_as_binding_vector!(VecU32, u32);

pub type TexturesBuffer = Arc<RwLock<HashBuffer<TextureId, TextureInfo, 0>>>;
pub type MaterialsBuffer = Arc<RwLock<HashBuffer<MaterialId, GPUMaterial, 0>>>;
pub type CommandsBuffer = Arc<RwLock<HashMap<MeshFlags, RenderCommandsPerType>>>;
pub type MeshesBuffer = Arc<RwLock<HashBuffer<MeshId, GPUMesh, 0>>>;
pub type MeshletsBuffer = Arc<RwLock<Buffer<GPUMeshlet, 0>>>; //MeshId <-> [GPUMeshlet]
pub type BHVBuffer = Arc<RwLock<Buffer<GPUBHVNode, 0>>>;
pub type IndicesBuffer = Arc<RwLock<Buffer<u32, 0>>>; //MeshId <-> [u32]
pub type VertexPositionsBuffer = Arc<RwLock<Buffer<u32, 0>>>; //MeshId <-> [u32] (10 x, 10 y, 10 z, 2 null)
pub type VertexAttributesBuffer = Arc<RwLock<Buffer<u32, 0>>>; //MeshId <-> [u32]
pub type CullingResults = Arc<RwLock<VecU32>>;
pub type LightsBuffer = Arc<RwLock<HashBuffer<LightId, LightData, 0>>>;
pub type RuntimeVerticesBuffer = Arc<RwLock<Buffer<GPURuntimeVertexData, 0>>>;

pub type RaysBuffer = Arc<RwLock<Buffer<GPURay, 0>>>;

const TLAS_UID: Uid = generate_static_uid_from_string("TLAS");
pub const ATOMIC_SIZE: u32 = 32;

//Alignment should be 4, 8, 16 or 32 bytes
#[derive(Default)]
pub struct RenderBuffers {
    pub textures: TexturesBuffer,
    pub lights: LightsBuffer,
    pub materials: MaterialsBuffer,
    pub commands: CommandsBuffer,
    pub meshes: MeshesBuffer,
    pub meshlets: MeshletsBuffer,
    pub bhv: BHVBuffer,
    pub triangles_ids: RwLock<HashMap<MeshId, Vec<ResourceId>>>,
    pub indices: IndicesBuffer,
    pub vertex_positions: VertexPositionsBuffer,
    pub vertex_attributes: VertexAttributesBuffer,
    pub runtime_vertices: RuntimeVerticesBuffer,
    pub rays: RaysBuffer,
    pub culling_result: CullingResults,
    pub tlas_start_index: AtomicU32,
}

impl RenderBuffers {
    fn extract_meshlets(
        &self,
        mesh_data: &MeshData,
        mesh_id: &MeshId,
        mesh_index: u32,
        indices_offset: u32,
    ) -> (usize, usize) {
        inox_profiler::scoped_profile!("render_buffers::extract_meshlets");

        let mut meshlets = Vec::new();
        meshlets.resize(mesh_data.meshlets.len(), GPUMeshlet::default());
        let mut meshlets_aabbs = Vec::new();
        meshlets_aabbs.resize_with(mesh_data.meshlets.len(), AABB::empty);
        mesh_data
            .meshlets
            .iter()
            .enumerate()
            .for_each(|(i, meshlet_data)| {
                let mut triangles_aabbs = Vec::new();
                triangles_aabbs.resize_with(meshlet_data.indices_count as usize / 3, AABB::empty);
                let mut v = 0;
                let offset = meshlet_data.indices_offset;
                while v < meshlet_data.indices_count {
                    let triangle_index = v / 3;
                    let v1 = mesh_data
                        .position(mesh_data.indices[(offset + triangle_index) as usize] as _);
                    let v2 = mesh_data
                        .position(mesh_data.indices[(offset + triangle_index + 1) as usize] as _);
                    let v3 = mesh_data
                        .position(mesh_data.indices[(offset + triangle_index + 2) as usize] as _);
                    let min = v1.min(v2).min(v3);
                    let max = v1.max(v2).max(v3);
                    triangles_aabbs[triangle_index as usize] =
                        AABB::create(min, max, triangle_index as _);
                    v += 3;
                }
                let bhv = BHVTree::new(&triangles_aabbs);
                let linearized_bhv = create_linearized_bhv(&bhv);
                let triangle_id = generate_random_uid();
                self.triangles_ids
                    .write()
                    .unwrap()
                    .entry(*mesh_id)
                    .or_default()
                    .push(triangle_id);
                let triangle_bhv_range = self
                    .bhv
                    .write()
                    .unwrap()
                    .allocate(&triangle_id, &linearized_bhv)
                    .1;
                let triangles_bhv_index = triangle_bhv_range.start as _;
                self.bhv.write().unwrap().data_mut()[triangle_bhv_range]
                    .iter_mut()
                    .for_each(|n| {
                        if n.miss >= 0 {
                            n.miss += triangles_bhv_index as i32;
                        }
                    });
                let cone_axis = meshlet_data.cone_axis.normalize();
                let cone_axis_cutoff = [
                    quantize_snorm(cone_axis.x, 8) as i8,
                    quantize_snorm(cone_axis.y, 8) as i8,
                    quantize_snorm(cone_axis.z, 8) as i8,
                    quantize_snorm(meshlet_data.cone_angle, 8) as i8,
                ];
                let meshlet = GPUMeshlet {
                    mesh_index,
                    indices_offset: (indices_offset + meshlet_data.indices_offset) as _,
                    indices_count: meshlet_data.indices_count,
                    triangles_bhv_index,
                    center: meshlet_data.cone_center.into(),
                    cone_axis_cutoff,
                };
                meshlets[i] = meshlet;
                meshlets_aabbs[i] =
                    AABB::create(meshlet_data.aabb_min, meshlet_data.aabb_max, i as _);
            });
        if meshlets.is_empty() {
            inox_log::debug_log!("No meshlet data for mesh {:?}", mesh_id);
        }

        let bhv = BHVTree::new(&meshlets_aabbs);
        let linearized_bhv = create_linearized_bhv(&bhv);
        let mesh_bhv_range = self
            .bhv
            .write()
            .unwrap()
            .allocate(mesh_id, &linearized_bhv)
            .1;
        let blas_index = mesh_bhv_range.start as _;
        self.bhv.write().unwrap().data_mut()[mesh_bhv_range]
            .iter_mut()
            .for_each(|n| {
                if n.miss >= 0 {
                    n.miss += blas_index as i32;
                }
            });
        let meshlet_range = self
            .meshlets
            .write()
            .unwrap()
            .allocate(mesh_id, meshlets.as_slice())
            .1;
        (blas_index, meshlet_range.start)
    }
    fn add_vertex_data(
        &self,
        mesh_id: &MeshId,
        mesh_index: u32,
        mesh_data: &MeshData,
    ) -> (u32, u32, u32) {
        inox_profiler::scoped_profile!("render_buffers::add_vertex_data");

        debug_assert!(
            mesh_data.vertex_count() > 0,
            "No vertices for mesh {:?}",
            mesh_id
        );
        debug_assert!(
            !mesh_data.indices.is_empty(),
            "No indices for mesh {:?}",
            mesh_id
        );

        let vertex_offset = self
            .vertex_positions
            .write()
            .unwrap()
            .allocate(mesh_id, to_slice(mesh_data.vertex_positions.as_slice()))
            .1
            .start;
        let attributes_offset = self
            .vertex_attributes
            .write()
            .unwrap()
            .allocate(mesh_id, to_slice(mesh_data.vertex_attributes.as_slice()))
            .1
            .start;
        let runtime_vertices = vec![
            GPURuntimeVertexData {
                mesh_index,
                ..Default::default()
            };
            mesh_data.vertex_count()
        ];
        self.runtime_vertices
            .write()
            .unwrap()
            .allocate(mesh_id, runtime_vertices.as_slice());
        let indices_offset = self
            .indices
            .write()
            .unwrap()
            .allocate(mesh_id, mesh_data.indices.as_slice())
            .1
            .start;
        (
            vertex_offset as _,
            indices_offset as _,
            attributes_offset as _,
        )
    }
    pub fn add_mesh(&self, mesh_id: &MeshId, mesh_data: &MeshData) {
        inox_profiler::scoped_profile!("render_buffers::add_mesh");
        self.remove_mesh(mesh_id, false);
        if mesh_data.vertex_count() == 0 {
            return;
        }
        let mesh_index = self
            .meshes
            .write()
            .unwrap()
            .insert(mesh_id, GPUMesh::default());

        let (vertex_offset, indices_offset, attributes_offset) =
            self.add_vertex_data(mesh_id, mesh_index as _, mesh_data);
        let (blas_index, meshlet_offset) =
            self.extract_meshlets(mesh_data, mesh_id, mesh_index as _, indices_offset);

        {
            let mut meshes = self.meshes.write().unwrap();
            let mesh = meshes.get_mut(mesh_id).unwrap();
            mesh.vertices_position_offset = vertex_offset;
            mesh.vertices_attribute_offset = attributes_offset;
            mesh.flags_and_vertices_attribute_layout = mesh_data.vertex_layout.into();
            mesh.blas_index = blas_index as _;
            mesh.meshlets_offset = meshlet_offset as _;
        }
        self.recreate_tlas();
        self.update_culling_data();
    }
    fn update_culling_data(&self) {
        let num_meshlets = self.meshlets.read().unwrap().item_count();
        let count = ((num_meshlets as u32 + ATOMIC_SIZE - 1) / ATOMIC_SIZE) as usize;
        self.culling_result
            .write()
            .unwrap()
            .set(vec![u32::MAX; count]);
    }
    fn recreate_tlas(&self) {
        inox_profiler::scoped_profile!("render_buffers::recreate_tlas");
        let mut meshes_aabbs = Vec::new();
        {
            let meshes = self.meshes.read().unwrap();
            let bhv = self.bhv.read().unwrap();
            let bhv = bhv.data();
            meshes.for_each_entry(|i, mesh| {
                let node = &bhv[mesh.blas_index as usize];
                let matrix = Matrix4::from_translation_orientation_scale(
                    mesh.position.into(),
                    mesh.orientation.into(),
                    mesh.scale.into(),
                );
                let min = matrix.rotate_point(node.min.into());
                let max = matrix.rotate_point(node.max.into());
                let aabb = AABB::create(min, max, i as _);
                meshes_aabbs.push(aabb);
            });
        }
        let bhv = BHVTree::new(&meshes_aabbs);
        let linearized_bhv = create_linearized_bhv(&bhv);
        let mut bhv = self.bhv.write().unwrap();
        let tlas_range = bhv.allocate(&TLAS_UID, &linearized_bhv).1;
        let tlas_starting_index = tlas_range.start as _;
        self.tlas_start_index
            .store(tlas_starting_index, Ordering::SeqCst);
        bhv.data_mut()[tlas_range].iter_mut().for_each(|n| {
            if n.miss >= 0 {
                n.miss += tlas_starting_index as i32;
            }
        });
    }
    fn update_transform(&self, mesh: &mut Mesh, m: &mut GPUMesh) -> bool {
        inox_profiler::scoped_profile!("render_buffers::update_transform");

        let matrix = mesh.matrix();
        let new_pos = matrix.translation();
        let new_orientation = matrix.orientation();
        let new_scale = matrix.scale();
        let old_pos = m.position.into();
        let old_orientation = m.orientation.into();
        let old_scale = m.scale.into();
        if new_pos != old_pos || new_orientation != old_orientation || new_scale != old_scale {
            m.position = new_pos.into();
            m.orientation = new_orientation.into();
            m.scale = new_scale.into();
            return true;
        }
        false
    }
    pub fn change_mesh(&self, mesh_id: &MeshId, mesh: &mut Mesh) {
        inox_profiler::scoped_profile!("render_buffers::change_mesh");
        let mut is_matrix_changed = false;
        {
            let mut meshes = self.meshes.write().unwrap();
            if let Some(m) = meshes.get_mut(mesh_id) {
                if let Some(material) = mesh.material() {
                    if let Some(index) = self.materials.read().unwrap().index_of(material.id()) {
                        m.material_index = index as _;
                    }
                    if let Some(material) = self.materials.write().unwrap().get_mut(material.id()) {
                        let flags: MaterialFlags = material.flags.into();
                        if flags.contains(MaterialFlags::AlphaModeBlend)
                            || material.base_color[3] < 1.
                        {
                            mesh.remove_flag(MeshFlags::Opaque);
                            mesh.add_flag(MeshFlags::Tranparent);
                        }
                    }
                }

                is_matrix_changed = self.update_transform(mesh, m);

                let mesh_flags = mesh.flags();
                let vertex_attribute_layout = m.flags_and_vertices_attribute_layout & 0x0000FFFF;
                let flags: u32 = (*mesh_flags).into();
                m.flags_and_vertices_attribute_layout = vertex_attribute_layout | (flags << 16);
                {
                    let mut commands = self.commands.write().unwrap();
                    commands.iter_mut().for_each(|(_, v)| {
                        v.remove_commands(mesh_id);
                    });
                    let entry = commands.entry(*mesh_flags).or_default();
                    entry.add_commands(mesh_id, m, &self.meshlets.read().unwrap());
                }

                meshes.set_dirty(true);
            }
        }
        if is_matrix_changed {
            self.recreate_tlas();
        }
    }
    pub fn remove_mesh(&self, mesh_id: &MeshId, recreate_tlas: bool) {
        inox_profiler::scoped_profile!("render_buffers::remove_mesh");

        if self.meshes.write().unwrap().remove(mesh_id).is_some() {
            self.commands
                .write()
                .unwrap()
                .iter_mut()
                .for_each(|(_, entry)| {
                    entry.remove_commands(mesh_id);
                });
            self.meshlets.write().unwrap().remove(mesh_id);
            {
                let mut bhv = self.bhv.write().unwrap();
                bhv.remove(mesh_id);
                let mut triangle_ids = self.triangles_ids.write().unwrap();
                triangle_ids.get(mesh_id).unwrap().iter().for_each(|id| {
                    bhv.remove(id);
                });
                triangle_ids.remove(mesh_id);
            }
            self.indices.write().unwrap().remove(mesh_id);
            self.vertex_positions.write().unwrap().remove(mesh_id);
            self.runtime_vertices.write().unwrap().remove(mesh_id);
            self.vertex_attributes.write().unwrap().remove(mesh_id);
        }
        if recreate_tlas {
            self.recreate_tlas();
        }
        self.update_culling_data();
    }
    pub fn add_material(&self, material_id: &MaterialId, material: &mut Material) {
        inox_profiler::scoped_profile!("render_buffers::add_material");

        let mut textures_index_and_coord_set = [0; TextureType::Count as _];
        material
            .textures()
            .iter()
            .enumerate()
            .for_each(|(i, handle_texture)| {
                if let Some(texture) = handle_texture {
                    textures_index_and_coord_set[i] = (texture.get().texture_index() + 1) as u32;
                }
            });
        let mut materials = self.materials.write().unwrap();
        if let Some(m) = materials.get_mut(material_id) {
            m.textures_index_and_coord_set = textures_index_and_coord_set;
        } else {
            let index = materials.insert(
                material_id,
                GPUMaterial {
                    textures_index_and_coord_set,
                    ..Default::default()
                },
            );
            material.set_material_index(index as _);
        }
        materials.set_dirty(true);
    }
    pub fn update_material(&self, material_id: &MaterialId, material_data: &MaterialData) {
        inox_profiler::scoped_profile!("render_buffers::update_material");
        let mut materials = self.materials.write().unwrap();
        if let Some(material) = materials.get_mut(material_id) {
            for (i, t) in material_data.texcoords_set.iter().enumerate() {
                material.textures_index_and_coord_set[i] |= (*t << 28) as u32;
            }
            material.roughness_factor = material_data.roughness_factor;
            material.metallic_factor = material_data.roughness_factor;
            material.ior = material_data.ior;
            material.transmission_factor = material_data.transmission_factor;
            material.base_color = material_data.base_color.into();
            material.emissive_color = material_data.emissive_color.into();
            material.occlusion_strength = material_data.occlusion_strength;
            material.diffuse_color = material_data.diffuse_factor.into();
            material.specular_color = material_data.specular_glossiness_factor.into();
            material.attenuation_color_and_distance =
                material_data.attenuation_color_and_distance.into();
            material.thickness_factor = material_data.thickness_factor;
            material.alpha_cutoff = material_data.alpha_cutoff;
            material.emissive_strength = material_data.emissive_strength;
            material.flags = material_data.flags.into();
            materials.set_dirty(true);
        }
    }
    pub fn remove_material(&self, material_id: &MaterialId) {
        inox_profiler::scoped_profile!("render_buffers::remove_material");

        self.materials.write().unwrap().remove(material_id);
    }

    pub fn add_light(&self, light_id: &LightId, light: &mut Light) {
        inox_profiler::scoped_profile!("render_buffers::add_light");

        let index = self
            .lights
            .write()
            .unwrap()
            .insert(light_id, LightData::default());
        light.set_light_index(index as _);
    }
    pub fn update_light(&self, light_id: &LightId, light_data: &LightData) {
        inox_profiler::scoped_profile!("render_buffers::update_light");
        let mut lights = self.lights.write().unwrap();
        if let Some(light) = lights.get_mut(light_id) {
            *light = *light_data;
            lights.set_dirty(true);
        }
    }
    pub fn remove_light(&self, light_id: &LightId) {
        inox_profiler::scoped_profile!("render_buffers::remove_light");

        self.lights.write().unwrap().remove(light_id);
    }

    pub fn add_texture(&self, texture_id: &TextureId, texture_data: &TextureInfo) -> usize {
        inox_profiler::scoped_profile!("render_buffers::add_texture");

        self.textures
            .write()
            .unwrap()
            .insert(texture_id, *texture_data)
    }
    pub fn remove_texture(&self, texture_id: &TextureId) {
        inox_profiler::scoped_profile!("render_buffers::remove_texture");

        self.textures.write().unwrap().remove(texture_id);
    }

    pub fn bind_commands(
        &self,
        binding_data_buffer: &BindingDataBuffer,
        render_core_context: &RenderCoreContext,
        force_rebind: bool,
    ) {
        inox_profiler::scoped_profile!("render_buffers::bind_commands");

        self.commands
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|(_, commands)| {
                commands.map.iter_mut().for_each(|(_, entry)| {
                    if entry.commands.is_empty() {
                        return;
                    }
                    if force_rebind {
                        entry.rebind();
                    }
                    if entry.commands.is_dirty() {
                        let usage = wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_SRC
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::INDIRECT;
                        binding_data_buffer.bind_buffer(
                            Some("Commands"),
                            &mut entry.commands,
                            usage,
                            render_core_context,
                        );
                    }
                    if entry.counter.is_dirty() {
                        let usage = wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_SRC
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::INDIRECT;
                        binding_data_buffer.bind_buffer(
                            Some("Counter"),
                            &mut entry.counter,
                            usage,
                            render_core_context,
                        );
                    }
                });
            });
    }
}
