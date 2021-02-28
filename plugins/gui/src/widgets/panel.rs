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
        let data = widget.get_data_mut();

        data.graphics.init(renderer, "UI");

        data.state.pos = Vector2f::default();
        data.state.size = [100.0, 100.0].into();
        data.state.is_draggable = true;

        widget.update_layout();
    }

    fn update<T: WidgetTrait>(
        widget: &mut Widget<T>,
        renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();

        let pos = screen.convert_from_pixels_into_screen_space(data.state.pos);
        let size =
            screen.convert_from_pixels_into_screen_space(screen.get_center() + data.state.size);
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, data.state.layer].into());
        data.graphics.set_mesh_data(renderer, mesh_data);

        data.graphics.update(renderer);
    }

    fn uninit<T: WidgetTrait>(_widget: &mut Widget<T>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
