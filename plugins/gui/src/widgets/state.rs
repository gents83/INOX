use super::align::*;
use nrg_math::*;

pub const LAYER_OFFSET: f32 = 0.001;

pub struct WidgetState {
    pos: Vector2f,
    size: Vector2f,
    is_active: bool,
    is_draggable: bool,
    is_dragging: bool,
    is_hover: bool,
    margins: WidgetMargins,
    layer: f32,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            pos: Vector2f::default(),
            size: [1., 1.].into(),
            is_active: true,
            is_draggable: false,
            is_dragging: false,
            is_hover: false,
            margins: WidgetMargins::default(),
            layer: 1.0 - LAYER_OFFSET,
            horizontal_alignment: HorizontalAlignment::None,
            vertical_alignment: VerticalAlignment::None,
        }
    }
}

impl WidgetState {
    pub fn get_position(&self) -> Vector2f {
        self.pos
    }

    pub fn set_position(&mut self, pos: Vector2f) -> &mut Self {
        self.pos = pos;
        self
    }
    pub fn get_size(&self) -> Vector2f {
        self.size
    }
    pub fn set_size(&mut self, size: Vector2f) -> &mut Self {
        self.size = size;
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

    pub fn set_draggable(&mut self, is_draggable: bool) -> &mut Self {
        self.is_draggable = is_draggable;
        self
    }
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    pub fn set_dragging(&mut self, is_dragging: bool) -> &mut Self {
        self.is_dragging = is_dragging;
        self.vertical_alignment = VerticalAlignment::None;
        self.horizontal_alignment = HorizontalAlignment::None;
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

    pub fn set_margins(&mut self, margins: WidgetMargins) -> &mut Self {
        self.margins = margins;
        self
    }

    pub fn get_margins(&self) -> &WidgetMargins {
        &self.margins
    }

    pub fn is_inside(&self, pos: Vector2f) -> bool {
        if pos.x >= self.pos.x
            && pos.x <= self.pos.x + self.size.x
            && pos.y >= self.pos.y
            && pos.y <= self.pos.y + self.size.y
        {
            return true;
        }
        false
    }
}
