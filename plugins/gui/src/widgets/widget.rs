use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct Widget {
    pub pos: Vector2f,
    pub material_id: MaterialId,
    pub mesh_id: MeshId,
    pub mesh_data: MeshData,
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            pos: Vector2f::default(),
            material_id: INVALID_ID,
            mesh_id: INVALID_ID,
            mesh_data: MeshData::default(),
        }
    }
}

impl Widget {
    pub fn update(&mut self, renderer: &mut Renderer, input_handler: &InputHandler) {
        let is_inside = self.is_inside(
            input_handler.get_mouse_data().get_x() as _,
            input_handler.get_mouse_data().get_y() as _,
        );
        if is_inside && input_handler.get_mouse_data().is_dragging() {
            let pos = self.get_position()
                + Vector2f {
                    x: input_handler.get_mouse_data().movement_x() as _,
                    y: input_handler.get_mouse_data().movement_y() as _,
                };
            self.set_position(pos.x, pos.y);
        }

        renderer.update_mesh(self.material_id, self.mesh_id, &self.mesh_data);
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) -> &mut Self {
        renderer.remove_mesh(self.material_id, self.mesh_id);
        renderer.remove_material(self.material_id);
        self.material_id = INVALID_ID;
        self.mesh_id = INVALID_ID;
        self.mesh_data.clear();
        self
    }

    pub fn get_position(&self) -> Vector2f {
        let pos: Vector2f = (self.pos + [1.0, 1.0].into()) * 0.5;
        pos
    }

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

    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        let pos: Vector2f = Vector2f { x, y } * 2.0 - [1.0, 1.0].into();
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
}
