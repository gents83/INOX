use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct GraphNode {
    container_data: ContainerData,
    title_widget: UID,
}

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            container_data: ContainerData::default(),
            title_widget: INVALID_ID,
        }
    }
}

impl ContainerTrait for GraphNode {
    fn get_container_data(&self) -> &ContainerData {
        &self.container_data
    }
    fn get_container_data_mut(&mut self) -> &mut ContainerData {
        &mut self.container_data
    }
}

impl WidgetTrait for GraphNode {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let default_size = DEFAULT_WIDGET_SIZE * screen.get_scale_factor();

        data.graphics
            .init(renderer, "UI")
            .set_style(WidgetStyle::default_background());

        let size: Vector2u = [200, 100].into();
        data.state
            .set_position(screen.get_center() - size / 2)
            .set_size(size);

        widget
            .draggable(true)
            .get_mut()
            .set_fill_type(ContainerFillType::Vertical)
            .set_fit_to_content(true);

        let mut title = Widget::<Text>::new(screen);
        title
            .init(renderer)
            .draggable(false)
            .size([0, default_size.y].into())
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Center);
        title.get_mut().set_text("Title");
        widget.get_mut().title_widget = widget.add_child(title);
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        Self::fit_to_content(widget);

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
