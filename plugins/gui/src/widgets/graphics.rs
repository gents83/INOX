use super::state::*;
use super::style::*;
use nrg_graphics::*;
use nrg_math::*;

pub struct WidgetGraphics {
    material_id: MaterialId,
    mesh_id: MeshId,
    mesh_data: MeshData,
    border_mesh_id: MeshId,
    border_mesh_data: MeshData,
    color: Vector4f,
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
            style: WidgetStyle::default(),
            border_style: WidgetStyle::default_border(),
        }
    }
}

impl WidgetGraphics {
    pub fn init(&mut self, renderer: &mut Renderer, pipeline: &str) -> &mut Self {
        let pipeline_id = renderer.get_pipeline_id(pipeline);
        self.material_id = renderer.add_material(pipeline_id);
        self
    }

    pub fn set_style(&mut self, style: WidgetStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn get_colors(&self, state: WidgetInteractiveState) -> (Vector4f, Vector4f) {
        (
            self.style.get_color(state),
            self.border_style.get_color(state),
        )
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
            border_vertex.pos.z += LAYER_OFFSET;
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
            .translate([0.0, 0.0, layer + LAYER_OFFSET].into());
        self
    }
    pub fn is_inside(&self, pos: Vector2f) -> bool {
        let mut i = 0;
        let count = self.mesh_data.indices.len();
        while i < count {
            let v1 = self.mesh_data.vertices[self.mesh_data.indices[i] as usize].pos;
            let v2 = self.mesh_data.vertices[self.mesh_data.indices[i + 1] as usize].pos;
            let v3 = self.mesh_data.vertices[self.mesh_data.indices[i + 2] as usize].pos;
            if is_point_in_triangle(v1.into(), v2.into(), v3.into(), pos.x, pos.y) {
                return true;
            }
            i += 3;
        }
        false
    }

    pub fn update(&mut self, renderer: &mut Renderer, clip_area: Vector4f) -> &mut Self {
        self.mesh_data.clip_in_rect(clip_area);
        self.border_mesh_data.clip_in_rect(clip_area);

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
        renderer.remove_mesh(self.material_id, self.border_mesh_id);
        renderer.remove_mesh(self.material_id, self.mesh_id);
        renderer.remove_material(self.material_id);
        self.material_id = INVALID_ID;
        self.border_mesh_id = INVALID_ID;
        self.border_mesh_data.clear();
        self.mesh_id = INVALID_ID;
        self.mesh_data.clear();
        self
    }
}
