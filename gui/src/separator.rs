use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Separator {
    data: WidgetData,
}
implement_widget!(Separator);

impl Default for Separator {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
        }
    }
}

impl InternalWidget for Separator {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size([DEFAULT_WIDGET_SIZE.x, 1].into())
            .stroke(1)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .selectable(false)
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::full_active())
            .set_border_style(WidgetStyle::full_active());
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
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
