use nrg_graphics::{MaterialId, MeshData, MeshId, Renderer, INVALID_ID};
use nrg_math::{Vector2f, Vector2u, Vector4f};
use nrg_serialize::{Deserialize, Serialize};

use crate::{Screen, DEFAULT_LAYER_OFFSET, DEFAULT_WIDGET_SIZE};
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
    color: Vector4f,
    #[serde(skip)]
    border_color: Vector4f,
    #[serde(skip)]
    is_dirty: bool,
    stroke: f32,
    layer: f32,
}

impl Default for WidgetGraphics {
    fn default() -> Self {
        Self {
            material_id: INVALID_ID,
            mesh_id: INVALID_ID,
            mesh_data: MeshData::default(),
            color: Vector4f::default(),
            border_color: Vector4f::default(),
            is_dirty: true,
            stroke: 0.0,
            layer: 1.0 - DEFAULT_LAYER_OFFSET,
        }
    }
}

impl WidgetGraphics {
    pub fn init(&mut self, renderer: &mut Renderer, pipeline: &str) -> &mut Self {
        let pipeline_id = renderer.get_pipeline_id(pipeline);
        self.material_id = renderer.add_material(pipeline_id);

        let zero_px = Screen::convert_from_pixels_into_screen_space(Vector2u::default());
        let one_px = Screen::convert_from_pixels_into_screen_space(DEFAULT_WIDGET_SIZE);

        let mut mesh_data = MeshData::default();
        mesh_data.add_quad_default(
            [zero_px.x, zero_px.y, one_px.x, one_px.y].into(),
            1.0 - DEFAULT_LAYER_OFFSET,
        );
        self.set_mesh_data(mesh_data);

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
    pub fn get_layer(&self) -> f32 {
        self.layer
    }
    pub fn set_layer(&mut self, layer: f32) -> &mut Self {
        self.layer = layer;
        self
    }
    pub fn get_stroke(&self) -> f32 {
        self.stroke
    }
    pub fn get_mesh_id(&mut self) -> MeshId {
        self.mesh_id
    }
    pub fn get_mesh_data(&mut self) -> &mut MeshData {
        &mut self.mesh_data
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.mesh_data = mesh_data;
        self.is_dirty = true;
        self
    }
    pub fn translate(&mut self, offset: Vector2f) -> &mut Self {
        self.mesh_data.translate([offset.x, offset.y, 0.].into());
        self.is_dirty = true;
        self
    }
    pub fn scale(&mut self, scale: Vector2f) -> &mut Self {
        self.mesh_data.scale([scale.x, scale.y, 1.].into());
        self.is_dirty = true;
        self
    }
    pub fn move_to_layer(&mut self, layer: f32) -> &mut Self {
        self.mesh_data.translate([0.0, 0.0, layer].into());
        self.is_dirty = true;
        self
    }
    pub fn get_color(&self) -> Vector4f {
        self.color
    }
    pub fn set_color(&mut self, rgb: Vector4f) -> &mut Self {
        if self.color != rgb {
            self.color = rgb;
            self.mesh_data.set_vertex_color(rgb);
            self.is_dirty = true;
        }
        self
    }
    pub fn set_border_color(&mut self, rgb: Vector4f) -> &mut Self {
        if self.border_color != rgb {
            self.border_color = rgb;
        }
        self
    }
    pub fn set_stroke(&mut self, stroke: f32) -> &mut Self {
        self.stroke = stroke;
        self
    }
    pub fn is_inside(&self, pos_in_px: Vector2u) -> bool {
        let pos_in_screen_space = Screen::convert_from_pixels_into_screen_space(pos_in_px);
        self.mesh_data.is_inside(pos_in_screen_space)
    }

    pub fn update(&mut self, renderer: &mut Renderer, is_visible: bool) -> &mut Self {
        if is_visible {
            if self.is_dirty {
                if self.mesh_id == INVALID_ID {
                    self.mesh_id = renderer.add_mesh(self.material_id, &self.mesh_data);
                } else {
                    renderer.update_mesh(self.material_id, self.mesh_id, &self.mesh_data);
                }
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
