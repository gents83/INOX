use std::path::PathBuf;

use inox_math::{is_point_in_triangle, VecBase, Vector2, Vector3, Vector4};

use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::{DrawVertex, MAX_TEXTURE_COORDS_SETS};

#[repr(C, align(16))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshletData {
    pub center: Vector3,
    pub radius: f32,
    pub cone_axis: Vector3,
    pub cone_cutoff: f32,
    pub vertices_count: u32,
    pub vertices_offset: u32,
    pub indices_count: u32,
    pub indices_offset: u32,
}

impl Default for MeshletData {
    fn default() -> Self {
        Self {
            center: Vector3::default_zero(),
            radius: 0.0,
            cone_axis: Vector3::default_zero(),
            cone_cutoff: 0.0,
            vertices_count: 0,
            vertices_offset: 0,
            indices_count: 0,
            indices_offset: 0,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshData {
    pub positions: Vec<Vector3>,
    pub colors: Vec<Vector4>,
    pub normals: Vec<Vector3>,
    pub tangents: Vec<Vector4>,
    pub uvs: Vec<Vector2>,
    pub vertices: Vec<DrawVertex>,
    pub indices: Vec<u32>,
    pub material: PathBuf,
    pub meshlets: Vec<MeshletData>,
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
        self.tangents.clear();
        self.uvs.clear();
        self.meshlets.clear();
        self.indices.clear();
        self
    }

    pub fn add_vertex_pos_color(&mut self, p: Vector3, c: Vector4) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            ..Default::default()
        };
        self.positions.push(p);
        self.colors.push(c);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_color_normal(&mut self, p: Vector3, c: Vector4, n: Vector3) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            normal_offset: self.normals.len() as _,
            ..Default::default()
        };
        self.positions.push(p);
        self.colors.push(c);
        self.normals.push(n);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_uv(&mut self, p: Vector3, uv: Vector2) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            uv_offset: [self.uvs.len() as _; MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        };
        self.positions.push(p);
        self.uvs.push(uv);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn add_vertex_pos_color_uv(&mut self, p: Vector3, c: Vector4, uv: Vector2) -> usize {
        let vertex = DrawVertex {
            position_and_color_offset: self.positions.len() as _,
            uv_offset: [self.uvs.len() as _; MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        };
        self.positions.push(p);
        self.colors.push(c);
        self.uvs.push(uv);
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
        self.positions.push(p);
        self.colors.push(c);
        self.normals.push(n);
        self.uvs.push(uv);
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
    pub fn append_mesh_data_as_meshlet(&mut self, mut mesh_data: MeshData) {
        let vertex_offset = self.vertex_count() as u32;
        let index_offset = self.index_count() as u32;

        let meshlet = MeshletData {
            vertices_offset: vertex_offset as _,
            vertices_count: mesh_data.vertex_count() as _,
            indices_offset: index_offset as _,
            indices_count: mesh_data.index_count() as _,
            ..Default::default()
        };
        self.meshlets.push(meshlet);

        self.positions.append(&mut mesh_data.positions);
        self.colors.append(&mut mesh_data.colors);
        self.normals.append(&mut mesh_data.normals);
        self.tangents.append(&mut mesh_data.tangents);
        self.uvs.append(&mut mesh_data.uvs);
        self.vertices.append(&mut mesh_data.vertices);
        mesh_data
            .indices
            .iter()
            .for_each(|i| self.indices.push(*i + vertex_offset));
    }

    pub fn compute_min_max(&self) -> (Vector3, Vector3) {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for i in 0..self.positions.len() {
            let pos = &self.positions[i];
            min.x = min.x.min(pos[0]);
            min.y = min.y.min(pos[1]);
            min.z = min.z.min(pos[2]);
            max.x = max.x.max(pos[0]);
            max.y = max.y.max(pos[1]);
            max.z = max.z.max(pos[2]);
        }
        (min, max)
    }

    pub fn compute_center(&self) -> Vector3 {
        let (min, max) = self.compute_min_max();
        [
            min.x + (max.x - min.x) * 0.5,
            min.y + (max.y - min.y) * 0.5,
            min.z + (max.z - min.z) * 0.5,
        ]
        .into()
    }

    pub fn set_vertex_color(&mut self, color: Vector4) -> &mut Self {
        self.colors.iter_mut().for_each(|c| *c = color);
        self
    }

    pub fn clip_in_rect(&mut self, clip_rect: Vector4) -> &mut Self {
        for i in 0..self.positions.len() {
            let pos = &mut self.positions[i];
            pos[0] = pos[0].max(clip_rect.x);
            pos[0] = pos[0].min(clip_rect.z);
            pos[1] = pos[1].max(clip_rect.y);
            pos[1] = pos[1].min(clip_rect.w);
        }
        self.compute_center();
        self
    }

    pub fn is_inside(&self, pos_in_screen_space: Vector2) -> bool {
        let mut i = 0;
        let count = self.indices.len();
        while i < count {
            let v1 = [
                self.positions[self.indices[i] as usize][0],
                self.positions[self.indices[i] as usize][1],
            ]
            .into();
            let v2 = [
                self.positions[self.indices[i + 1] as usize][0],
                self.positions[self.indices[i + 1] as usize][1],
            ]
            .into();
            let v3 = [
                self.positions[self.indices[i + 2] as usize][0],
                self.positions[self.indices[i + 2] as usize][1],
            ]
            .into();
            if is_point_in_triangle(v1, v2, v3, pos_in_screen_space.x, pos_in_screen_space.y) {
                return true;
            }
            i += 3;
        }
        false
    }
}
