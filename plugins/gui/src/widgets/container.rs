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
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();

        data.graphics.init(renderer, "UI");

        data.state.pos = Vector2f::default();
        data.state.size = [100.0, 100.0].into();
        data.state.is_draggable = true;

        let mut subpanel = Widget::<Panel>::new(Panel::default(), screen.clone());
        subpanel
            .init(renderer)
            .size([550., 250.].into())
            .border_color(1., 1., 1., 1.)
            .color(0., 0., 1., 1.)
            .stroke(10.);

        let mut text = Widget::<Text>::new(Text::default(), screen);
        text.init(renderer)
            .color(1., 0., 1., 1.)
            .set_draggable(false);
        text.get_mut().set_text("Prova");

        subpanel.add_child(text);

        let subpanel_id = widget.add_child(subpanel);
        widget.propagate_on_child(subpanel_id, |subpanel| {
            subpanel.set_margins(10.0, 10.0, 0.0, 0.0);
            subpanel.set_draggable(false);
        });

        widget.update_layout();
    }

    fn update(
        widget: &mut Widget<Self>,
        parent_data: Option<&WidgetState>,
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
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.layer)
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        let clip_area: Vector4f = if let Some(parent_state) = parent_data {
            let parent_pos = screen.convert_from_pixels_into_screen_space(parent_state.pos);
            let parent_size = screen
                .convert_from_pixels_into_screen_space(screen.get_center() + parent_state.size);
            [
                parent_pos.x,
                parent_pos.y,
                parent_pos.x + parent_size.x,
                parent_pos.y + parent_size.y,
            ]
            .into()
        } else {
            [-1.0, -1.0, 1.0, 1.0].into()
        };
        data.graphics.set_mesh_data(renderer, clip_area, mesh_data);
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
