use nrg_graphics::*;
use nrg_math::*;

const DEFAULT_WIDTH: f32 = 0.5;
const DEFAULT_HEIGHT: f32 = 0.5;

pub struct Panel {
    pos: Vector2f,
    material_id: MaterialId,
    mesh_id: MeshId,
    mesh_data: MeshData,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            pos: Vector2f::default(),
            material_id: INVALID_ID,
            mesh_id: INVALID_ID,
            mesh_data: MeshData::default(),
        }
    }
}

impl Panel {
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        let pos: Vector2f = Vector2f { x, y } * 2.0 - [1.0, 1.0].into();
        self.mesh_data.translate(pos - self.pos);
        self.pos = pos;
        self
    }
    pub fn set_size(&mut self, scale_x: f32, scale_y: f32) -> &mut Self {
        let original_pos = self.pos;
        let scale = Vector2f {
            x: scale_x,
            y: scale_y,
        } * 2.0;
        self.mesh_data.translate(-original_pos);
        self.mesh_data.scale(scale);
        self.mesh_data.translate(original_pos);
        self
    }
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.mesh_data.set_vertex_color([r, g, b].into());
        self
    }

    pub fn update(&self, renderer: &mut Renderer) {
        renderer.update_mesh(self.material_id, self.mesh_id, &self.mesh_data);
    }

    pub fn init(&mut self, renderer: &mut Renderer) -> &mut Self {
        let pipeline_id = renderer.get_pipeline_id("UI");
        self.material_id = renderer.add_material(pipeline_id);
        self.mesh_data
            .add_quad_default(
                [
                    self.pos.x,
                    self.pos.y,
                    self.pos.x + DEFAULT_WIDTH * 2.0,
                    self.pos.y + DEFAULT_HEIGHT * 2.0,
                ]
                .into(),
            )
            .set_vertex_color([0.3, 0.8, 1.0].into());

        self.mesh_id = renderer.add_mesh(self.material_id, &self.mesh_data);
        self
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) -> &mut Self {
        renderer.remove_mesh(self.material_id, self.mesh_id);
        renderer.remove_material(self.material_id);
        self.material_id = INVALID_ID;
        self.mesh_id = INVALID_ID;
        self.mesh_data.clear();
        self
    }
}
