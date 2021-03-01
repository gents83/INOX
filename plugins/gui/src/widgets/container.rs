use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct Container {}

impl Default for Container {
    fn default() -> Self {
        Self {}
    }
}

impl WidgetTrait for Container {
    fn init<T: WidgetTrait>(widget: &mut Widget<T>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();

        data.graphics.init(renderer, "UI");

        data.state.pos = Vector2f::default();
        data.state.size = [100.0, 100.0].into();
        data.state.is_draggable = true;

        let mut subpanel = Widget::<Panel>::new(Panel::default(), screen);
        subpanel
            .init(renderer)
            .size([800.0, 100.0].into())
            .border_color(1.0, 1.0, 1.0)
            .color(0.0, 0.0, 1.0)
            .stroke(10.0);
        let subpanel_id = widget.add_child(subpanel);
        widget.propagate_on_child(subpanel_id, |subpanel| {
            subpanel.set_margins(10.0, 10.0, 0.0, 0.0);
            subpanel.set_draggable(false);
        });

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
        data.graphics.set_mesh_data(renderer, &screen, mesh_data);
        data.graphics
            .translate([pos.x, pos.y, data.state.layer].into());

        data.graphics.update(renderer);
    }

    fn uninit<T: WidgetTrait>(_widget: &mut Widget<T>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
