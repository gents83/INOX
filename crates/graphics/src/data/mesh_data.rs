use std::path::PathBuf;

use inox_bvh::{create_linearized_bvh, BVHTree, GPUBVHNode, AABB};
use inox_math::{
    decode_unorm, pack_4_f32_to_snorm, quantize_half, quantize_snorm, quantize_unorm, VecBase,
    Vector2, Vector3, Vector4,
};

use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::VertexAttributeLayout;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshletData {
    pub aabb_min: Vector3,
    pub indices_offset: u32,
    pub aabb_max: Vector3,
    pub indices_count: u32,
    pub child_meshlets: Vec<u32>,
    pub bhv_offset: u32,
}

impl Default for MeshletData {
    fn default() -> Self {
        Self {
            aabb_min: Vector3::new(f32::MAX, f32::MAX, f32::MAX),
            aabb_max: Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX),
            indices_offset: 0,
            indices_count: 0,
            child_meshlets: Vec::default(),
            bhv_offset: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshData {
    pub aabb_min: Vector3,
    pub aabb_max: Vector3,
    pub vertex_positions: Vec<u32>,
    pub vertex_layout: VertexAttributeLayout,
    pub vertex_attributes: Vec<u32>,
    pub indices: Vec<u32>,
    pub material: PathBuf,
    pub meshlets: Vec<Vec<MeshletData>>,
    pub meshlets_bvh: Vec<Vec<GPUBVHNode>>,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            aabb_min: Vector3 {
                x: f32::INFINITY,
                y: f32::INFINITY,
                z: f32::INFINITY,
            },
            aabb_max: Vector3 {
                x: -f32::INFINITY,
                y: -f32::INFINITY,
                z: -f32::INFINITY,
            },
            vertex_positions: Vec::new(),
            vertex_layout: VertexAttributeLayout::default(),
            vertex_attributes: Vec::new(),
            indices: Vec::new(),
            material: PathBuf::default(),
            meshlets: vec![Vec::new()],
            meshlets_bvh: vec![Vec::new()],
        }
    }
}

impl SerializeFile for MeshData {
    fn extension() -> &'static str {
        "mesh"
    }
}

impl MeshData {
    pub fn vertex_count(&self) -> usize {
        self.vertex_positions.len()
    }
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
    pub fn clear(&mut self) -> &mut Self {
        self.vertex_positions.clear();
        self.vertex_attributes.clear();
        self.meshlets.clear();
        self.indices.clear();
        self
    }

    pub fn insert_position(&mut self, p: Vector3) {
        let size = self.aabb_max - self.aabb_min;
        let mut v = p - self.aabb_min;
        v.x /= size.x;
        v.y /= size.y;
        v.z /= size.z;
        let vx = quantize_unorm(v.x, 10);
        let vy = quantize_unorm(v.y, 10);
        let vz = quantize_unorm(v.z, 10);
        let new_p = vx << 20 | vy << 10 | vz;
        self.vertex_positions.push(new_p);
    }

    pub fn insert_normal(&mut self, n: Vector3) {
        let nx = quantize_snorm(n.x, 10);
        let ny = quantize_snorm(n.y, 10);
        let nz = quantize_snorm(n.z, 10);
        self.vertex_attributes.push(nx << 20 | ny << 10 | nz);
    }

    pub fn insert_tangent(&mut self, t: Vector4) {
        let v = pack_4_f32_to_snorm(t);
        self.vertex_attributes.push(v);
    }

    pub fn insert_color(&mut self, c: Vector4) {
        let r = quantize_unorm(c.x, 8);
        let g = quantize_unorm(c.y, 8);
        let b = quantize_unorm(c.z, 8);
        let a = quantize_unorm(c.w, 8);
        self.vertex_attributes.push(r << 24 | g << 16 | b << 8 | a);
    }

    pub fn insert_uv(&mut self, uv: Vector2) {
        let u = quantize_half(uv.x) as u32;
        let v = (quantize_half(uv.y) as u32) << 16;
        self.vertex_attributes.push(u | v);
    }

    pub fn add_vertex_pos_color(&mut self, p: Vector3, c: Vector4) -> &mut Self {
        debug_assert!(
            self.vertex_layout == VertexAttributeLayout::pos_color(),
            "Adding a vertex to a mesh with a different vertex layout"
        );
        self.insert_position(p);
        self.insert_color(c);
        self
    }
    pub fn add_vertex_pos_color_normal(&mut self, p: Vector3, c: Vector4, n: Vector3) -> &mut Self {
        debug_assert!(
            self.vertex_layout == VertexAttributeLayout::pos_color_normal(),
            "Adding a vertex to a mesh with a different vertex layout"
        );
        self.insert_position(p);
        self.insert_color(c);
        self.insert_normal(n);
        self
    }
    pub fn add_vertex_pos_color_normal_uv(
        &mut self,
        p: Vector3,
        c: Vector4,
        n: Vector3,
        uv: Vector2,
    ) -> &mut Self {
        debug_assert!(
            self.vertex_layout == VertexAttributeLayout::pos_color_normal_uv1(),
            "Adding a vertex to a mesh with a different vertex layout"
        );
        self.insert_position(p);
        self.insert_color(c);
        self.insert_normal(n);
        self.insert_uv(uv);
        self
    }

    pub fn append_mesh_data(
        &mut self,
        mut mesh_data: MeshData,
        lod_level: usize,
        as_separate_meshlet: bool,
    ) -> &mut Self {
        debug_assert!(
            self.vertex_layout == mesh_data.vertex_layout,
            "Appending a mesh_data to a mesh with a different vertex layout"
        );
        let vertex_offset = self.vertex_count() as u32;
        let index_offset = self.index_count() as u32;

        if as_separate_meshlet || self.meshlets.is_empty() {
            let meshlet = MeshletData {
                indices_offset: index_offset as _,
                indices_count: mesh_data.index_count() as _,
                aabb_min: mesh_data.aabb_min(),
                aabb_max: mesh_data.aabb_max(),
                ..Default::default()
            };
            self.meshlets[lod_level].push(meshlet);
        } else {
            let meshlet = self.meshlets[lod_level].last_mut().unwrap();
            meshlet.indices_count += mesh_data.index_count() as u32;
        }

        self.aabb_min = self.aabb_min.min(mesh_data.aabb_min);
        self.aabb_max = self.aabb_min.max(mesh_data.aabb_max);
        let size = mesh_data.aabb_max - mesh_data.aabb_min;
        self.vertex_positions
            .reserve(self.vertex_positions.len() + mesh_data.vertex_positions.len());
        mesh_data.vertex_positions.iter().for_each(|p| {
            let px = decode_unorm((p >> 20) & 0x000003FF, 10);
            let py = decode_unorm((p >> 10) & 0x000003FF, 10);
            let pz = decode_unorm(p & 0x000003FF, 10);
            let pos = Vector3 {
                x: mesh_data.aabb_min.x + size.x * px,
                y: mesh_data.aabb_min.y + size.y * py,
                z: mesh_data.aabb_min.z + size.z * pz,
            };
            self.insert_position(pos);
        });
        self.vertex_attributes
            .append(&mut mesh_data.vertex_attributes);
        self.indices
            .reserve(self.indices.len() + mesh_data.indices.len());
        mesh_data
            .indices
            .iter()
            .for_each(|i| self.indices.push(*i + vertex_offset));

        let mut meshlets_aabbs = Vec::new();
        meshlets_aabbs.resize_with(self.meshlets.len(), AABB::empty);
        self.meshlets[lod_level]
            .iter()
            .enumerate()
            .for_each(|(i, m)| {
                meshlets_aabbs[i] = AABB::create(m.aabb_min, m.aabb_max, i as _);
            });
        let bvh = BVHTree::new(&meshlets_aabbs);
        self.meshlets_bvh[lod_level] = create_linearized_bvh(&bvh);

        self
    }

    pub fn aabb_min(&self) -> Vector3 {
        self.aabb_min
    }
    pub fn aabb_max(&self) -> Vector3 {
        self.aabb_max
    }

    pub fn compute_center(&self) -> Vector3 {
        let min = self.aabb_min();
        let max = self.aabb_max();
        [
            min.x + (max.x - min.x) * 0.5,
            min.y + (max.y - min.y) * 0.5,
            min.z + (max.z - min.z) * 0.5,
        ]
        .into()
    }

    pub fn set_vertex_color(&mut self, color: Vector4) -> &mut Self {
        let r = quantize_unorm(color.x, 8);
        let g = quantize_unorm(color.y, 8);
        let b = quantize_unorm(color.z, 8);
        let a = quantize_unorm(color.w, 8);
        let c = r << 24 | g << 16 | b << 8 | a;

        let stride_in_count = self.vertex_layout.stride_in_count()
            + self.vertex_layout.offset(VertexAttributeLayout::HasColor);
        self.vertex_attributes
            .iter_mut()
            .step_by(stride_in_count)
            .for_each(|old_c| *old_c = c);
        self
    }

    pub fn position(&self, i: usize) -> Vector3 {
        let size = self.aabb_max - self.aabb_min;
        let p = self.vertex_positions[i];
        let px = decode_unorm((p >> 20) & 0x000003FF, 10);
        let py = decode_unorm((p >> 10) & 0x000003FF, 10);
        let pz = decode_unorm(p & 0x000003FF, 10);
        Vector3 {
            x: self.aabb_min.x + size.x * px,
            y: self.aabb_min.y + size.y * py,
            z: self.aabb_min.z + size.z * pz,
        }
    }

    pub fn packed_color(&self, i: usize) -> u32 {
        let index = i * self.vertex_layout.stride_in_count()
            + self.vertex_layout.offset(VertexAttributeLayout::HasColor);
        self.vertex_attributes[index]
    }
}
