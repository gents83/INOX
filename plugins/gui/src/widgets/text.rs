use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

#[allow(dead_code)]
pub enum HorizontalAlignment {
    Left,
    Right,
    Center,
    Stretch,
}
#[allow(dead_code)]
pub enum VerticalAlignment {
    Top,
    Bottom,
    Center,
    Stretch,
}

pub struct Text {
    font_id: FontId,
    text: String,
    scale: f32,
    spacing: Vector2f,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            font_id: INVALID_ID,
            text: String::new(),
            scale: 1.0,
            spacing: Vector2f::default(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }
}

impl Text {
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = String::from(text);
        self
    }

    pub fn set_horizontal_alignment(&mut self, alignment: HorizontalAlignment) -> &mut Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn set_vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = alignment;
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
        data.state.size = [500.0, 100.0].into();
        data.state.is_draggable = false;
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
        let mut pos = screen.convert_from_pixels_into_screen_space(widget.get_data_mut().state.pos);
        let mut size = screen
            .convert_from_pixels(widget.get_data_mut().state.size * screen.get_scale_factor());

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

        let lines_count = widget.get_mut().text.lines().count().max(1);
        let mut max_chars = 1;
        for text in widget.get_mut().text.lines() {
            max_chars = max_chars.max(text.len());
        }

        match widget.get().horizontal_alignment {
            HorizontalAlignment::Left => {
                pos.x = clip_area.x;
            }
            HorizontalAlignment::Right => {
                pos.x = clip_area.x + (clip_area.z - clip_area.x).abs() - size.x;
            }
            HorizontalAlignment::Center => {
                pos.x = clip_area.x + (clip_area.z - clip_area.x).abs() * 0.5 - size.x * 0.5;
            }
            HorizontalAlignment::Stretch => {
                pos.x = clip_area.x;
                size.x = (clip_area.z - clip_area.x).abs();
            }
        }

        match widget.get().vertical_alignment {
            VerticalAlignment::Top => {
                pos.y = clip_area.y;
            }
            VerticalAlignment::Bottom => {
                pos.y = clip_area.y + (clip_area.w - clip_area.y).abs() - size.y;
            }
            VerticalAlignment::Center => {
                pos.y = clip_area.y + (clip_area.w - clip_area.y).abs() * 0.5 - size.y * 0.5;
            }
            VerticalAlignment::Stretch => {
                pos.y = clip_area.y;
                size.y = (clip_area.w - clip_area.y).abs();
            }
        }

        widget.get_data_mut().state.pos = screen.convert_from_screen_space_into_pixels(pos);
        widget.get_data_mut().state.size = screen.convert_into_pixels(size) * 0.5;

        let char_height = size.y / lines_count as f32;
        let char_width = size.x / max_chars as f32;
        let char_color = widget.get_data().graphics.get_color();
        let char_layer = widget.get_data().state.layer;

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
        widget
            .get_data_mut()
            .graphics
            .set_mesh_data(renderer, clip_area, mesh_data);

        widget.update_layout();
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
