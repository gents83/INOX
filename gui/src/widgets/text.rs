use std::any::TypeId;

use nrg_graphics::{FontId, FontInstance, MaterialId, MaterialInstance, MeshData};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_undoable_message, Message};
use nrg_platform::{MouseEvent, MouseState};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, WidgetData, WidgetEvent,
    DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_TEXT_SIZE: [f32; 2] =
    [DEFAULT_WIDGET_WIDTH * 100., DEFAULT_WIDGET_HEIGHT / 4. * 3.];

#[derive(Clone, Copy)]
pub enum TextEvent {
    AddChar(Uid, i32, char),
    RemoveChar(Uid, i32, char),
}
implement_undoable_message!(TextEvent, undo_event, debug_info_event);
fn undo_event(event: &TextEvent) -> TextEvent {
    match event {
        TextEvent::AddChar(widget_id, character_index, character) => {
            TextEvent::RemoveChar(*widget_id, *character_index + 1, *character)
        }
        TextEvent::RemoveChar(widget_id, character_index, character) => {
            TextEvent::AddChar(*widget_id, *character_index - 1, *character)
        }
    }
}
fn debug_info_event(event: &TextEvent) -> String {
    match event {
        TextEvent::AddChar(_widget_id, _character_index, character) => {
            let mut str = String::from("AddChar[");
            str.push(*character);
            str.push(']');
            str
        }
        TextEvent::RemoveChar(_widget_id, _character_index, character) => {
            let mut str = String::from("RemoveChar[");
            str.push(*character);
            str.push(']');
            str
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Text {
    #[serde(skip)]
    font_id: FontId,
    #[serde(skip)]
    material_id: MaterialId,
    text: String,
    #[serde(skip)]
    hover_char_index: i32,
    char_width: u32,
    #[serde(skip)]
    is_dirty: bool,
    data: WidgetData,
}
implement_widget_with_custom_members!(Text {
    font_id: INVALID_UID,
    material_id: INVALID_UID,
    text: String::new(),
    hover_char_index: -1,
    char_width: DEFAULT_TEXT_SIZE[1] as _,
    is_dirty: true
});

impl Text {
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.is_dirty = true;
        self.text = String::from(text);
        self.compute_size();
        self
    }

    pub fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    pub fn is_hover_char(&self) -> bool {
        self.hover_char_index >= 0 && self.hover_char_index <= self.text.len() as _
    }
    pub fn get_hover_char(&self) -> i32 {
        self.hover_char_index
    }
    pub fn get_char_pos(&self, index: i32) -> Vector2 {
        let pos = self.state().get_position();
        if index >= 0 && index < self.text.len() as _ {
            return [pos.x + self.char_width as f32 * (index as f32 + 1.), pos.y].into();
        }
        pos
    }
    pub fn get_char_at(&self, index: i32) -> Option<char> {
        if index >= 0 && index < self.text.len() as _ {
            return Some(self.text.as_bytes()[index as usize] as _);
        }
        None
    }

    fn add_char(&mut self, index: i32, character: char) {
        let mut new_index = index + 1;
        if new_index > self.text.len() as i32 {
            new_index = self.text.len() as i32;
        }
        if new_index < 0 {
            new_index = 0;
        }
        self.is_dirty = true;
        self.text.insert(new_index as _, character);
        self.compute_size();
    }
    fn remove_char(&mut self, index: i32) -> char {
        if index >= 0 && index < self.text.len() as _ {
            self.is_dirty = true;
            let c = self.text.remove(index as usize);
            self.compute_size();
            return c;
        }
        char::default()
    }

    fn compute_size(&mut self) -> &mut Self {
        let drawing_area_in_px = self.state().get_drawing_area();
        let size: Vector2 = [drawing_area_in_px.z, drawing_area_in_px.w].into();

        let lines_count = self.text.lines().count().max(1);
        let mut max_chars = 1;
        for text in self.text.lines() {
            max_chars = max_chars.max(text.len());
        }

        let char_size = self.char_width as f32;
        let mut char_width = char_size;
        let mut char_height = char_size;
        if *self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch {
            char_width = size.x / max_chars as f32;
        }
        if *self.state().get_vertical_alignment() == VerticalAlignment::Stretch {
            char_height = size.y / lines_count as f32;
        }

        let new_size: Vector2 = [
            char_width * max_chars as f32,
            char_height * lines_count as f32,
        ]
        .into();

        self.set_size(new_size);
        self
    }

    fn update_mesh_from_text(&mut self) {
        let mut mesh_data = MeshData::default();
        let mut pos_y = 0.;
        let mut mesh_index = 0;
        let lines_count = self.text.lines().count().max(1);
        let size = self.state_mut().get_size();
        let char_width = self.char_width as f32 / size.x;
        let char_height = 1. / lines_count as f32;
        for text in self.text.lines() {
            let mut pos_x = 0.;
            for c in text.as_bytes().iter() {
                mesh_data.add_quad(
                    Vector4::new(pos_x, pos_y, pos_x + char_width, pos_y + char_height),
                    0.,
                    FontInstance::get_glyph_texture_coord(
                        &self.get_shared_data(),
                        self.font_id,
                        *c as _,
                    ),
                    Some(mesh_index),
                );
                mesh_index += 4;
                pos_x += char_width;
            }
            pos_y += char_height;
        }

        self.graphics_mut().set_mesh_data(mesh_data);
    }
}

impl InternalWidget for Text {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<TextEvent>()
            .register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

        let font_id = FontInstance::get_default(self.get_shared_data());
        let material_id = MaterialInstance::create_from_font(self.get_shared_data(), font_id);

        self.font_id = font_id;
        self.material_id = material_id;

        self.graphics_mut().link_to_material(material_id);
        if self.is_initialized() {
            self.is_dirty = true;
            return;
        }

        let size: Vector2 = DEFAULT_TEXT_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .style(WidgetStyle::DefaultText);

        self.char_width = (DEFAULT_TEXT_SIZE[1] * Screen::get_scale_factor()) as _;
    }

    fn widget_update(&mut self) {
        if self.is_dirty {
            self.update_mesh_from_text();
            self.is_dirty = false;
        }
    }

    fn widget_uninit(&mut self) {
        self.graphics_mut().remove_meshes();

        self.unregister_to_listen_event::<TextEvent>()
            .unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<TextEvent>() {
            let event = msg.as_any().downcast_ref::<TextEvent>().unwrap();
            match *event {
                TextEvent::AddChar(widget_id, char_index, character) => {
                    if self.id() == widget_id {
                        self.add_char(char_index, character);
                    }
                }
                TextEvent::RemoveChar(widget_id, char_index, _character) => {
                    if self.id() == widget_id {
                        self.remove_char(char_index);
                    }
                }
            }
        } else if msg.type_id() == TypeId::of::<MouseEvent>() {
            let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
            if event.state == MouseState::Move {
                let mouse_pos = Vector2::new(event.x as _, event.y as _);
                let pos = self.state().get_position();
                let size = self.state().get_size();
                let count = self.text.lines().count();
                let line_height = size.y / count as f32;
                for (line_index, t) in self.text.lines().enumerate() {
                    for (i, _c) in t.as_bytes().iter().enumerate() {
                        if mouse_pos.x >= pos.x + self.char_width as f32 * i as f32
                            && mouse_pos.x <= pos.x + self.char_width as f32 * (i as f32 + 1.)
                            && mouse_pos.y >= pos.y + line_height * line_index as f32
                            && mouse_pos.y <= pos.y + size.y + line_height * line_index as f32
                        {
                            self.hover_char_index = 1 + i as i32;
                        }
                    }
                }
            }
        }
    }
}
