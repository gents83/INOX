use nrg_graphics::{
    MaterialInstance, MaterialRc, MeshData, MeshInstance, MeshRc, PipelineInstance,
};
use nrg_math::{Deg, Matrix4, Rad, VecBase, Vector2, Vector3, Vector4, Zero};
use nrg_resources::{DataTypeResource, ResourceRef, SharedDataRw};
use nrg_serialize::{Deserialize, Serialize, INVALID_UID};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetGraphics {
    #[serde(skip)]
    shared_data: SharedDataRw,
    #[serde(skip, default = "nrg_resources::ResourceRef::default")]
    material: MaterialRc,
    #[serde(skip, default = "nrg_resources::ResourceRef::default")]
    mesh: MeshRc,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    color: Vector4,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    border_color: Vector4,
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
            material: ResourceRef::default(),
            mesh: ResourceRef::default(),
            color: Vector4::default_zero(),
            border_color: Vector4::default_zero(),
            is_visible: true,
            is_dirty: true,
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
        if let Some(pipeline) = PipelineInstance::find_from_name(&self.shared_data, pipeline) {
            self.material = MaterialInstance::create_from_pipeline(&self.shared_data, pipeline);
        }

        self.create_default_mesh();

        self
    }

    fn create_default_mesh(&mut self) -> &mut Self {
        if self.mesh.id() != INVALID_UID {
            self.remove_meshes();
        }
        let mut mesh_data = MeshData::default();
        mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
        self.mesh = MeshInstance::create_from_data(&self.shared_data, mesh_data);
        self.mesh
            .resource()
            .get_mut()
            .set_material(self.material.clone());
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn link_to_material(&mut self, material: MaterialRc) -> &mut Self {
        self.material = material;
        self.create_default_mesh().mark_as_dirty();
        self
    }

    #[inline]
    pub fn unlink_from_material(&mut self) -> &mut Self {
        self.material = MaterialRc::default();
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn remove_meshes(&mut self) -> &mut Self {
        self.mesh = MeshRc::default();
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn mark_as_dirty(&mut self) {
        self.is_dirty = true;
    }

    #[inline]
    pub fn get_stroke(&self) -> f32 {
        self.border_color.w
    }

    #[inline]
    pub fn get_mesh(&self) -> MeshRc {
        self.mesh.clone()
    }

    #[inline]
    pub fn get_material(&self) -> MaterialRc {
        self.material.clone()
    }

    #[inline]
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.mesh.resource().get_mut().set_mesh_data(mesh_data);
        self.mark_as_dirty();
        self
    }

    #[inline]
    pub fn get_layer(&self) -> f32 {
        self.position.z
    }

    #[inline]
    pub fn set_layer(&mut self, layer: f32) -> &mut Self {
        if (self.position.z - layer).abs() > f32::EPSILON {
            self.position.z = layer;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_position(&mut self, pos_in_px: Vector2) -> &mut Self {
        if (self.position.x - pos_in_px.x).abs() > f32::EPSILON
            || (self.position.y - pos_in_px.y).abs() > f32::EPSILON
        {
            self.position.x = pos_in_px.x;
            self.position.y = pos_in_px.y;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_size(&mut self, scale: Vector2) -> &mut Self {
        if (self.scale.x - scale.x).abs() > f32::EPSILON
            || (self.scale.y - scale.y).abs() > f32::EPSILON
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
        if self.border_color.xyz() != rgb.xyz() {
            self.border_color.x = rgb.x;
            self.border_color.y = rgb.y;
            self.border_color.z = rgb.z;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_stroke(&mut self, stroke: f32) -> &mut Self {
        if (self.border_color.w as f32 - stroke).abs() > f32::EPSILON {
            self.border_color.w = stroke;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        if self.is_visible != visible {
            self.is_visible = visible;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_visible && !self.color.w.is_zero()
    }

    #[inline]
    pub fn is_valid_drawing_area(drawing_area: Vector4) -> bool {
        drawing_area.z > 0. && drawing_area.w > 0.
    }

    #[inline]
    pub fn update(&mut self, drawing_area: Vector4) -> &mut Self {
        if self.is_dirty && !self.material.id().is_nil() && !self.mesh.id().is_nil() {
            nrg_profiler::scoped_profile!("widget::graphics_update");
            let visible = self.is_visible() && WidgetGraphics::is_valid_drawing_area(drawing_area);

            self.material
                .resource()
                .get_mut()
                .set_outline_color(self.border_color);
            self.material
                .resource()
                .get_mut()
                .set_diffuse_color(self.color);

            let transform = Matrix4::from_translation(self.position)
                * Matrix4::from_angle_z(Rad::from(Deg(self.rotation.x)))
                * Matrix4::from_angle_y(Rad::from(Deg(self.rotation.y)))
                * Matrix4::from_angle_z(Rad::from(Deg(self.rotation.z)))
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

            self.mesh.resource().get_mut().set_transform(transform);
            self.mesh.resource().get_mut().set_draw_area(drawing_area);
            self.mesh.resource().get_mut().set_visible(visible);
            self.is_dirty = false;
        }
        self
    }

    #[inline]
    pub fn uninit(&mut self) -> &mut Self {
        self.remove_meshes();
        self.material = ResourceRef::default();
        self
    }
}
