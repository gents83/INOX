use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct Panel {}

impl Default for Panel {
    fn default() -> Self {
        Self {}
    }
}

impl WidgetTrait for Panel {
    fn init<T: WidgetTrait>(widget: &mut Widget<T>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let pipeline_id = renderer.get_pipeline_id("UI");
        data.graphics.material_id = renderer.add_material(pipeline_id);
        data.state.pos = Vector2f::default();
        data.state.size = [100.0, 100.0].into();
        data.state.is_draggable = true;
        let pos = screen.convert_from_pixels_into_screen_space(data.state.pos);
        let size = screen.convert_from_pixels_into_screen_space(data.state.size);
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([pos.x, pos.y, size.x, size.y].into())
            .set_vertex_color([0.3, 0.8, 1.0].into());
        data.graphics.mesh_data = mesh_data;

        data.graphics.mesh_id =
            renderer.add_mesh(data.graphics.material_id, &data.graphics.mesh_data);
        widget.update_layout();
    }

    fn update<T: WidgetTrait>(
        _widget: &mut Widget<T>,
        _renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
    }

    fn uninit<T: WidgetTrait>(_widget: &mut Widget<T>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
