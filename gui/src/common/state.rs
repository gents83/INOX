use super::*;
use nrg_math::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetState {
    pos_in_px: Vector2u,
    size_in_px: Vector2u,
    #[serde(skip)]
    clip_area: Vector4u,
    is_active: bool,
    is_selectable: bool,
    is_draggable: bool,
    #[serde(skip)]
    is_pressed: bool,
    #[serde(skip)]
    is_hover: bool,
    layer: f32,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            pos_in_px: Vector2u::default(),
            size_in_px: DEFAULT_WIDGET_SIZE,
            clip_area: Vector4u::default(),
            is_active: true,
            is_selectable: true,
            is_draggable: false,
            is_pressed: false,
            is_hover: false,
            layer: 1.0 - DEFAULT_LAYER_OFFSET,
            horizontal_alignment: HorizontalAlignment::None,
            vertical_alignment: VerticalAlignment::None,
        }
    }
}

impl WidgetState {
    pub fn get_position(&self) -> Vector2u {
        self.pos_in_px
    }

    pub fn set_position(&mut self, pos_in_px: Vector2u) -> &mut Self {
        self.pos_in_px = pos_in_px;
        self
    }
    pub fn get_size(&self) -> Vector2u {
        self.size_in_px
    }
    pub fn set_size(&mut self, size_in_px: Vector2u) -> &mut Self {
        self.size_in_px = size_in_px;
        self
    }
    pub fn get_clip_area(&self) -> Vector4u {
        self.clip_area
    }

    pub fn set_clip_area(&mut self, clip_area: Vector4u) -> &mut Self {
        self.clip_area = clip_area;
        self
    }
    pub fn get_layer(&self) -> f32 {
        self.layer
    }
    pub fn set_layer(&mut self, layer: f32) -> &mut Self {
        self.layer = layer;
        self
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn is_hover(&self) -> bool {
        self.is_hover
    }
    pub fn is_draggable(&self) -> bool {
        self.is_draggable
    }
    pub fn is_selectable(&self) -> bool {
        self.is_selectable
    }

    pub fn set_selectable(&mut self, is_selectable: bool) -> &mut Self {
        self.is_selectable = is_selectable;
        self
    }

    pub fn set_draggable(&mut self, is_draggable: bool) -> &mut Self {
        self.is_draggable = is_draggable;
        self
    }
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub fn set_pressed(&mut self, is_pressed: bool) -> &mut Self {
        self.is_pressed = is_pressed;
        self
    }

    pub fn set_hover(&mut self, is_hover: bool) -> &mut Self {
        self.is_hover = is_hover;
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

    pub fn get_horizontal_alignment(&self) -> &HorizontalAlignment {
        &self.horizontal_alignment
    }

    pub fn get_vertical_alignment(&self) -> &VerticalAlignment {
        &self.vertical_alignment
    }

    pub fn is_inside(&self, pos_in_px: Vector2u) -> bool {
        if pos_in_px.x >= self.pos_in_px.x
            && pos_in_px.x <= self.pos_in_px.x + self.size_in_px.x
            && pos_in_px.y >= self.pos_in_px.y
            && pos_in_px.y <= self.pos_in_px.y + self.size_in_px.y
        {
            return true;
        }
        false
    }
}
