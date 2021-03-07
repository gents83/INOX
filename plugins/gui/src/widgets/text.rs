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
        /*let spacing = screen.convert_from_pixels(widget.get_mut().spacing);
         */
        let color = widget.get_data_mut().graphics.get_color();

        widget.get_mut().text = "A\nCIAO".to_string();

        let font = renderer.get_font(widget.get_mut().font_id).unwrap();
        let mut mesh_data = MeshData::default();
        const VERTICES_COUNT: usize = 4;

        let mut prev_pos = Vector2f::default();
        let scale = 2. * DEFAULT_FONT_GLYPH_SIZE as f32 / DEFAULT_FONT_TEXTURE_SIZE as f32;
        let size =
            converted_scale.y * widget.get_mut().scale / DEFAULT_FONT_GLYPH_SIZE as f32 * scale;
        let spacing_x = scale * widget.get_mut().spacing.x / DEFAULT_FONT_GLYPH_SIZE as f32;
        let spacing_y = scale * widget.get_mut().spacing.y / DEFAULT_FONT_GLYPH_SIZE as f32;

        for (i, c) in widget.get_mut().text.as_bytes().iter().enumerate() {
            let id = font.get_glyph_index(*c as _);
            let g = font.get_glyph(id);
            mesh_data.add_quad(
                Vector4f::new(prev_pos.x, prev_pos.y, prev_pos.x + size, prev_pos.y + size),
                0.0,
                g.texture_coord,
                Some(i * VERTICES_COUNT),
            );

            if *c == b'\n' {
                prev_pos.x = 0.;
                prev_pos.y += size + spacing_y;
            } else {
                prev_pos.x += size + spacing_x;
            }
        }

        mesh_data.set_vertex_color(color);
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
        widget
            .get_data_mut()
            .graphics
            .set_mesh_data(renderer, clip_area, mesh_data);
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
