use inox_math::{MatBase, Matrix4};
use inox_resources::{Buffer, HashBuffer, ResourceId};

use crate::{
    DrawMaterial, DrawMesh, DrawMeshlet, DrawVertex, Light, LightData, LightId, Material,
    MaterialAlphaMode, MaterialData, MaterialId, Mesh, MeshData, MeshId, TextureData, TextureId,
    TextureType, INVALID_INDEX, MAX_TEXTURE_COORDS_SETS, MESH_FLAGS_NONE, MESH_FLAGS_OPAQUE,
    MESH_FLAGS_TRANSPARENT,
};

#[derive(Default)]
pub struct RenderBuffers {
    pub textures: HashBuffer<TextureId, TextureData, 0>,
    pub lights: HashBuffer<LightId, LightData, 0>,
    pub materials: HashBuffer<MaterialId, DrawMaterial, 0>,
    pub meshes: HashBuffer<MeshId, DrawMesh, 0>,
    pub matrix: HashBuffer<ResourceId, [[f32; 4]; 4], 0>,
    pub meshlets: Buffer<DrawMeshlet>, //MeshId <-> [DrawMeshlet]
    pub vertices: Buffer<DrawVertex>,  //MeshId <-> [DrawVertex]
    pub indices: Buffer<u32>,          //MeshId <-> [u32]
    pub vertex_positions: Buffer<[f32; 3]>, //MeshId <-> [f32; 3]
    pub vertex_colors: Buffer<u32>,    //MeshId <-> u32
    pub vertex_normals: Buffer<[f32; 3]>, //MeshId <-> [f32; 3]
    pub vertex_tangents: Buffer<[f32; 4]>, //MeshId <-> [f32; 4]
    pub vertex_uvs: [Buffer<[f32; 2]>; MAX_TEXTURE_COORDS_SETS], //MeshId <-> [[f32; 2]; 4]
}

impl RenderBuffers {
    fn extract_meshlets(mesh_data: &MeshData) -> Vec<DrawMeshlet> {
        inox_profiler::scoped_profile!("render_buffers::extract_meshlets");

        let mut meshlets = Vec::new();
        mesh_data.meshlets.iter().for_each(|meshlet_data| {
            let meshlet = DrawMeshlet {
                vertex_offset: meshlet_data.vertices_offset,
                vertex_count: meshlet_data.vertices_count,
                indices_offset: meshlet_data.indices_offset,
                indices_count: meshlet_data.indices_count,
                center_radius: [
                    meshlet_data.center.x,
                    meshlet_data.center.y,
                    meshlet_data.center.z,
                    meshlet_data.radius,
                ],
                cone_axis_cutoff: [
                    meshlet_data.cone_axis.x,
                    meshlet_data.cone_axis.y,
                    meshlet_data.cone_axis.z,
                    meshlet_data.cone_cutoff,
                ],
            };
            meshlets.push(meshlet);
        });
        meshlets
    }
    fn add_vertex_data(&mut self, mesh_id: &MeshId, mesh_data: &MeshData) {
        inox_profiler::scoped_profile!("render_buffers::add_vertex_data");

        if mesh_data.vertices.is_empty() {
            inox_log::debug_log!("No vertices for mesh {:?}", mesh_id);
            return;
        }
        self.vertices
            .allocate(mesh_id, mesh_data.vertices.as_slice());
        if !mesh_data.positions.is_empty() {
            self.vertex_positions
                .allocate(mesh_id, mesh_data.positions.as_slice());
        }
        if !mesh_data.colors.is_empty() {
            self.vertex_colors
                .allocate(mesh_id, mesh_data.colors.as_slice());
        }
        if !mesh_data.normals.is_empty() {
            self.vertex_normals
                .allocate(mesh_id, mesh_data.normals.as_slice());
        }
        if !mesh_data.tangents.is_empty() {
            self.vertex_tangents
                .allocate(mesh_id, mesh_data.tangents.as_slice());
        }
        for i in 0..MAX_TEXTURE_COORDS_SETS {
            if !mesh_data.uvs[i].is_empty() {
                self.vertex_uvs[i].allocate(mesh_id, mesh_data.uvs[i].as_slice());
            }
        }
        if !mesh_data.indices.is_empty() {
            self.indices.allocate(mesh_id, mesh_data.indices.as_slice());
        }
    }
    pub fn add_mesh(&mut self, mesh_id: &MeshId, mesh_data: &MeshData) {
        inox_profiler::scoped_profile!("render_buffers::add_mesh");

        self.add_vertex_data(mesh_id, mesh_data);
        let meshlets = Self::extract_meshlets(mesh_data);
        if meshlets.is_empty() {
            inox_log::debug_log!("No meshlet data for mesh {:?}", mesh_id);
            return;
        }
        let range = self.meshlets.allocate(mesh_id, meshlets.as_slice()).1;
        let draw_mesh = DrawMesh {
            meshlet_offset: range.start as _,
            meshlet_count: meshlets.len() as _,
            material_index: INVALID_INDEX,
            mesh_flags: MESH_FLAGS_NONE,
            matrix_index: self.add_matrix(mesh_id) as _,
        };
        self.meshes.insert(mesh_id, draw_mesh);
    }
    pub fn change_mesh(&mut self, mesh_id: &MeshId, mesh: &mut Mesh) {
        inox_profiler::scoped_profile!("render_buffers::change_mesh");

        if let Some(m) = self.meshes.get_mut(mesh_id) {
            if let Some(material) = mesh.material() {
                if let Some(index) = self.materials.index(material.id()) {
                    m.material_index = index as _;
                }
                if let Some(material) = self.materials.get_mut(material.id()) {
                    let blend_alpha_mode: u32 = MaterialAlphaMode::Blend.into();
                    if material.alpha_mode == blend_alpha_mode || material.base_color[3] < 1. {
                        mesh.remove_flag(MESH_FLAGS_OPAQUE);
                        mesh.add_flag(MESH_FLAGS_TRANSPARENT);
                    } else {
                        mesh.remove_flag(MESH_FLAGS_TRANSPARENT);
                        mesh.add_flag(MESH_FLAGS_OPAQUE);
                    }
                }
            }
            m.mesh_flags = mesh.flags();

            self.update_matrix(mesh_id, &mesh.matrix());
        }
    }
    pub fn remove_mesh(&mut self, mesh_id: &MeshId) {
        inox_profiler::scoped_profile!("render_buffers::remove_mesh");

        self.remove_matrix(mesh_id);
        self.meshes.remove(mesh_id);
        self.meshlets.remove(mesh_id);
        self.vertices.remove(mesh_id);
        self.indices.remove(mesh_id);
        self.vertex_positions.remove(mesh_id);
        self.vertex_colors.remove(mesh_id);
        self.vertex_normals.remove(mesh_id);
        self.vertex_tangents.remove(mesh_id);
        for i in 0..MAX_TEXTURE_COORDS_SETS {
            self.vertex_uvs[i].remove(mesh_id);
        }
    }
    pub fn add_material(&mut self, material_id: &MaterialId, material: &mut Material) {
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
        let index = self.materials.insert(
            material_id,
            DrawMaterial {
                textures_indices,
                ..Default::default()
            },
        );
        material.set_material_index(index as _);
    }
    pub fn update_material(&mut self, material_id: &MaterialId, material_data: &MaterialData) {
        inox_profiler::scoped_profile!("render_buffers::update_material");

        if let Some(material) = self.materials.get_mut(material_id) {
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
        }
    }
    pub fn remove_material(&mut self, material_id: &MaterialId) {
        inox_profiler::scoped_profile!("render_buffers::remove_material");

        self.materials.remove(material_id);
    }

    pub fn add_light(&mut self, light_id: &LightId, light: &mut Light) {
        inox_profiler::scoped_profile!("render_buffers::add_light");

        let index = self.lights.insert(light_id, LightData::default());
        light.set_light_index(index as _);
    }
    pub fn update_light(&mut self, light_id: &LightId, light_data: &LightData) {
        inox_profiler::scoped_profile!("render_buffers::update_light");

        if let Some(light) = self.lights.get_mut(light_id) {
            *light = *light_data;
        }
    }
    pub fn remove_light(&mut self, light_id: &LightId) {
        inox_profiler::scoped_profile!("render_buffers::remove_light");

        self.lights.remove(light_id);
    }

    pub fn add_texture(&mut self, texture_id: &TextureId, texture_data: &TextureData) -> usize {
        inox_profiler::scoped_profile!("render_buffers::add_texture");

        self.textures.insert(texture_id, *texture_data)
    }
    pub fn remove_texture(&mut self, texture_id: &TextureId) {
        inox_profiler::scoped_profile!("render_buffers::remove_texture");

        self.textures.remove(texture_id);
    }

    pub fn add_matrix(&mut self, id: &ResourceId) -> usize {
        inox_profiler::scoped_profile!("render_buffers::add_matrix");

        self.matrix.insert(id, Matrix4::default_identity().into())
    }
    pub fn update_matrix(&mut self, id: &ResourceId, matrix: &Matrix4) {
        inox_profiler::scoped_profile!("render_buffers::update_matrix");

        if let Some(m) = self.matrix.get_mut(id) {
            *m = (*matrix).into();
        }
    }
    pub fn remove_matrix(&mut self, id: &ResourceId) {
        inox_profiler::scoped_profile!("render_buffers::remove_matrix");

        self.matrix.remove(id);
    }
}
