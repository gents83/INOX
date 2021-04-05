use nrg_graphics::{FontId, MaterialId, MeshData, Renderer, INVALID_ID};
use nrg_math::{Vector2f, Vector2u, Vector4f, Vector4u};
use nrg_platform::{Event, EventsRw, InputHandler};
use nrg_serialize::{Deserialize, Serialize, UID};

use crate::{implement_widget, InternalWidget, WidgetData, DEFAULT_WIDGET_SIZE};

pub const DEFAULT_TEXT_SIZE: Vector2u = Vector2u {
    x: DEFAULT_WIDGET_SIZE.x * 8,
    y: DEFAULT_WIDGET_SIZE.y / 5 * 4,
};

pub enum TextEvent {
    AddChar(UID, i32, char),
    RemoveChar(UID, i32, char),
}
impl Event for TextEvent {}

#[derive(Debug, Clone, Copy)]
pub struct TextChar {
    pub min: Vector2f,
    pub max: Vector2f,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Text {
    #[serde(skip)]
    font_id: FontId,
    #[serde(skip)]
    material_id: MaterialId,
    text: String,
    multiline: bool,
    #[serde(skip)]
    characters: Vec<TextChar>,
    #[serde(skip)]
    hover_char_index: i32,
    data: WidgetData,
}
implement_widget!(Text);

impl Default for Text {
    fn default() -> Self {
        Self {
            font_id: INVALID_ID,
            material_id: INVALID_ID,
            text: String::new(),
            multiline: false,
            characters: Vec::new(),
            hover_char_index: -1,
            data: WidgetData::default(),
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
    pub fn get_char_at(&self, index: i32) -> Option<char> {
        if index >= 0 && index < self.text.len() as _ {
            return Some(self.text.as_bytes()[index as usize] as _);
        }
        None
    }

    fn update_text(&mut self, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(mut text_events) = events.read_events::<TextEvent>() {
            for event in text_events.iter_mut() {
                match event {
                    TextEvent::AddChar(widget_id, char_index, character) => {
                        if *widget_id == self.id() {
                            self.add_char(*char_index, *character);
                        }
                    }
                    TextEvent::RemoveChar(widget_id, char_index, _character) => {
                        if *widget_id == self.id() {
                            self.remove_char(*char_index);
                        }
                    }
                }
            }
        }
    }

    fn add_char(&mut self, index: i32, character: char) {
        let mut new_index = index + 1;
        if new_index > self.text.len() as i32 {
            new_index = self.text.len() as i32;
        }
        if new_index < 0 {
            new_index = 0;
        }
        self.text.insert(new_index as _, character);
    }
    fn remove_char(&mut self, index: i32) -> char {
        if index >= 0 && index < self.text.len() as _ {
            return self.text.remove(index as usize);
        }
        char::default()
    }
}

impl InternalWidget for Text {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        let font_id = renderer.get_default_font_id();
        let material_id = renderer.get_font_material_id(font_id);

        self.font_id = font_id;
        self.material_id = material_id;

        self.get_data_mut().graphics.link_to_material(material_id);
        if self.is_initialized() {
            return;
        }

        self.size(DEFAULT_TEXT_SIZE * Screen::get_scale_factor())
            .selectable(false)
            .style(WidgetStyle::DefaultText);
    }

    fn widget_update(
        &mut self,
        drawing_area_in_px: Vector4u,
        renderer: &mut Renderer,
        events_rw: &mut EventsRw,
        input_handler: &InputHandler,
    ) {
        self.update_text(events_rw);

        let mouse_pos = Vector2f {
            x: input_handler.get_mouse_data().get_x() as _,
            y: input_handler.get_mouse_data().get_y() as _,
        };
        let mouse_pos = Screen::from_normalized_into_screen_space(mouse_pos);

        let pos =
            Screen::convert_from_pixels_into_screen_space(self.get_data_mut().state.get_position());
        let mut size = Screen::convert_size_from_pixels(self.get_data_mut().state.get_size());
        let min_size = size;
        size =
            Screen::convert_size_from_pixels([drawing_area_in_px.z, drawing_area_in_px.w].into());

        let lines_count = self.text.lines().count().max(1);
        let mut max_chars = 1;
        for text in self.text.lines() {
            max_chars = max_chars.max(text.len());
        }

        let char_size = min_size.y / lines_count as f32;
        let mut char_width = char_size;
        let mut char_height = char_size;
        if *self.get_data().state.get_horizontal_alignment() == HorizontalAlignment::Stretch {
            char_width = size.x / max_chars as f32;
        }
        if *self.get_data().state.get_vertical_alignment() == VerticalAlignment::Stretch {
            char_height = size.y / lines_count as f32;
        }

        let new_size: Vector2f = [
            char_width * max_chars as f32,
            char_height * lines_count as f32,
        ]
        .into();

        let char_color = self.get_data().graphics.get_color();
        let char_layer = self.get_data().state.get_layer();

        let font = renderer.get_font(self.font_id).unwrap();

        let mut mesh_data = MeshData::default();
        let mut pos_y = pos.y;
        let mut mesh_index = 0;
        let mut characters: Vec<TextChar> = Vec::new();
        let mut hover_char_index = -1;
        for text in self.text.lines() {
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
        self.hover_char_index = hover_char_index;
        self.characters = characters;
        self.get_data_mut()
            .state
            .set_size(Screen::convert_size_into_pixels(new_size));

        self.get_data_mut().graphics.set_mesh_data(mesh_data);
    }

    fn widget_uninit(&mut self, renderer: &mut Renderer) {
        let data = self.get_data_mut();
        data.graphics.remove_meshes(renderer).unlink_from_material();
    }
}
