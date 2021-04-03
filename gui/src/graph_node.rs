use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    #[serde(skip)]
    title_widget: UID,
    #[serde(skip)]
    container: ContainerData,
    data: WidgetData,
}
implement_widget!(GraphNode);
implement_container!(GraphNode);

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            container: ContainerData::default(),
            data: WidgetData::default(),
            title_widget: INVALID_ID,
        }
    }
}

impl InternalWidget for GraphNode {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        let size: Vector2u = [200, 100].into();

        self.position(Screen::get_center() - size / 2)
            .size(size)
            .draggable(true)
            .style(WidgetStyle::DefaultBackground);

        let mut title = Text::default();
        title.init(renderer);
        title
            .draggable(false)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Center);
        title.set_text("Title");
        self.title_widget = self.add_child(Box::new(title));
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        self.apply_fit_to_content();

        let data = self.get_data_mut();
        let pos = Screen::convert_from_pixels_into_screen_space(data.state.get_position());
        let size = Screen::convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
