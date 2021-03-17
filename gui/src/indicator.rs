use std::time::{Duration, Instant};

use super::*;
use nrg_graphics::*;
use nrg_platform::*;

pub struct Indicator {
    is_active: bool,
    is_blinking: bool,
    refresh_time: Duration,
    elapsed_time: Instant,
}

impl Default for Indicator {
    fn default() -> Self {
        Self {
            is_active: true,
            is_blinking: true,
            refresh_time: Duration::from_millis(500),
            elapsed_time: Instant::now(),
        }
    }
}

impl Indicator {
    fn update_blinkng(widget: &mut Widget<Self>) {
        if widget.get().elapsed_time.elapsed() >= widget.get().refresh_time {
            let blinking = widget.get().is_blinking;
            widget.get_mut().elapsed_time = Instant::now();

            if !blinking {
                widget
                    .get_data_mut()
                    .graphics
                    .set_style(WidgetStyle::full_active())
                    .set_border_style(WidgetStyle::full_active());
            } else {
                widget
                    .get_data_mut()
                    .graphics
                    .set_style(WidgetStyle::full_inactive())
                    .set_border_style(WidgetStyle::full_inactive());
            }
            widget.get_mut().is_blinking = !blinking;
        }
    }
}

impl WidgetTrait for Indicator {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let data = widget.get_data_mut();
        data.graphics.init(renderer, "UI");

        widget
            .draggable(false)
            .size([1., DEFAULT_WIDGET_SIZE.y - 2.].into())
            .stroke(1.)
            .vertical_alignment(VerticalAlignment::Stretch)
            .selectable(false)
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::full_active())
            .set_border_style(WidgetStyle::full_active());
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        Self::update_blinkng(widget);

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
