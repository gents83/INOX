use nrg_graphics::{
    MaterialId, MaterialInstance, MeshData, MeshId, MeshInstance, PipelineInstance,
};
use nrg_math::{MatBase, Matrix4, VecBase, Vector2, Vector3, Vector4, Zero};
use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize, INVALID_UID};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetGraphics {
    #[serde(skip)]
    shared_data: SharedDataRw,
    #[serde(skip)]
    material_id: MaterialId,
    #[serde(skip)]
    mesh_id: MeshId,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    color: Vector4,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    border_color: Vector4,
    stroke: f32,
    #[serde(skip)]
    is_dirty: bool,
    #[serde(skip)]
    is_visible: bool,
    #[serde(skip, default = "nrg_math::MatBase::default_identity")]
    transform: Matrix4,
}

impl WidgetGraphics {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        Self {
            shared_data: shared_data.clone(),
            material_id: INVALID_UID,
            mesh_id: INVALID_UID,
            color: Vector4::default_zero(),
            border_color: Vector4::default_zero(),
            is_visible: true,
            is_dirty: true,
            stroke: 0.,
            transform: Matrix4::default_identity(),
        }
    }
}

impl WidgetGraphics {
    pub fn init(&mut self, pipeline: &str) -> &mut Self {
        let pipeline_id = PipelineInstance::find_id_from_name(&self.shared_data, pipeline);
        self.material_id = MaterialInstance::create_from_pipeline(&self.shared_data, pipeline_id);

        let mut mesh_data = MeshData::default();
        mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
        self.mesh_id = MeshInstance::create(&self.shared_data, mesh_data);
        MaterialInstance::add_mesh(&self.shared_data, self.material_id, self.mesh_id);
        self.is_dirty = true;

        self
    }
    pub fn link_to_material(&mut self, material_id: MaterialId) -> &mut Self {
        if self.material_id != INVALID_UID {
            MaterialInstance::destroy(&self.shared_data, self.material_id);
        }
        self.material_id = material_id;
        self.is_dirty = true;
        self
    }
    pub fn unlink_from_material(&mut self) -> &mut Self {
        self.material_id = INVALID_UID;
        self.is_dirty = true;
        self
    }
    pub fn remove_meshes(&mut self) -> &mut Self {
        if self.mesh_id != INVALID_UID {
            MaterialInstance::remove_mesh(&self.shared_data, self.material_id, self.mesh_id);
            MeshInstance::destroy(&self.shared_data, self.mesh_id);
        }
        self.mesh_id = INVALID_UID;
        self.is_dirty = true;
        self
    }
    pub fn get_stroke(&self) -> f32 {
        self.stroke
    }
    pub fn get_mesh_id(&mut self) -> MeshId {
        self.mesh_id
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.remove_meshes();
        self.mesh_id = MeshInstance::create(&self.shared_data, mesh_data);
        MaterialInstance::add_mesh(&self.shared_data, self.material_id, self.mesh_id);
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

    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.is_visible = visible;
        self.is_dirty = true;
        self
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn update(&mut self) -> &mut Self {
        if self.is_dirty {
            let mut visible = self.is_visible;
            if visible && self.color.w.is_zero() {
                visible = false;
            }
            MaterialInstance::set_diffuse_color(&self.shared_data, self.material_id, self.color);
            MeshInstance::set_transform(&self.shared_data, self.mesh_id, self.transform);
            MeshInstance::set_visible(&self.shared_data, self.mesh_id, visible);
            self.is_dirty = false;
        }
        self
    }

    pub fn uninit(&mut self) -> &mut Self {
        self.remove_meshes();
        MaterialInstance::destroy(&self.shared_data, self.material_id);
        self.material_id = INVALID_UID;
        self
    }
}
