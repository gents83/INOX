use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, RwLock},
};

use inox_math::{quantize_snorm, InnerSpace, Mat4Ops};
use inox_resources::{to_slice, Buffer, HashBuffer};

use crate::{
    AsBinding, BindingDataBuffer, DrawBoundingBox, DrawMaterial, DrawMesh, DrawMeshlet, DrawVertex,
    Light, LightData, LightId, Material, MaterialAlphaMode, MaterialData, MaterialId, Mesh,
    MeshData, MeshFlags, MeshId, RenderCommandsPerType, RenderCoreContext, TextureId, TextureInfo,
    TextureType, INVALID_INDEX, MAX_TEXTURE_COORDS_SETS,
};

pub type TexturesBuffer = Arc<RwLock<HashBuffer<TextureId, TextureInfo, 0>>>;
pub type LightsBuffer = Arc<RwLock<HashBuffer<LightId, LightData, 0>>>;
pub type MaterialsBuffer = Arc<RwLock<HashBuffer<MaterialId, DrawMaterial, 0>>>;
pub type CommandsBuffer = Arc<RwLock<HashMap<MeshFlags, RenderCommandsPerType>>>;
pub type MeshesBuffer = Arc<RwLock<HashBuffer<MeshId, DrawMesh, 0>>>;
pub type MeshesAABBsBuffer = Arc<RwLock<HashBuffer<MeshId, DrawBoundingBox, 0>>>;
pub type MeshletsBuffer = Arc<RwLock<Buffer<DrawMeshlet>>>; //MeshId <-> [DrawMeshlet]
pub type MeshletsAABBsBuffer = Arc<RwLock<Buffer<DrawBoundingBox>>>; //MeshId <-> [DrawBoundingBox]
pub type VerticesBuffer = Arc<RwLock<Buffer<DrawVertex>>>; //MeshId <-> [DrawVertex]
pub type IndicesBuffer = Arc<RwLock<Buffer<u32>>>; //MeshId <-> [u32]
pub type VertexPositionsBuffer = Arc<RwLock<Buffer<u32>>>; //MeshId <-> [u32] (10 x, 10 y, 10 z, 2 null)
pub type VertexColorsBuffer = Arc<RwLock<Buffer<u32>>>; //MeshId <-> [u32] (rgba)
pub type VertexNormalsBuffer = Arc<RwLock<Buffer<u32>>>; //MeshId <-> [u32] (10 x, 10 y, 10 z, 2 null)
pub type VertexUVsBuffer = Arc<RwLock<Buffer<u32>>>; //MeshId <-> [u32] (2 half)

//Alignment should be 4, 8, 16 or 32 bytes
#[derive(Default)]
pub struct RenderBuffers {
    pub textures: TexturesBuffer,
    pub lights: LightsBuffer,
    pub materials: MaterialsBuffer,
    pub commands: CommandsBuffer,
    pub meshes: MeshesBuffer,
    pub meshes_aabb: MeshesAABBsBuffer,
    pub meshlets: MeshletsBuffer,
    pub meshlets_aabb: MeshletsAABBsBuffer,
    pub vertices: VerticesBuffer,
    pub indices: IndicesBuffer,
    pub vertex_positions: VertexPositionsBuffer,
    pub vertex_colors: VertexColorsBuffer,
    pub vertex_normals: VertexNormalsBuffer,
    pub vertex_uvs: VertexUVsBuffer,
}

impl RenderBuffers {
    fn extract_meshlets(
        &self,
        mesh_data: &MeshData,
        mesh_id: &MeshId,
        mesh_index: u32,
    ) -> Range<usize> {
        inox_profiler::scoped_profile!("render_buffers::extract_meshlets");

        let mut meshlets = Vec::new();
        meshlets.resize(mesh_data.meshlets.len(), DrawMeshlet::default());
        let mut bhvs = Vec::new();
        bhvs.resize_with(mesh_data.meshlets.len(), || DrawBoundingBox {
            min: mesh_data.aabb_min.into(),
            max: mesh_data.aabb_max.into(),
            parent_or_count: INVALID_INDEX,
            children_start: INVALID_INDEX,
        });
        mesh_data
            .meshlets
            .iter()
            .enumerate()
            .for_each(|(i, meshlet_data)| {
                let cone_axis = meshlet_data.cone_axis.normalize();
                let cone_axis_cutoff = [
                    quantize_snorm(cone_axis.x, 8) as i8,
                    quantize_snorm(cone_axis.y, 8) as i8,
                    quantize_snorm(cone_axis.z, 8) as i8,
                    quantize_snorm(meshlet_data.cone_angle, 8) as i8,
                ];
                let meshlet = DrawMeshlet {
                    mesh_index,
                    bb_index: i as _,
                    indices_offset: meshlet_data.indices_offset as _,
                    indices_count: meshlet_data.indices_count,
                    center: meshlet_data.cone_center.into(),
                    cone_axis_cutoff,
                };
                meshlets[i] = meshlet;
                bhvs[i].min = meshlet_data.aabb_min.into();
                bhvs[i].max = meshlet_data.aabb_max.into();
            });
        let bhv_range = self
            .meshlets_aabb
            .write()
            .unwrap()
            .allocate(mesh_id, bhvs.as_slice())
            .1;
        meshlets.iter_mut().enumerate().for_each(|(i, meshlet)| {
            meshlet.bb_index = (bhv_range.start + i) as _;
        });
        if meshlets.is_empty() {
            inox_log::debug_log!("No meshlet data for mesh {:?}", mesh_id);
        }
        let mesh_bhv_index = self.add_mesh_bhv(mesh_id, mesh_data, &bhv_range);
        self.meshlets_aabb.write().unwrap().data_mut()[bhv_range]
            .iter_mut()
            .for_each(|bhv| {
                bhv.parent_or_count = mesh_bhv_index as _;
            });
        self.meshlets
            .write()
            .unwrap()
            .allocate(mesh_id, meshlets.as_slice())
            .1
    }
    fn add_vertex_data(
        &self,
        mesh_id: &MeshId,
        mesh_data: &MeshData,
        mesh_index: u32,
    ) -> (u32, u32) {
        inox_profiler::scoped_profile!("render_buffers::add_vertex_data");

        if mesh_data.vertices.is_empty() {
            inox_log::debug_log!("No vertices for mesh {:?}", mesh_id);
            return (0, 0);
        }
        if mesh_data.indices.is_empty() {
            inox_log::debug_log!("No indices for mesh {:?}", mesh_id);
            return (0, 0);
        }

        let position_range = self
            .vertex_positions
            .write()
            .unwrap()
            .allocate(mesh_id, to_slice(mesh_data.positions.as_slice()))
            .1;
        //We're expecting positions and colors to be always present
        if mesh_data.colors.is_empty() {
            let colors = vec![0xFFFFFFFFu32; mesh_data.positions.len()];
            self.vertex_colors
                .write()
                .unwrap()
                .allocate(mesh_id, to_slice(colors.as_slice()));
        } else {
            self.vertex_colors
                .write()
                .unwrap()
                .allocate(mesh_id, to_slice(mesh_data.colors.as_slice()));
        }

        let mut normal_range = Range::<usize>::default();
        if !mesh_data.normals.is_empty() {
            normal_range = self
                .vertex_normals
                .write()
                .unwrap()
                .allocate(mesh_id, to_slice(mesh_data.normals.as_slice()))
                .1;
        }

        let mut uv_range = Range::<usize>::default();
        if !mesh_data.uvs.is_empty() {
            uv_range = self
                .vertex_uvs
                .write()
                .unwrap()
                .allocate(mesh_id, to_slice(mesh_data.uvs.as_slice()))
                .1;
        }

        let mut vertices = mesh_data.vertices.clone();
        vertices.iter_mut().for_each(|v| {
            v.position_and_color_offset += position_range.start as u32;
            v.normal_offset += normal_range.start as i32;
            (0..MAX_TEXTURE_COORDS_SETS).for_each(|i| {
                v.uv_offset[i] += uv_range.start as i32;
            });
            v.mesh_index = mesh_index;
        });
        let vertex_offset = self
            .vertices
            .write()
            .unwrap()
            .allocate(mesh_id, vertices.as_slice())
            .1
            .start;
        let indices_offset = self
            .indices
            .write()
            .unwrap()
            .allocate(mesh_id, mesh_data.indices.as_slice())
            .1
            .start;
        (vertex_offset as _, indices_offset as _)
    }
    fn add_mesh_bhv(
        &self,
        mesh_id: &MeshId,
        mesh_data: &MeshData,
        children_range: &Range<usize>,
    ) -> usize {
        inox_profiler::scoped_profile!("render_buffers::add_mesh_bhv");

        let bhv = DrawBoundingBox {
            min: mesh_data.aabb_min.into(),
            max: mesh_data.aabb_max.into(),
            children_start: children_range.start as _,
            parent_or_count: mesh_data.meshlets.len() as _,
        };
        self.meshes_aabb.write().unwrap().insert(mesh_id, bhv)
    }
    pub fn add_mesh(&self, mesh_id: &MeshId, mesh_data: &MeshData) {
        inox_profiler::scoped_profile!("render_buffers::add_mesh");
        self.remove_mesh(mesh_id);
        if mesh_data.vertex_count() == 0 {
            return;
        }
        let mesh_index = self
            .meshes
            .write()
            .unwrap()
            .insert(mesh_id, DrawMesh::default());

        let (vertex_offset, indices_offset) =
            self.add_vertex_data(mesh_id, mesh_data, mesh_index as _);
        let range = self.extract_meshlets(mesh_data, mesh_id, mesh_index as _);

        let mut meshes = self.meshes.write().unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        mesh.vertex_offset = vertex_offset;
        mesh.indices_offset = indices_offset;
        mesh.meshlets_offset = range.start as _;
        mesh.meshlets_count = mesh_data.meshlets.len() as _;
    }
    pub fn change_mesh(&self, mesh_id: &MeshId, mesh: &mut Mesh) {
        inox_profiler::scoped_profile!("render_buffers::change_mesh");
        let mut meshes = self.meshes.write().unwrap();
        if let Some(m) = meshes.get_mut(mesh_id) {
            if let Some(material) = mesh.material() {
                if let Some(index) = self.materials.read().unwrap().index_of(material.id()) {
                    m.material_index = index as _;
                }
                if let Some(material) = self.materials.write().unwrap().get_mut(material.id()) {
                    let blend_alpha_mode: u32 = MaterialAlphaMode::Blend.into();
                    if material.alpha_mode == blend_alpha_mode || material.base_color[3] < 1. {
                        mesh.remove_flag(MeshFlags::Opaque);
                        mesh.add_flag(MeshFlags::Tranparent);
                    }
                }
            }
            let mesh_flags = mesh.flags();
            {
                let mut commands = self.commands.write().unwrap();
                let entry = commands.entry(*mesh_flags).or_default();
                entry.add_commands(mesh_id, m, &self.meshlets.read().unwrap());
            }

            let matrix = mesh.matrix();
            m.position = matrix.translation().into();
            m.orientation = matrix.orientation().into();
            m.scale = matrix.scale().into();
            m.mesh_flags = (*mesh_flags).into();
            meshes.set_dirty(true);
        }
    }
    pub fn remove_mesh(&self, mesh_id: &MeshId) {
        inox_profiler::scoped_profile!("render_buffers::remove_mesh");

        if let Some(mesh) = self.meshes.write().unwrap().remove(mesh_id) {
            let mesh_flags: MeshFlags = mesh.mesh_flags.into();
            if let Some(entry) = self.commands.write().unwrap().get_mut(&mesh_flags) {
                entry.remove_commands(mesh_id);
            }
            self.meshlets.write().unwrap().remove(mesh_id);
            self.meshes_aabb.write().unwrap().remove(mesh_id);
            self.meshlets_aabb.write().unwrap().remove(mesh_id);
            self.vertices.write().unwrap().remove(mesh_id);
            self.indices.write().unwrap().remove(mesh_id);
            self.vertex_positions.write().unwrap().remove(mesh_id);
            self.vertex_colors.write().unwrap().remove(mesh_id);
            self.vertex_normals.write().unwrap().remove(mesh_id);
            self.vertex_uvs.write().unwrap().remove(mesh_id);
        }
    }
    pub fn add_material(&self, material_id: &MaterialId, material: &mut Material) {
        inox_profiler::scoped_profile!("render_buffers::add_material");

        let mut textures_indices = [INVALID_INDEX; TextureType::Count as _];
        material
            .textures()
            .iter()
            .enumerate()
            .for_each(|(i, handle_texture)| {
                if let Some(texture) = handle_texture {
                    textures_indices[i] = texture.get().texture_index() as _;
                }
            });
        let mut materials = self.materials.write().unwrap();
        if let Some(m) = materials.get_mut(material_id) {
            m.textures_indices = textures_indices;
        } else {
            let index = materials.insert(
                material_id,
                DrawMaterial {
                    textures_indices,
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
            let mut textures_coord_set: [u32; TextureType::Count as _] = Default::default();
            for (i, t) in material_data.texcoords_set.iter().enumerate() {
                textures_coord_set[i] = *t as _;
            }
            material.textures_coord_set = textures_coord_set;
            material.roughness_factor = material_data.roughness_factor;
            material.metallic_factor = material_data.metallic_factor;
            material.alpha_cutoff = material_data.alpha_cutoff;
            material.alpha_mode = material_data.alpha_mode.into();
            material.base_color = material_data.base_color.into();
            material.emissive_color = material_data.emissive_color.into();
            material.occlusion_strength = material_data.occlusion_strength;
            material.diffuse_color = material_data.diffuse_color.into();
            material.specular_color = material_data.specular_color.into();
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
