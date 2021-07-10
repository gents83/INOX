use std::any::TypeId;

use nrg_graphics::{FontInstance, FontRc, MaterialInstance, MaterialRc, MeshData};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_undoable_message, Message};
use nrg_platform::{MouseEvent, MouseState};
use nrg_resources::{Resource, ResourceBase, SharedData};
use nrg_serialize::{Deserialize, Serialize, Uid};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Screen, WidgetData,
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
    #[serde(skip, default = "nrg_resources::Resource::default::<FontInstance>")]
    font: FontRc,
    #[serde(skip, default = "nrg_resources::Resource::default::<MaterialInstance>")]
    material: MaterialRc,
    editable: bool,
    text: String,
    #[serde(skip)]
    hover_char_index: i32,
    char_width: f32,
    space_between_chars: f32,
    #[serde(skip)]
    is_dirty: bool,
    data: WidgetData,
}
implement_widget_with_custom_members!(Text {
    font: Resource::default::<FontInstance>(),
    material: Resource::default::<MaterialInstance>(),
    editable: true,
    text: String::new(),
    hover_char_index: -1,
    char_width: DEFAULT_TEXT_SIZE[1],
    space_between_chars: 0.7,
    is_dirty: true
});

impl Text {
    pub fn editable(&mut self, is_editable: bool) -> &mut Self {
        if self.editable != is_editable {
            if is_editable {
                self.register_to_listen_event::<TextEvent>()
                    .register_to_listen_event::<MouseEvent>();
            } else {
                self.unregister_to_listen_event::<TextEvent>()
                    .unregister_to_listen_event::<MouseEvent>();
            }
            self.editable = is_editable;
        }
        self
    }
    pub fn set_text(&mut self, text: &str) -> &mut Self {
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
            return [
                pos.x + self.char_width * self.space_between_chars * (index as f32 + 1.),
                pos.y,
            ]
            .into();
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
        self.text.insert(new_index as _, character);
        self.compute_size();
    }
    fn remove_char(&mut self, index: i32) -> char {
        if index >= 0 && index < self.text.len() as _ {
            let c = self.text.remove(index as usize);
            self.compute_size();
            return c;
        }
        char::default()
    }

    fn compute_size(&mut self) -> &mut Self {
        let size = self.state().get_size();

        let lines_count = self.text.lines().count().max(1);
        let mut max_chars = 1;
        for text in self.text.lines() {
            max_chars = max_chars.max(text.len());
        }

        let char_size = size.y;

        let mut new_size: Vector2 = [char_size, char_size * lines_count as f32].into();
        if max_chars > 0 {
            new_size.x += char_size * self.space_between_chars * (max_chars - 1) as f32;
        }
        if self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch {
            new_size.x = size.x;
        }
        self.char_width = (new_size.x / (1. + ((max_chars - 1) as f32 * self.space_between_chars)))
            .min(char_size);

        self.set_size(new_size);
        self.is_dirty = true;
        self
    }

    fn update_mesh_from_text(&mut self) {
        let mut mesh_data = MeshData::default();
        let mut pos_y = 0.;
        let mut mesh_index = 0;
        let lines_count = self.text.lines().count().max(1);
        let size = self.state_mut().get_size();
        let char_width = self.char_width / size.x;
        let char_height = 1. / lines_count as f32;
        for text in self.text.lines() {
            let mut pos_x = 0.;
            for c in text.as_bytes().iter() {
                mesh_data.add_quad(
                    Vector4::new(pos_x, pos_y, pos_x + char_width, pos_y + char_height),
                    0.,
                    self.font
                        .get::<FontInstance>()
                        .get_glyph_texture_coord(*c as _),
                    Some(mesh_index),
                );
                mesh_index += 4;
                pos_x += char_width * self.space_between_chars;
            }
            pos_y += char_height;
        }

        self.graphics_mut().set_mesh_data(mesh_data);
    }
}

impl InternalWidget for Text {
    fn widget_init(&mut self) {
        let font_id = FontInstance::get_default(self.get_shared_data());
        self.font = SharedData::get_resource::<FontInstance>(self.get_shared_data(), font_id);
        self.material = self.font.get::<FontInstance>().material();
        let material = self.material.clone();
        self.graphics_mut().link_to_material(material);
        if self.is_initialized() {
            self.is_dirty = true;
            return;
        }

        let size: Vector2 = DEFAULT_TEXT_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .style(WidgetStyle::DefaultText)
            .editable(false);

        self.char_width = DEFAULT_TEXT_SIZE[1] * Screen::get_scale_factor();
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {
        if self.is_dirty {
            self.update_mesh_from_text();
            self.is_dirty = false;
        }
    }

    fn widget_uninit(&mut self) {
        self.graphics_mut().remove_meshes();

        self.editable(false);
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
                        if mouse_pos.x
                            >= pos.x + (self.char_width * self.space_between_chars) * i as f32
                            && mouse_pos.x
                                <= pos.x
                                    + (self.char_width * self.space_between_chars) * (i as f32 + 1.)
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
    fn widget_on_layout_changed(&mut self) {}
}
