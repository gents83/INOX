use nrg_graphics::{MaterialId, MeshData, MeshId, Renderer, INVALID_ID};
use nrg_math::{MatBase, Matrix4, VecBase, Vector2, Vector3, Vector4};
use nrg_serialize::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetGraphics {
    #[serde(skip)]
    material_id: MaterialId,
    #[serde(skip)]
    mesh_id: MeshId,
    #[serde(skip)]
    mesh_data: MeshData,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    color: Vector4,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    border_color: Vector4,
    #[serde(skip)]
    is_dirty: bool,
    stroke: f32,
    #[serde(skip, default = "nrg_math::MatBase::default_identity")]
    transform: Matrix4,
}

impl Default for WidgetGraphics {
    fn default() -> Self {
        Self {
            material_id: INVALID_ID,
            mesh_id: INVALID_ID,
            mesh_data: MeshData::default(),
            color: Vector4::default_zero(),
            border_color: Vector4::default_zero(),
            is_dirty: true,
            stroke: 0.,
            transform: Matrix4::default_identity(),
        }
    }
}

impl WidgetGraphics {
    pub fn init(&mut self, renderer: &mut Renderer, pipeline: &str) -> &mut Self {
        let pipeline_id = renderer.get_pipeline_id(pipeline);
        self.material_id = renderer.add_material(pipeline_id);

        let mut mesh_data = MeshData::default();
        mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
        self.set_mesh_data(renderer, mesh_data);
        self
    }
    pub fn link_to_material(&mut self, material_id: MaterialId) -> &mut Self {
        self.material_id = material_id;
        self.is_dirty = true;
        self
    }
    pub fn unlink_from_material(&mut self) -> &mut Self {
        self.material_id = INVALID_ID;
        self.is_dirty = true;
        self
    }
    pub fn remove_meshes(&mut self, renderer: &mut Renderer) -> &mut Self {
        renderer.remove_mesh(self.material_id, self.mesh_id);
        self.mesh_id = INVALID_ID;
        self.is_dirty = true;
        self
    }
    pub fn get_stroke(&self) -> f32 {
        self.stroke
    }
    pub fn get_mesh_id(&mut self) -> MeshId {
        self.mesh_id
    }
    pub fn set_mesh_data(&mut self, renderer: &mut Renderer, mesh_data: MeshData) -> &mut Self {
        self.remove_meshes(renderer);
        self.mesh_data = mesh_data;
        self.is_dirty = true;
        self
    }
    pub fn get_layer(&self) -> f32 {
        self.transform.w[2]
    }
    pub fn set_layer(&mut self, layer: f32) -> &mut Self {
        if (self.transform.w[2] - layer).abs() >= f32::EPSILON {
            self.transform.w[2] = layer;
            self.is_dirty = true;
        }
        self
    }
    pub fn set_position(&mut self, pos_in_px: Vector2) -> &mut Self {
        if (self.transform.w[0] - pos_in_px.x).abs() >= f32::EPSILON
            || (self.transform.w[1] - pos_in_px.y).abs() >= f32::EPSILON
        {
            self.transform.w[0] = pos_in_px.x;
            self.transform.w[1] = pos_in_px.y;
            self.is_dirty = true;
        }
        self
    }
    pub fn set_size(&mut self, scale: Vector2) -> &mut Self {
        if (self.transform.x[0] - scale.x).abs() >= f32::EPSILON
            || (self.transform.y[1] - scale.y).abs() >= f32::EPSILON
        {
            let pos_in_px: Vector3 = [
                self.transform.w[0],
                self.transform.w[1],
                self.transform.w[2],
            ]
            .into();
            self.transform = Matrix4::from_nonuniform_scale(scale.x, scale.y, 1.);
            self.transform.w[0] = pos_in_px.x;
            self.transform.w[1] = pos_in_px.y;
            self.transform.w[2] = pos_in_px.z;
            self.is_dirty = true;
        }
        self
    }
    pub fn get_color(&self) -> Vector4 {
        self.color
    }
    pub fn set_color(&mut self, rgb: Vector4) -> &mut Self {
        if self.color != rgb {
            self.color = rgb;
            self.is_dirty = true;
        }
        self
    }
    pub fn set_border_color(&mut self, rgb: Vector4) -> &mut Self {
        if self.border_color != rgb {
            self.border_color = rgb;
        }
        self
    }
    pub fn set_stroke(&mut self, stroke: f32) -> &mut Self {
        self.stroke = stroke;
        self
    }

    pub fn update(&mut self, renderer: &mut Renderer, is_visible: bool) -> &mut Self {
        if is_visible {
            if self.is_dirty && !self.mesh_data.vertices.is_empty() {
                if self.mesh_id == INVALID_ID {
                    self.mesh_id = renderer.add_mesh(self.material_id, &self.mesh_data);
                }
                if let Some(mesh_data) = renderer.get_mesh(self.material_id, self.mesh_id) {
                    mesh_data.set_vertex_color(self.color);
                    renderer.update_mesh(self.material_id, self.mesh_id, &self.transform);
                }
                self.is_dirty = false;
            }
        } else if self.mesh_id != INVALID_ID {
            self.remove_meshes(renderer);
        }
        self
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) -> &mut Self {
        self.remove_meshes(renderer);
        renderer.remove_material(self.material_id);
        self.material_id = INVALID_ID;
        self
    }
}
