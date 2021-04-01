use std::time::{Duration, Instant};

use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Indicator {
    #[serde(skip)]
    is_active: bool,
    #[serde(skip)]
    is_blinking: bool,
    #[serde(skip)]
    refresh_time: Duration,
    #[serde(skip, default = "Instant::now")]
    elapsed_time: Instant,
    data: WidgetData,
}
implement_widget!(Indicator);

impl Default for Indicator {
    fn default() -> Self {
        Self {
            is_active: true,
            is_blinking: true,
            refresh_time: Duration::from_millis(500),
            elapsed_time: Instant::now(),
            data: WidgetData::default(),
        }
    }
}

impl Indicator {
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn set_active(&mut self, active: bool) -> &mut Self {
        self.is_active = active;
        self
    }
    fn update_blinkng(&mut self) {
        if self.elapsed_time.elapsed() >= self.refresh_time {
            let blinking = self.is_blinking;
            self.elapsed_time = Instant::now();

            if !blinking {
                self.get_data_mut()
                    .graphics
                    .set_style(WidgetStyle::FullActive)
                    .set_border_style(WidgetStyle::FullActive);
            } else {
                self.get_data_mut()
                    .graphics
                    .set_style(WidgetStyle::FullInactive)
                    .set_border_style(WidgetStyle::FullInactive);
            }
            self.is_blinking = !blinking;
        }
    }
}

impl InternalWidget for Indicator {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }
        self.draggable(false)
            .size([1, DEFAULT_WIDGET_SIZE.y - 2].into())
            .stroke(1)
            .vertical_alignment(VerticalAlignment::Stretch)
            .selectable(false)
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::FullActive)
            .set_border_style(WidgetStyle::FullActive);
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        if self.is_active {
            self.update_blinkng();

            let data = self.get_data_mut();
            let pos = Screen::convert_from_pixels_into_screen_space(data.state.get_position());
            let size = Screen::convert_size_from_pixels(data.state.get_size());
            let mut mesh_data = MeshData::default();
            mesh_data
                .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
                .set_vertex_color(data.graphics.get_color());
            mesh_data.translate([pos.x, pos.y, 0.0].into());
            data.graphics.set_mesh_data(mesh_data);
        } else {
            let data = self.get_data_mut();
            data.graphics.remove_meshes(renderer);
        }
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
