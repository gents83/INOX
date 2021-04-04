use nrg_graphics::{MeshData, Renderer};
use nrg_math::{Vector2u, Vector4u};
use nrg_platform::{EventsRw, InputHandler};
use nrg_serialize::{Deserialize, Serialize};

use crate::{
    implement_container, implement_widget, ContainerData, InternalWidget, WidgetData,
    DEFAULT_WIDGET_SIZE,
};

pub const DEFAULT_PANEL_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 10,
    y: DEFAULT_WIDGET_SIZE.y * 10,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Panel {
    #[serde(skip)]
    container: ContainerData,
    data: WidgetData,
}
implement_widget!(Panel);
implement_container!(Panel);

impl Default for Panel {
    fn default() -> Self {
        Self {
            container: ContainerData::default(),
            data: WidgetData::default(),
        }
    }
}

impl InternalWidget for Panel {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        self.size(DEFAULT_PANEL_SIZE)
            .style(WidgetStyle::DefaultBackground);
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
