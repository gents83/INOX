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
        data.graphics.set_style(WidgetStyle::default_text());
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
        let screen = widget.get_screen();
        let pos = screen
            .convert_from_pixels_into_screen_space(widget.get_data_mut().state.get_position());
        let size = screen.convert_size_from_pixels(widget.get_data_mut().state.get_size());

        let lines_count = widget.get_mut().text.lines().count().max(1);
        let mut max_chars = 1;
        for text in widget.get_mut().text.lines() {
            max_chars = max_chars.max(text.len());
        }

        let mut char_height = size.y / lines_count as f32;
        let mut char_width = size.x / max_chars as f32;
        let char_color = widget.get_data().graphics.get_color();
        let char_layer = widget.get_data().state.get_layer();
        let char_size = char_height.min(char_width);
        if *widget.get_data().state.get_horizontal_alignment() != HorizontalAlignment::Stretch {
            char_width = char_size;
        }
        if *widget.get_data().state.get_vertical_alignment() != VerticalAlignment::Stretch {
            char_height = char_size;
        }

        let font = renderer.get_font(widget.get_mut().font_id).unwrap();

        let mut mesh_data = MeshData::default();
        let mut pos_y = pos.y;
        let mut mesh_index = 0;
        for text in widget.get_mut().text.lines() {
            let mut pos_x = pos.x;
            for c in text.as_bytes().iter() {
                let id = font.get_glyph_index(*c as _);
                let g = font.get_glyph(id);
                mesh_data
                    .add_quad(
                        Vector4f {
                            x: pos_x,
                            y: pos_y,
                            z: pos_x + char_width,
                            w: pos_y + char_height,
                        },
                        char_layer,
                        g.texture_coord,
                        Some(mesh_index),
                    )
                    .set_vertex_color(char_color);
                mesh_index += 4;
                pos_x += char_width;
            }
            pos_y += char_height;
        }
        let mut new_size: Vector2f =
            [char_size * max_chars as f32, char_size * lines_count as f32].into();
        if *widget.get_data().state.get_horizontal_alignment() == HorizontalAlignment::Stretch {
            new_size.x = size.x;
        }
        if *widget.get_data().state.get_vertical_alignment() == VerticalAlignment::Stretch {
            new_size.y = size.y;
        }
        widget
            .get_data_mut()
            .state
            .set_size(screen.convert_size_into_pixels(new_size));
        widget.get_data_mut().graphics.set_mesh_data(mesh_data);
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
