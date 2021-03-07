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

        data.state
            .set_position(Vector2f::default())
            .set_size([100.0, 100.0].into())
            .set_draggable(true);

        let mut subpanel = Widget::<Panel>::new(Panel::default(), screen.clone());
        subpanel
            .init(renderer)
            .size([550., 250.].into())
            .stroke(10.);

        let mut text = Widget::<Text>::new(Text::default(), screen);
        text.init(renderer)
            .margins(10.0, 0.0, 0.0, 0.0)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center);
        text.get_mut().set_text("Title");

        subpanel.add_child(text);

        let subpanel_id = widget.add_child(subpanel);
        widget.propagate_on_child(subpanel_id, |subpanel| {
            subpanel.set_margins(10.0, 10.0, 0.0, 0.0);
            subpanel.set_draggable(true);
        });
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();

        let pos = screen.convert_from_pixels_into_screen_space(data.state.get_position());
        let size = screen.convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
