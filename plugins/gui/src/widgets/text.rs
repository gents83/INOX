use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct Text {
    font_id: FontId,
    text: String,
    scale: f32,
    spacing: Vector2f,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            font_id: INVALID_ID,
            text: String::new(),
            scale: 1.0,
            spacing: Vector2f::default(),
        }
    }
}

impl Text {
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = String::from(text);
        self
    }
}

impl WidgetTrait for Text {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        widget.get_mut().font_id = renderer.get_default_font_id();
        widget.get_mut().scale = 100.;
        widget.get_mut().spacing = [0., 0.].into();

        let data = widget.get_data_mut();
        data.graphics.init(renderer, "Font");

        data.state.pos = [0., 0.].into();
        data.graphics.set_style(WidgetStyle::default_text());

        widget.update_layout();
    }

    fn update(
        widget: &mut Widget<Self>,
        parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
        let screen = widget.get_screen();
        let pos = screen.convert_from_pixels_into_screen_space(widget.get_data_mut().state.pos);
        let converted_scale =
            screen.convert_from_pixels([widget.get_mut().scale, widget.get_mut().scale].into());

        widget.get_mut().text = "Click me!".to_string();

        let font = renderer.get_font(widget.get_mut().font_id).unwrap();
        let text_data = TextData {
            text: widget.get_mut().text.clone(),
            position: Vector2f::default(),
            color: widget.get_data_mut().graphics.get_color(),
            spacing: widget.get_mut().spacing,
            scale: 200. * screen.get_scale_factor(),
        };
        let mut mesh_data = font.create_mesh_from_text(&text_data);
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        /*
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
        widget.get_data_mut().graphics.set_mesh_data(
            renderer,
            [-1.0, -1.0, 1.0, 1.0].into(),
            mesh_data,
        );*/
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
