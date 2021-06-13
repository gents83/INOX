use nrg_graphics::{
    MaterialId, MaterialInstance, MeshData, MeshId, MeshInstance, PipelineInstance,
};
use nrg_math::{Deg, Matrix4, Rad, VecBase, Vector2, Vector3, Vector4, Zero};
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
    is_visible: bool,
    position: Vector3,
    rotation: Vector3,
    scale: Vector3,
}

impl WidgetGraphics {
    #[inline]
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
            position: Vector3::default_zero(),
            rotation: Vector3::default_zero(),
            scale: [1., 1., 1.].into(),
        }
    }

    #[inline]
    pub fn load_override(&mut self, shared_data: &SharedDataRw) -> &mut Self {
        self.shared_data = shared_data.clone();
        self
    }
}

impl WidgetGraphics {
    pub fn init(&mut self, pipeline: &str) -> &mut Self {
        let pipeline_id = PipelineInstance::find_id_from_name(&self.shared_data, pipeline);
        self.material_id = MaterialInstance::create_from_pipeline(&self.shared_data, pipeline_id);

        self.create_default_mesh();

        self
    }

    fn create_default_mesh(&mut self) -> &mut Self {
        if self.mesh_id != INVALID_UID {
            self.remove_meshes();
        }
        let mut mesh_data = MeshData::default();
        mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
        self.mesh_id = MeshInstance::create(&self.shared_data, mesh_data);
        MaterialInstance::add_mesh(&self.shared_data, self.material_id, self.mesh_id);
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn link_to_material(&mut self, material_id: MaterialId) -> &mut Self {
        if self.material_id != INVALID_UID {
            MaterialInstance::destroy(&self.shared_data, self.material_id);
        }
        self.material_id = material_id;

        self.create_default_mesh().mark_as_dirty();
        self
    }

    #[inline]
    pub fn unlink_from_material(&mut self) -> &mut Self {
        self.material_id = INVALID_UID;
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn remove_meshes(&mut self) -> &mut Self {
        if self.mesh_id != INVALID_UID {
            MaterialInstance::remove_mesh(&self.shared_data, self.material_id, self.mesh_id);
            MeshInstance::destroy(&self.shared_data, self.mesh_id);
        }
        self.mesh_id = INVALID_UID;
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn mark_as_dirty(&mut self) {
        self.is_dirty = true;
    }

    #[inline]
    pub fn get_stroke(&self) -> f32 {
        self.stroke
    }

    #[inline]
    pub fn get_mesh_id(&self) -> MeshId {
        self.mesh_id
    }

    #[inline]
    pub fn get_material_id(&self) -> MaterialId {
        self.material_id
    }

    #[inline]
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        MeshInstance::set_mesh_data(&self.shared_data, self.mesh_id, mesh_data);
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn get_layer(&self) -> f32 {
        self.position.z
    }

    #[inline]
    pub fn set_layer(&mut self, layer: f32) -> &mut Self {
        if (self.position.z - layer).abs() >= f32::EPSILON {
            self.position.z = layer;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_position(&mut self, pos_in_px: Vector2) -> &mut Self {
        if (self.position.x - pos_in_px.x).abs() >= f32::EPSILON
            || (self.position.y - pos_in_px.y).abs() >= f32::EPSILON
        {
            self.position.x = pos_in_px.x;
            self.position.y = pos_in_px.y;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_size(&mut self, scale: Vector2) -> &mut Self {
        if (self.scale.x - scale.x).abs() >= f32::EPSILON
            || (self.scale.y - scale.y).abs() >= f32::EPSILON
        {
            self.scale.x = scale.x;
            self.scale.y = scale.y;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn get_color(&self) -> Vector4 {
        self.color
    }

    #[inline]
    pub fn set_color(&mut self, rgb: Vector4) -> &mut Self {
        if self.color != rgb {
            self.color = rgb;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_border_color(&mut self, rgb: Vector4) -> &mut Self {
        if self.border_color != rgb {
            self.border_color = rgb;
        }
        self
    }

    #[inline]
    pub fn set_stroke(&mut self, stroke: f32) -> &mut Self {
        self.stroke = stroke;
        self
    }

    #[inline]
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.is_visible = visible;
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    #[inline]
    pub fn update(&mut self, drawing_area: Vector4) -> &mut Self {
        if self.is_dirty && !self.material_id.is_nil() && !self.mesh_id.is_nil() {
            nrg_profiler::scoped_profile!("widget::graphics_update");
            let mut visible = self.is_visible;
            if visible && (self.color.w.is_zero() || drawing_area.z <= 0. || drawing_area.w <= 0.) {
                visible = false;
            }
            MaterialInstance::set_outline_color(
                &self.shared_data,
                self.material_id,
                self.border_color,
            );
            MaterialInstance::set_diffuse_color(&self.shared_data, self.material_id, self.color);

            let transform = Matrix4::from_translation(self.position)
                * Matrix4::from_angle_z(Rad::from(Deg(self.rotation.x)))
                * Matrix4::from_angle_y(Rad::from(Deg(self.rotation.y)))
                * Matrix4::from_angle_z(Rad::from(Deg(self.rotation.z)))
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

            MeshInstance::set_transform(&self.shared_data, self.mesh_id, transform);
            MeshInstance::set_draw_area(&self.shared_data, self.mesh_id, drawing_area);
            MeshInstance::set_visible(&self.shared_data, self.mesh_id, visible);
            self.is_dirty = false;
        }
        self
    }

    #[inline]
    pub fn uninit(&mut self) -> &mut Self {
        self.remove_meshes();
        MaterialInstance::destroy(&self.shared_data, self.material_id);
        self.material_id = INVALID_UID;
        self
    }
}
