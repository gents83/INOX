use nrg_graphics::*;
use nrg_math::*;

const DEFAULT_WIDTH: f32 = 400.0;
const DEFAULT_HEIGHT: f32 = 300.0;

pub struct Panel {
    pos: Vector2f,
    size: Vector2f,
    material_id: MaterialId,
    mesh_data: MeshData,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            pos: Vector2f::default(),
            size: Vector2f::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
            material_id: INVALID_ID,
            mesh_data: MeshData::default(),
        }
    }
}

impl Panel {
    pub fn init(&mut self, renderer: &mut Renderer) {
        self.pos = [0.0, 0.0].into();
        self.size = [0.5, 0.3].into();
        let pipeline_id = renderer.get_pipeline_id("UI");
        self.material_id = renderer.add_material(pipeline_id);
        self.mesh_data
            .add_quad_default([self.pos.x, self.pos.y, self.size.x, self.size.y].into())
            .set_vertex_color([1.0, 0.0, 0.0].into());

        renderer.add_mesh(self.material_id, &self.mesh_data);
    }
}
