use std::path::PathBuf;

use inox_math::{decode_unorm, quantize_half, quantize_unorm, VecBase, Vector2, Vector3, Vector4};

use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::{DrawVertex, MAX_TEXTURE_COORDS_SETS};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshletData {
    pub aabb_min: Vector3,
    pub indices_offset: u32,
    pub aabb_max: Vector3,
    pub indices_count: u32,
    pub cone_center: Vector3,
    pub cone_axis: Vector3,
    pub cone_angle: f32,
}

impl Default for MeshletData {
    fn default() -> Self {
        Self {
            aabb_min: Vector3::new(f32::MAX, f32::MAX, f32::MAX),
            aabb_max: Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX),
            cone_center: Vector3::default_zero(),
            cone_axis: Vector3::default_zero(),
            cone_angle: 0.,
            indices_offset: 0,
            indices_count: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshData {
    pub aabb_min: Vector3,
    pub aabb_max: Vector3,
    pub positions: Vec<u32>, // u32 (10 x, 10 y, 10 z, 2 null)
    pub colors: Vec<u32>,    //rgba
    pub normals: Vec<u32>,   // u32 (10 x, 10 y, 10 z, 2 null)
    pub uvs: Vec<u32>,       // 2 half - f16
    pub vertices: Vec<DrawVertex>,
    pub indices: Vec<u32>,
    pub material: PathBuf,
    pub meshlets: Vec<MeshletData>,
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
            positions: Vec::new(),
            colors: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            material: PathBuf::default(),
            meshlets: Vec::new(),
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
        self.vertices.len()
    }
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
    pub fn clear(&mut self) -> &mut Self {
        self.vertices.clear();
        self.positions.clear();
        self.colors.clear();
        self.normals.clear();
        self.uvs.clear();
        self.meshlets.clear();
        self.indices.clear();
        self
    }

    pub fn position(&self, i: usize) -> Vector3 {
        let size = self.aabb_max - self.aabb_min;
        let p = self.positions[i];
        let px = decode_unorm((p >> 20) & 0x000003FF, 10);
        let py = decode_unorm((p >> 10) & 0x000003FF, 10);
        let pz = decode_unorm(p & 0x000003FF, 10);
        Vector3 {
            x: self.aabb_min.x + size.x * px,
            y: self.aabb_min.y + size.y * py,
            z: self.aabb_min.z + size.z * pz,
        }
    }

    fn insert_position(&mut self, p: Vector3) {
        let old_size = self.aabb_max - self.aabb_min;
        let new_max = self.aabb_max.max(p);
        let new_min = self.aabb_min.min(p);
        let new_size = new_max - new_min;
        if new_max != self.aabb_max || new_min != self.aabb_min || new_size != old_size {
            self.positions.iter_mut().for_each(|p| {
                let px = decode_unorm((*p >> 20) & 0x000003FF, 10);
                let py = decode_unorm((*p >> 10) & 0x000003FF, 10);
                let pz = decode_unorm(*p & 0x000003FF, 10);
                let pos = Vector3 {
                    x: self.aabb_min.x + old_size.x * px,
                    y: self.aabb_min.y + old_size.y * py,
                    z: self.aabb_min.z + old_size.z * pz,
                };

                let mut v = pos - new_min;
                v.x /= new_size.x;
                v.y /= new_size.y;
                v.z /= new_size.z;
                let vx = quantize_unorm(v.x, 10);
                let vy = quantize_unorm(v.y, 10);
                let vz = quantize_unorm(v.z, 10);
                let new_p = vx << 20 | vy << 10 | vz;
                *p = new_p;
            });
        }

        let mut v = p - new_min;
        v.x /= new_size.x;
        v.y /= new_size.y;
        v.z /= new_size.z;
        let vx = quantize_unorm(v.x, 10);
        let vy = quantize_unorm(v.y, 10);
        let vz = quantize_unorm(v.z, 10);
        let new_p = vx << 20 | vy << 10 | vz;
        self.positions.push(new_p);

        self.aabb_max = new_max;
        self.aabb_min = new_min;
    }

    fn insert_normal(&mut self, n: Vector3) {
        let nx = quantize_unorm(n.x, 10);
        let ny = quantize_unorm(n.y, 10);
        let nz = quantize_unorm(n.z, 10);
        self.normals.push(nx << 20 | ny << 10 | nz);
    }

    fn insert_color(&mut self, c: Vector4) {
        let r = quantize_unorm(c.x, 8);
        let g = quantize_unorm(c.y, 8);
        let b = quantize_unorm(c.z, 8);
        let a = quantize_unorm(c.w, 8);
        self.colors.push(r << 24 | g << 16 | b << 8 | a);
    }

    fn insert_uv(&mut self, uv: Vector2) {
        let u = quantize_half(uv.x) as u32;
        let v = (quantize_half(uv.y) as u32) << 16;
        self.uvs.push(u | v);
    }

    pub fn add_vertex_pos_color(&mut self, p: Vector3, c: Vector4) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            ..Default::default()
        };
        self.insert_position(p);
        self.insert_color(c);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_color_normal(&mut self, p: Vector3, c: Vector4, n: Vector3) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            normal_offset: self.normals.len() as _,
            ..Default::default()
        };
        self.insert_position(p);
        self.insert_color(c);
        self.insert_normal(n);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_uv(&mut self, p: Vector3, uv: Vector2) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            uv_offset: [self.uvs.len() as _; MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        };
        self.insert_position(p);
        self.insert_uv(uv);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_color_uv(&mut self, p: Vector3, c: Vector4, uv: Vector2) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            uv_offset: [self.uvs.len() as _; MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        };
        self.insert_position(p);
        self.insert_color(c);
        self.insert_uv(uv);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_color_normal_uv(
        &mut self,
        p: Vector3,
        c: Vector4,
        n: Vector3,
        uv: Vector2,
    ) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            normal_offset: self.normals.len() as _,
            uv_offset: [self.uvs.len() as _; MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        };
        self.insert_position(p);
        self.insert_color(c);
        self.insert_normal(n);
        self.insert_uv(uv);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }

    pub fn append_mesh_data(&mut self, mut mesh_data: MeshData, as_separate_meshlet: bool) {
        let vertex_offset = self.vertex_count() as u32;
        let index_offset = self.index_count() as u32;
        let position_offset = self.positions.len() as u32;
        let normals_offset = self.normals.len() as u32;
        let uvs_offset = self.uvs.len() as u32;

        if as_separate_meshlet || self.meshlets.is_empty() {
            let meshlet = MeshletData {
                indices_offset: index_offset as _,
                indices_count: mesh_data.index_count() as _,
                aabb_min: mesh_data.aabb_min(),
                aabb_max: mesh_data.aabb_max(),
                ..Default::default()
            };
            self.meshlets.push(meshlet);
        } else {
            let meshlet = self.meshlets.last_mut().unwrap();
            meshlet.indices_count += mesh_data.index_count() as u32;
        }

        let size = mesh_data.aabb_max - mesh_data.aabb_min;
        self.positions
            .reserve(self.positions.len() + mesh_data.positions.len());
        mesh_data.positions.iter().for_each(|p| {
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
        self.colors.append(&mut mesh_data.colors);
        self.normals.append(&mut mesh_data.normals);
        self.uvs.append(&mut mesh_data.uvs);
        self.vertices
            .reserve(self.vertices.len() + mesh_data.vertices.len());
        mesh_data.vertices.iter_mut().for_each(|v| {
            v.position_and_color_offset += position_offset;
            v.normal_offset += normals_offset as i32;
            v.uv_offset.iter_mut().for_each(|uv| {
                *uv += uvs_offset as i32;
            });
            self.vertices.push(*v);
        });
        self.indices
            .reserve(self.vertices.len() + mesh_data.indices.len());
        mesh_data
            .indices
            .iter()
            .for_each(|i| self.indices.push(*i + vertex_offset));
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

        self.colors.iter_mut().for_each(|old_c| *old_c = c);
        self
    }
}
