use super::*;
use crate::screen::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetGraphics {
    #[serde(skip)]
    material_id: MaterialId,
    #[serde(skip)]
    mesh_id: MeshId,
    #[serde(skip)]
    mesh_data: MeshData,
    #[serde(skip)]
    border_mesh_id: MeshId,
    #[serde(skip)]
    border_mesh_data: MeshData,
    #[serde(skip)]
    color: Vector4f,
    #[serde(skip)]
    border_color: Vector4f,
    stroke: f32,
    style: WidgetStyle,
    border_style: WidgetStyle,
}

impl Default for WidgetGraphics {
    fn default() -> Self {
        Self {
            material_id: INVALID_ID,
            mesh_id: INVALID_ID,
            mesh_data: MeshData::default(),
            border_mesh_id: INVALID_ID,
            border_mesh_data: MeshData::default(),
            color: Vector4f::default(),
            border_color: Vector4f::default(),
            stroke: 0.0,
            style: WidgetStyle::Default,
            border_style: WidgetStyle::DefaultBorder,
        }
    }
}

impl WidgetGraphics {
    pub fn init(&mut self, renderer: &mut Renderer, pipeline: &str) -> &mut Self {
        let pipeline_id = renderer.get_pipeline_id(pipeline);
        self.material_id = renderer.add_material(pipeline_id);
        self
    }
    pub fn link_to_material(&mut self, material_id: MaterialId) -> &mut Self {
        self.material_id = material_id;
        self
    }
    pub fn unlink_from_material(&mut self) -> &mut Self {
        self.material_id = INVALID_ID;
        self
    }
    pub fn remove_meshes(&mut self, renderer: &mut Renderer) -> &mut Self {
        renderer.remove_mesh(self.material_id, self.border_mesh_id);
        renderer.remove_mesh(self.material_id, self.mesh_id);
        self.border_mesh_id = INVALID_ID;
        self.border_mesh_data.clear();
        self.mesh_id = INVALID_ID;
        self.mesh_data.clear();
        self
    }

    pub fn set_style(&mut self, style: WidgetStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_border_style(&mut self, style: WidgetStyle) -> &mut Self {
        self.border_style = style;
        self
    }

    pub fn get_colors(&self, state: WidgetInteractiveState) -> (Vector4f, Vector4f) {
        (
            WidgetStyle::color(&self.style, state),
            WidgetStyle::color(&self.border_style, state),
        )
    }

    pub fn get_stroke(&self) -> f32 {
        self.stroke
    }

    fn compute_border(&mut self) -> &mut Self {
        if self.stroke <= 0.0 {
            return self;
        }
        let center = self.mesh_data.center;
        self.border_mesh_data = MeshData::default();
        for v in self.mesh_data.vertices.iter() {
            let mut dir = (v.pos - center).normalize();
            dir.x = dir.x.signum();
            dir.y = dir.y.signum();
            let mut border_vertex = v.clone();
            border_vertex.pos += dir * self.stroke;
            border_vertex.pos.z += DEFAULT_LAYER_OFFSET;
            border_vertex.color = self.border_color;
            self.border_mesh_data.vertices.push(border_vertex);
        }
        self.border_mesh_data.indices = self.mesh_data.indices.clone();
        self.border_mesh_data.compute_center();
        self
    }

    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.mesh_data = mesh_data;
        self.compute_border();
        self
    }
    pub fn get_color(&self) -> Vector4f {
        self.color
    }
    pub fn set_color(&mut self, rgb: Vector4f) -> &mut Self {
        self.color = rgb;
        self
    }
    pub fn set_border_color(&mut self, rgb: Vector4f) -> &mut Self {
        self.border_color = rgb;
        self
    }
    pub fn set_stroke(&mut self, stroke: f32) -> &mut Self {
        self.stroke = stroke;
        self
    }
    pub fn move_to_layer(&mut self, layer: f32) -> &mut Self {
        self.mesh_data.translate([0.0, 0.0, layer].into());
        self.border_mesh_data
            .translate([0.0, 0.0, layer + DEFAULT_LAYER_OFFSET].into());
        self
    }
    pub fn is_inside(&self, pos_in_px: Vector2u) -> bool {
        let pos_in_screen_space = Screen::convert_from_pixels_into_screen_space(pos_in_px);
        self.mesh_data.is_inside(pos_in_screen_space)
    }

    pub fn update(&mut self, renderer: &mut Renderer) -> &mut Self {
        if self.border_mesh_id == INVALID_ID && self.stroke > 0.0 {
            self.border_mesh_id = renderer.add_mesh(self.material_id, &self.border_mesh_data);
        } else {
            renderer.update_mesh(
                self.material_id,
                self.border_mesh_id,
                &self.border_mesh_data,
            );
        }
        if self.mesh_id == INVALID_ID {
            self.mesh_id = renderer.add_mesh(self.material_id, &self.mesh_data);
        } else {
            renderer.update_mesh(self.material_id, self.mesh_id, &self.mesh_data);
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
