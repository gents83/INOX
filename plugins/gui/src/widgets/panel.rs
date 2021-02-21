use nrg_graphics::*;
use nrg_math::*;

use super::widget::*;

const DEFAULT_WIDTH: f32 = 0.5;
const DEFAULT_HEIGHT: f32 = 0.5;

pub struct Panel {
    widget: Widget,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            widget: Widget::default(),
        }
    }
}

impl Panel {
    pub fn update(&self, renderer: &mut Renderer) {
        self.widget.update(renderer);
    }
    pub fn uninit(&mut self, renderer: &mut Renderer) -> &mut Self {
        self.widget.uninit(renderer);
        self
    }

    pub fn get_position(&self) -> Vector2f {
        self.widget.get_position()
    }
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.widget.set_position(x, y);
        self
    }
    pub fn set_size(&mut self, scale_x: f32, scale_y: f32) -> &mut Self {
        self.widget.set_size(scale_x, scale_y);
        self
    }
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.widget.set_color(r, g, b);
        self
    }
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        self.widget.is_inside(x, y)
    }
}

impl Panel {
    pub fn init(&mut self, renderer: &mut Renderer) -> &mut Self {
        let pipeline_id = renderer.get_pipeline_id("UI");
        self.widget.material_id = renderer.add_material(pipeline_id);
        self.widget
            .mesh_data
            .add_quad_default(
                [
                    self.widget.pos.x,
                    self.widget.pos.y,
                    self.widget.pos.x + DEFAULT_WIDTH * 2.0,
                    self.widget.pos.y + DEFAULT_HEIGHT * 2.0,
                ]
                .into(),
            )
            .set_vertex_color([0.3, 0.8, 1.0].into());

        self.widget.mesh_id = renderer.add_mesh(self.widget.material_id, &self.widget.mesh_data);
        self
    }

    pub fn get_widget(&self) -> &Widget {
        &self.widget
    }
}
