use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock,
    },
};

use inox_bvh::{create_linearized_bvh, BVHTree, GPUBVHNode, AABB};
use inox_math::{quantize_half, Mat4Ops, MatBase, Matrix4, VecBase, Vector2};
use inox_resources::{to_slice, Buffer, ResourceId};
use inox_uid::{generate_static_uid_from_string, generate_uid_from_type, Uid};

use crate::{
    platform::has_primitive_index_support, AsBinding, ConstantDataRw, GPULight, GPUMaterial,
    GPUMesh, GPUMeshlet, GPUPrimitiveIndices, GPUTexture, GPUVertexAttributes, GPUVertexIndices,
    GPUVertexPosition, Light, LightId, Material, MaterialData, MaterialFlags, MaterialId, Mesh,
    MeshData, MeshFlags, MeshId, RenderContext, TextureId, TextureType, INVALID_INDEX,
    MAX_LOD_LEVELS,
};

pub const TLAS_UID: ResourceId = generate_static_uid_from_string("TLAS");
pub const LUT_PBR_CHARLIE_UID: ResourceId = generate_static_uid_from_string("LUT_PBR_CHARLIE_UID");
pub const LUT_PBR_GGX_UID: ResourceId = generate_static_uid_from_string("LUT_PBR_GGX_UID");
pub const ENV_MAP_UID: ResourceId = generate_static_uid_from_string("ENV_MAP_UID");

pub const ATOMIC_SIZE: u32 = 32;
pub const NUM_FRAMES_OF_HISTORY: usize = 2;

pub type AtomicCounter = Arc<RwLock<AtomicU32>>;

pub trait GPUDataBuffer {}

impl<T> GPUDataBuffer for Buffer<T> where T: Clone + Default + 'static {}
pub trait GPUDataVector {}

impl<T> GPUDataVector for Vec<T> where T: Clone {}

pub type GPUBuffer<T> = Arc<RwLock<Buffer<T>>>;
pub type GPUVector<T> = Arc<RwLock<Vec<T>>>;
pub type DynGPUBuffer = Arc<RwLock<dyn GPUDataBuffer>>;
pub type DynGPUVector = Arc<RwLock<dyn GPUDataVector>>;
pub type DynGPUBufferMap = HashMap<Uid, DynGPUBuffer>;
pub type DynGPUVectorMap = HashMap<Uid, DynGPUVector>;

//Alignment should be 4, 8, 16 or 32 bytes
#[derive(Default)]
pub struct GlobalBuffers {
    pub constant_data: ConstantDataRw,
    pub tlas_start_index: AtomicCounter,
    pub buffers: Arc<RwLock<DynGPUBufferMap>>,
    pub vectors: Arc<RwLock<DynGPUVectorMap>>,
}
unsafe impl Send for GlobalBuffers {}
unsafe impl Sync for GlobalBuffers {}

impl GlobalBuffers {
    pub fn buffer_with_id<T>(&self, id: Uid) -> Arc<RwLock<Buffer<T>>>
    where
        T: Clone + Default + 'static,
    {
        if let Some(buffer) = self.buffers.read().unwrap().get(&id) {
            let any = Arc::into_raw(buffer.clone());
            return unsafe { Arc::from_raw(any as *const RwLock<Buffer<T>>) };
        }
        let buffer = Arc::new(RwLock::new(Buffer::<T>::default()));
        self.buffers.write().unwrap().insert(id, buffer.clone());
        buffer
    }
    pub fn vector_with_id<T>(&self, id: Uid) -> Arc<RwLock<Vec<T>>>
    where
        T: Clone + 'static,
    {
        if let Some(vector) = self.vectors.read().unwrap().get(&id) {
            let any = Arc::into_raw(vector.clone());
            return unsafe { Arc::from_raw(any as *const RwLock<Vec<T>>) };
        }
        let vector = Arc::new(RwLock::new(Vec::<T>::default()));
        self.vectors.write().unwrap().insert(id, vector.clone());
        vector
    }
    pub fn buffer<T>(&self) -> Arc<RwLock<Buffer<T>>>
    where
        T: Clone + Default + 'static,
    {
        let id = generate_uid_from_type::<T>();
        self.buffer_with_id(id)
    }
    pub fn vector<T>(&self) -> Arc<RwLock<Vec<T>>>
    where
        T: Clone + 'static,
    {
        let id = generate_uid_from_type::<T>();
        self.vector_with_id(id)
    }
}

impl GlobalBuffers {
    fn extract_meshlets(
        &self,
        render_context: &RenderContext,
        mesh_data: &MeshData,
        mesh_id: &MeshId,
        mesh_index: u32,
        indices_offset: u32,
    ) -> (usize, usize, Vec<usize>, Vec<usize>) {
        inox_profiler::scoped_profile!("render_buffers::extract_meshlets");

        let mut meshlets = Vec::new();
        let mut lod_meshlets_count = Vec::new();
        mesh_data
            .meshlets
            .iter()
            .enumerate()
            .for_each(|(lod_level, meshlets_data)| {
                lod_meshlets_count.push(meshlets_data.len());
                meshlets_data.iter().for_each(|meshlet_data| {
                    let meshlet = GPUMeshlet {
                        mesh_index,
                        lod_level: (MAX_LOD_LEVELS - 1 - lod_level) as u32,
                        indices_offset: (indices_offset + meshlet_data.indices_offset) as _,
                        indices_count: meshlet_data.indices_count,
                        aabb_min: meshlet_data.aabb_min.into(),
                        aabb_max: meshlet_data.aabb_max.into(),
                        parent_error: meshlet_data.parent_error,
                        group_error: meshlet_data.error,
                        bounding_sphere: meshlet_data.bounding_sphere.into(),
                        parent_bounding_sphere: meshlet_data.parent_bounding_sphere.into(),
                    };
                    meshlets.push(meshlet);
                });
            });
        let mut lod_meshlets_starting_offset = Vec::with_capacity(lod_meshlets_count.len());
        let mut lod_meshlets_ending_offset = Vec::with_capacity(lod_meshlets_count.len());
        let mut total_offset = 0;
        lod_meshlets_count.iter().for_each(|&count| {
            lod_meshlets_starting_offset.push(total_offset);
            total_offset += count;
            lod_meshlets_ending_offset.push(total_offset);
        });
        if meshlets.is_empty() {
            inox_log::debug_log!("No meshlet data for mesh {:?}", mesh_id);
        }
        let mesh_bvh_range = self
            .buffer::<GPUBVHNode>()
            .write()
            .unwrap()
            .allocate(mesh_id, mesh_data.meshlets_bvh.last().unwrap())
            .1;
        let blas_index = mesh_bvh_range.start as _;
        self.buffer::<GPUBVHNode>().write().unwrap().data_mut()[mesh_bvh_range]
            .iter_mut()
            .for_each(|n| {
                if n.miss >= 0 {
                    n.miss += blas_index as i32;
                }
            });
        self.buffer::<GPUBVHNode>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);

        let meshlet_range = self
            .buffer::<GPUMeshlet>()
            .write()
            .unwrap()
            .allocate(mesh_id, meshlets.as_slice())
            .1;
        let meshlet_start_index = meshlet_range.start as i32;
        self.buffer::<GPUMeshlet>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);

        lod_meshlets_starting_offset.reverse();
        lod_meshlets_ending_offset.reverse();
        (
            blas_index,
            meshlet_start_index as _,
            lod_meshlets_starting_offset,
            lod_meshlets_ending_offset,
        )
    }
    fn add_vertex_data(
        &self,
        render_context: &RenderContext,
        mesh_id: &MeshId,
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
            .buffer::<GPUVertexPosition>()
            .write()
            .unwrap()
            .allocate(mesh_id, to_slice(mesh_data.vertex_positions.as_slice()))
            .1
            .start;
        self.buffer::<GPUVertexPosition>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
        let attributes_offset = self
            .buffer::<GPUVertexAttributes>()
            .write()
            .unwrap()
            .allocate(mesh_id, to_slice(mesh_data.vertex_attributes.as_slice()))
            .1
            .start;
        self.buffer::<GPUVertexAttributes>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
        let indices_offset = self
            .buffer::<GPUVertexIndices>()
            .write()
            .unwrap()
            .allocate(
                mesh_id,
                mesh_data
                    .indices
                    .iter()
                    .map(|v| GPUVertexIndices(*v))
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .1
            .start;
        self.buffer::<GPUVertexIndices>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
        if !has_primitive_index_support() {
            let mut primitive_indices = Vec::with_capacity(mesh_data.indices.len());
            for (new_index, _i) in mesh_data.indices.iter().enumerate() {
                primitive_indices.push(GPUPrimitiveIndices(new_index as _));
            }
            self.buffer::<GPUPrimitiveIndices>()
                .write()
                .unwrap()
                .allocate(mesh_id, &primitive_indices);
            self.buffer::<GPUPrimitiveIndices>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
        }
        (
            vertex_offset as _,
            indices_offset as _,
            attributes_offset as _,
        )
    }
    pub fn add_mesh(
        &self,
        render_context: &RenderContext,
        mesh_id: &MeshId,
        mesh_data: &MeshData,
    ) -> usize {
        inox_profiler::scoped_profile!("render_buffers::add_mesh");
        self.remove_mesh(render_context, mesh_id, false);
        if mesh_data.vertex_count() == 0 {
            return INVALID_INDEX as _;
        }
        let mesh_index = self
            .buffer::<GPUMesh>()
            .write()
            .unwrap()
            .push(mesh_id, GPUMesh::default())
            .1
            .start;

        let (vertex_offset, indices_offset, attributes_offset) =
            self.add_vertex_data(render_context, mesh_id, mesh_data);
        let (blas_index, meshlet_offset, lod_meshlets_starting_offset, lod_meshlets_ending_offsets) =
            self.extract_meshlets(
                render_context,
                mesh_data,
                mesh_id,
                mesh_index as _,
                indices_offset,
            );

        {
            let meshes = self.buffer::<GPUMesh>();
            let mut meshes = meshes.write().unwrap();
            let mesh = meshes.get_first_mut(mesh_id).unwrap();
            mesh.vertices_position_offset = vertex_offset;
            mesh.vertices_attribute_offset = attributes_offset;
            mesh.flags_and_vertices_attribute_layout = mesh_data.vertex_layout.into();
            mesh.blas_index = blas_index as _;
            mesh.indices_offset = indices_offset as _;
            mesh.indices_count = mesh_data.indices.len() as _;
            mesh.meshlets_offset = meshlet_offset as _;
            mesh.lods_meshlets_offset
                .iter_mut()
                .enumerate()
                .for_each(|(i, c)| {
                    if i < lod_meshlets_starting_offset.len()
                        && i < lod_meshlets_ending_offsets.len()
                    {
                        *c = ((lod_meshlets_starting_offset[i]) << 16
                            | (lod_meshlets_ending_offsets[i])) as _;
                    }
                });
        }
        self.buffer::<GPUMesh>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
        self.recreate_tlas(render_context);
        mesh_index
    }
    pub fn recreate_tlas(&self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("render_buffers::recreate_tlas");
        let mut meshes_aabbs = Vec::new();
        {
            let meshes = self.buffer::<GPUMesh>();
            let meshes = meshes.read().unwrap();
            let bvh = self.buffer::<GPUBVHNode>();
            let bvh = bvh.read().unwrap();
            let bvh = bvh.data();
            meshes.for_each_data(|i, _id, mesh| {
                let node = &bvh[mesh.blas_index as usize];
                //let matrix = Matrix4::from_translation_orientation_scale(
                //    mesh.position.into(),
                //    mesh.orientation.into(),
                //    mesh.scale.into(),
                //);
                let matrix = Matrix4::default_identity();
                let min = matrix.rotate_point(node.min.into());
                let max = matrix.rotate_point(node.max.into());
                let aabb = AABB::create(min.min(max), max.max(min), i as _);
                meshes_aabbs.push(aabb);
            });
        }
        let bvh = BVHTree::new(&meshes_aabbs);
        let linearized_bvh = create_linearized_bvh(&bvh);
        let bvh = self.buffer::<GPUBVHNode>();
        let mut bvh = bvh.write().unwrap();
        let tlas_range = bvh.allocate(&TLAS_UID, &linearized_bvh).1;
        let tlas_starting_index = tlas_range.start as _;
        self.tlas_start_index
            .write()
            .unwrap()
            .store(tlas_starting_index, Ordering::SeqCst);
        bvh.data_mut()[tlas_range].iter_mut().for_each(|n| {
            if n.miss >= 0 {
                n.miss += tlas_starting_index as i32;
            }
        });
        bvh.mark_as_dirty(render_context);
        //println!("\n\nTLAS: {}", tlas_starting_index);
        //print_bvh(bvh.data());
    }
    pub fn change_mesh(&self, render_context: &RenderContext, mesh_id: &MeshId, mesh: &mut Mesh) {
        inox_profiler::scoped_profile!("render_buffers::change_mesh");
        let meshes = self.buffer::<GPUMesh>();
        let mut meshes = meshes.write().unwrap();
        if let Some(m) = meshes.get_first_mut(mesh_id) {
            if let Some(material) = mesh.material() {
                if let Some(data) = self
                    .buffer::<GPUMaterial>()
                    .read()
                    .unwrap()
                    .indices(material.id())
                {
                    m.material_index = data.range().start as _;
                }
                if let Some(material) = self
                    .buffer::<GPUMaterial>()
                    .write()
                    .unwrap()
                    .get_first_mut(material.id())
                {
                    let flags: MaterialFlags = material.flags.into();
                    if flags.contains(MaterialFlags::AlphaModeBlend) || material.base_color[3] < 1.
                    {
                        mesh.remove_flag(MeshFlags::Opaque);
                        mesh.add_flag(MeshFlags::Tranparent);
                    }
                }
            }

            let mesh_flags = mesh.flags();
            let vertex_attribute_layout = m.flags_and_vertices_attribute_layout & 0x0000FFFF;
            let flags: u32 = (*mesh_flags).into();
            m.flags_and_vertices_attribute_layout = vertex_attribute_layout | (flags << 16);
            meshes.mark_as_dirty(render_context);
        }
    }
    pub fn remove_mesh(
        &self,
        render_context: &RenderContext,
        mesh_id: &MeshId,
        recreate_tlas: bool,
    ) {
        inox_profiler::scoped_profile!("render_buffers::remove_mesh");

        if self.buffer::<GPUMesh>().write().unwrap().remove(mesh_id) {
            self.buffer::<GPUMesh>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
            self.buffer::<GPUMeshlet>().write().unwrap().remove(mesh_id);
            self.buffer::<GPUMeshlet>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
            self.buffer::<GPUBVHNode>().write().unwrap().remove(mesh_id);
            self.buffer::<GPUBVHNode>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
            self.buffer::<GPUVertexIndices>()
                .write()
                .unwrap()
                .remove(mesh_id);
            self.buffer::<GPUVertexIndices>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
            self.buffer::<GPUVertexPosition>()
                .write()
                .unwrap()
                .remove(mesh_id);
            self.buffer::<GPUVertexPosition>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
            self.buffer::<GPUVertexAttributes>()
                .write()
                .unwrap()
                .remove(mesh_id);
            self.buffer::<GPUVertexAttributes>()
                .write()
                .unwrap()
                .mark_as_dirty(render_context);
        }
        if recreate_tlas {
            self.recreate_tlas(render_context);
        }
    }
    pub fn add_material(
        &self,
        render_context: &RenderContext,
        material_id: &MaterialId,
        material: &mut Material,
    ) {
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
        let materials = self.buffer::<GPUMaterial>();
        let mut materials = materials.write().unwrap();
        if let Some(m) = materials.get_first_mut(material_id) {
            for (v, item) in textures_index_and_coord_set.iter().enumerate() {
                let i = v & 3; // Equivalent to index % 4 (faster than modulo on power of 2)
                let j = v >> 2; // Equivalent to index / 4 (faster than division by power of 2)
                m.textures_index_and_coord_set[j][i] = f32::from_bits(*item);
            }
        } else {
            let mut m = GPUMaterial::default();
            for (v, item) in textures_index_and_coord_set.iter().enumerate() {
                let i = v & 3; // Equivalent to index % 4 (faster than modulo on power of 2)
                let j = v >> 2; // Equivalent to index / 4 (faster than division by power of 2)
                m.textures_index_and_coord_set[j][i] = f32::from_bits(*item);
            }
            let index = materials.push(material_id, m).1.start;
            material.set_material_index(index as _);
        }
        materials.mark_as_dirty(render_context);
    }
    pub fn update_material(
        &self,
        render_context: &RenderContext,
        material_id: &MaterialId,
        material_data: &MaterialData,
    ) {
        inox_profiler::scoped_profile!("render_buffers::update_material");
        let materials = self.buffer::<GPUMaterial>();
        let mut materials = materials.write().unwrap();
        if let Some(material) = materials.get_first_mut(material_id) {
            for (v, t) in material_data.texcoords_set.iter().enumerate() {
                let i = v & 3; // Equivalent to index % 4 (faster than modulo on power of 2)
                let j = v >> 2; // Equivalent to index / 4 (faster than division by power of 2)
                let mut v = f32::to_bits(material.textures_index_and_coord_set[j][i]);
                v |= (*t << 28) as u32;
                material.textures_index_and_coord_set[j][i] = f32::from_bits(v);
            }
            material.roughness_factor = material_data.roughness_factor;
            material.metallic_factor = material_data.metallic_factor;
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
            material.normal_scale_and_alpha_cutoff = quantize_half(material_data.normal_scale)
                as u32
                | (quantize_half(material_data.alpha_cutoff) as u32) << 16;
            material.emissive_strength = material_data.emissive_strength;
            material.flags = material_data.flags.into();
            materials.mark_as_dirty(render_context);
        }
    }
    pub fn remove_material(&self, render_context: &RenderContext, material_id: &MaterialId) {
        inox_profiler::scoped_profile!("render_buffers::remove_material");

        self.buffer::<GPUMaterial>()
            .write()
            .unwrap()
            .remove(material_id);
        self.buffer::<GPUMaterial>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
    }

    pub fn add_light(&self, render_context: &RenderContext, light_id: &LightId, light: &mut Light) {
        inox_profiler::scoped_profile!("render_buffers::add_light");

        let index = self
            .buffer::<GPULight>()
            .write()
            .unwrap()
            .push(light_id, GPULight::default())
            .1
            .start;
        light.set_light_index(index as _);

        let mut constant_data = self.constant_data.write().unwrap();
        let num_lights = constant_data.num_lights();
        constant_data.set_num_lights(render_context, num_lights + 1);
        self.buffer::<GPULight>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
    }
    pub fn update_light(
        &self,
        render_context: &RenderContext,
        light_id: &LightId,
        light_data: &GPULight,
    ) {
        inox_profiler::scoped_profile!("render_buffers::update_light");
        let lights = self.buffer::<GPULight>();
        let mut lights = lights.write().unwrap();
        if let Some(light) = lights.get_first_mut(light_id) {
            *light = *light_data;
            lights.mark_as_dirty(render_context);
        }
    }
    pub fn remove_light(&self, render_context: &RenderContext, light_id: &LightId) {
        inox_profiler::scoped_profile!("render_buffers::remove_light");

        self.buffer::<GPULight>().write().unwrap().remove(light_id);
        let mut constant_data = self.constant_data.write().unwrap();
        let num_lights = constant_data.num_lights();
        constant_data.set_num_lights(render_context, num_lights - 1);

        self.buffer::<GPULight>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
    }

    pub fn add_texture(
        &self,
        render_context: &RenderContext,
        texture_id: &TextureId,
        texture_data: &GPUTexture,
        lut_id: &Uid,
    ) -> usize {
        inox_profiler::scoped_profile!("render_buffers::add_texture");

        let uniform_index = self
            .buffer::<GPUTexture>()
            .write()
            .unwrap()
            .push(texture_id, *texture_data)
            .1
            .start;
        if !lut_id.is_nil() {
            self.constant_data
                .write()
                .unwrap()
                .set_LUT(render_context, lut_id, uniform_index as _);
        }

        self.buffer::<GPUTexture>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
        uniform_index
    }
    pub fn remove_texture(&self, render_context: &RenderContext, texture_id: &TextureId) {
        inox_profiler::scoped_profile!("render_buffers::remove_texture");

        self.buffer::<GPUTexture>()
            .write()
            .unwrap()
            .remove(texture_id);

        self.buffer::<GPUTexture>()
            .write()
            .unwrap()
            .mark_as_dirty(render_context);
    }

    pub fn update_constant_data(
        &self,
        render_context: &RenderContext,
        view_proj_near_far_fov: (Matrix4, Matrix4, f32, f32, f32),
        screen_size: Vector2,
        debug_coords: Vector2,
    ) {
        inox_profiler::scoped_profile!("render_context::update_constant_data");
        self.constant_data.write().unwrap().update(
            render_context,
            view_proj_near_far_fov,
            (screen_size, debug_coords),
            self.tlas_start_index
                .read()
                .unwrap()
                .load(Ordering::Relaxed),
        );
    }
}
