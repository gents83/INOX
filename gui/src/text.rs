use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

#[derive(Debug, Clone, Copy)]
pub struct TextChar {
    pub min: Vector2f,
    pub max: Vector2f,
}

pub struct Text {
    font_id: FontId,
    material_id: MaterialId,
    text: String,
    multiline: bool,
    characters: Vec<TextChar>,
    hover_char_index: i32,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            font_id: INVALID_ID,
            material_id: INVALID_ID,
            text: String::new(),
            multiline: false,
            characters: Vec::new(),
            hover_char_index: -1,
        }
    }
}

impl Text {
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = String::from(text);
        self
    }

    pub fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    pub fn set_multiline(&mut self, is_multiline: bool) -> &mut Self {
        self.multiline = is_multiline;
        self
    }
    pub fn is_multiline(&self) -> bool {
        self.multiline
    }
    pub fn is_hover_char(&self) -> bool {
        self.hover_char_index >= 0 && self.hover_char_index <= self.text.len() as _
    }
    pub fn get_hover_char(&self) -> i32 {
        self.hover_char_index
    }
    pub fn get_char_pos(&self, index: i32) -> Vector2f {
        if index >= 0 && index < self.text.len() as _ {
            return self.characters[index as usize].min;
        }
        Vector2f::default()
    }
    pub fn add_char(&mut self, index: i32, character: char) -> i32 {
        let mut new_index = index + 1;
        if (new_index < 0 && !self.text.is_empty()) || (new_index > self.text.len() as i32) {
            new_index = self.text.len() as i32;
        }
        if new_index < 0 {
            new_index = 0;
        }
        self.text.insert(new_index as _, character);
        new_index
    }
    pub fn remove_char(&mut self, index: i32) -> i32 {
        let mut new_index = index;
        if new_index < 0 && !self.text.is_empty() {
            new_index = self.text.len() as i32 - 1;
        }
        if new_index >= 0 && new_index < self.text.len() as _ {
            self.text.remove(new_index as usize);
        }
        new_index -= 1;
        if new_index < 0 && !self.text.is_empty() {
            new_index = 0;
        }
        new_index
    }
}

impl WidgetTrait for Text {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let font_id = renderer.get_default_font_id();
        let material_id = renderer.get_font_material_id(font_id);

        widget.get_mut().font_id = font_id;
        widget.get_mut().material_id = material_id;

        let data = widget.get_data_mut();
        data.graphics.link_to_material(material_id);
        data.graphics.set_style(WidgetStyle::default_text());

        data.state.set_draggable(false).set_selectable(false);
    }

    fn update(
        widget: &mut Widget<Self>,
        parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        _events: &mut EventsRw,
        input_handler: &InputHandler,
    ) {
        let screen = widget.get_screen();

        let mouse_pos = Vector2f {
            x: input_handler.get_mouse_data().get_x() as _,
            y: input_handler.get_mouse_data().get_y() as _,
        };
        let mouse_pos = screen.from_normalized_into_screen_space(mouse_pos);

        let pos = screen
            .convert_from_pixels_into_screen_space(widget.get_data_mut().state.get_position());
        let mut size = screen.convert_size_from_pixels(widget.get_data_mut().state.get_size());
        let min_size = size;
        if let Some(parent_state) = parent_data {
            size = screen.convert_size_from_pixels(parent_state.get_size());
        }

        let lines_count = widget.get_mut().text.lines().count().max(1);
        let mut max_chars = 1;
        for text in widget.get_mut().text.lines() {
            max_chars = max_chars.max(text.len());
        }

        let char_size = min_size.y / lines_count as f32;
        let min_char_width = min_size.x / max_chars as f32;
        let max_char_width = size.x / max_chars as f32;
        let char_size = char_size.min(min_char_width.max(max_char_width));
        let mut char_width = char_size;
        let mut char_height = char_size;
        if *widget.get_data().state.get_horizontal_alignment() == HorizontalAlignment::Stretch {
            char_width = size.x / max_chars as f32;
        }
        if *widget.get_data().state.get_vertical_alignment() == VerticalAlignment::Stretch {
            char_height = size.y / lines_count as f32;
        }

        let new_size: Vector2f = [
            min_size.x.max(char_width * max_chars as f32).min(size.x),
            min_size.y.max(char_height * lines_count as f32).min(size.y),
        ]
        .into();

        let char_color = widget.get_data().graphics.get_color();
        let char_layer = widget.get_data().state.get_layer();

        let font = renderer.get_font(widget.get_mut().font_id).unwrap();

        let mut mesh_data = MeshData::default();
        let mut pos_y = pos.y;
        let mut mesh_index = 0;
        let mut characters: Vec<TextChar> = Vec::new();
        let mut hover_char_index = -1;
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
                characters.push(TextChar {
                    min: [pos_x, pos_y].into(),
                    max: [pos_x + char_width, pos_y + char_height].into(),
                });
                if hover_char_index < 0 && mesh_data.is_inside(mouse_pos) {
                    hover_char_index = characters.len() as i32 - 1;
                }
            }
            pos_y += char_height;
        }
        widget.get_mut().hover_char_index = hover_char_index;
        widget.get_mut().characters = characters;
        widget
            .get_data_mut()
            .state
            .set_size(screen.convert_size_into_pixels(new_size));

        widget.get_data_mut().graphics.set_mesh_data(mesh_data);
    }

    fn uninit(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let data = widget.get_data_mut();
        data.graphics.remove_meshes(renderer).unlink_from_material();
    }

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
