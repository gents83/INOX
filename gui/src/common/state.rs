use nrg_math::{VecBase, Vector2, Vector4};
use nrg_serialize::{Deserialize, Serialize};

use crate::{
    ContainerFillType, HorizontalAlignment, VerticalAlignment, WidgetInteractiveState, WidgetStyle,
    DEFAULT_WIDGET_SIZE,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetState {
    pos_in_px: Vector2,
    size_in_px: Vector2,
    is_active: bool,
    is_selectable: bool,
    is_draggable: bool,
    #[serde(skip)]
    is_pressed: bool,
    #[serde(skip)]
    is_hover: bool,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    dragging_pos_in_px: Vector2,
    style: WidgetStyle,
    border_style: WidgetStyle,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    fill_type: ContainerFillType,
    use_space_before_after: bool,
    keep_fixed_width: bool,
    keep_fixed_height: bool,
    space_between_elements: u32,
    #[serde(skip)]
    is_dirty: bool,
}

impl Default for WidgetState {
    #[inline]
    fn default() -> Self {
        Self {
            pos_in_px: Vector2::default_zero(),
            size_in_px: DEFAULT_WIDGET_SIZE.into(),
            is_active: true,
            is_selectable: true,
            is_draggable: false,
            is_pressed: false,
            is_hover: false,
            dragging_pos_in_px: Vector2::default_zero(),
            style: WidgetStyle::Default,
            border_style: WidgetStyle::DefaultBorder,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            fill_type: ContainerFillType::None,
            use_space_before_after: false,
            keep_fixed_width: true,
            keep_fixed_height: true,
            space_between_elements: 0,
            is_dirty: true,
        }
    }
}

impl WidgetState {
    #[inline]
    pub fn set_dirty(&mut self, value: bool) {
        self.is_dirty = value;
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    #[inline]
    pub fn set_style(&mut self, style: WidgetStyle) -> &mut Self {
        self.style = style;
        self
    }

    #[inline]
    pub fn set_border_style(&mut self, style: WidgetStyle) -> &mut Self {
        self.border_style = style;
        self
    }

    #[inline]
    pub fn get_color(&self, state: WidgetInteractiveState) -> Vector4 {
        WidgetStyle::color(&self.style, state)
    }

    #[inline]
    pub fn get_border_color(&self, state: WidgetInteractiveState) -> Vector4 {
        WidgetStyle::color(&self.border_style, state)
    }

    #[inline]
    pub fn fill_type(&mut self, fill_type: ContainerFillType) -> &mut Self {
        self.fill_type = fill_type;
        if fill_type == ContainerFillType::Horizontal {
            self.keep_fixed_width = false;
        } else if fill_type == ContainerFillType::Vertical {
            self.keep_fixed_height = false;
        }
        self
    }

    #[inline]
    pub fn get_fill_type(&self) -> ContainerFillType {
        self.fill_type
    }

    #[inline]
    pub fn keep_fixed_width(&mut self, keep_fixed_width: bool) -> &mut Self {
        self.keep_fixed_width = keep_fixed_width;
        self
    }

    #[inline]
    pub fn should_keep_fixed_width(&self) -> bool {
        self.keep_fixed_width
    }

    #[inline]
    pub fn keep_fixed_height(&mut self, keep_fixed_height: bool) -> &mut Self {
        self.keep_fixed_height = keep_fixed_height;
        self
    }

    #[inline]
    pub fn should_keep_fixed_height(&self) -> bool {
        self.keep_fixed_height
    }

    #[inline]
    pub fn space_between_elements(&mut self, space_in_px: u32) -> &mut Self {
        self.space_between_elements = space_in_px;
        self
    }

    #[inline]
    pub fn get_space_between_elements(&self) -> u32 {
        self.space_between_elements
    }

    #[inline]
    pub fn use_space_before_and_after(&mut self, use_space_before_after: bool) -> &mut Self {
        self.use_space_before_after = use_space_before_after;
        self
    }

    #[inline]
    pub fn should_use_space_before_and_after(&self) -> bool {
        self.use_space_before_after
    }

    #[inline]
    pub fn get_position(&self) -> Vector2 {
        self.pos_in_px
    }

    #[inline]
    pub fn set_position(&mut self, pos_in_px: Vector2) -> &mut Self {
        self.pos_in_px = pos_in_px;
        self
    }

    #[inline]
    pub fn get_size(&self) -> Vector2 {
        self.size_in_px
    }

    #[inline]
    pub fn set_size(&mut self, size_in_px: Vector2) -> &mut Self {
        self.size_in_px = size_in_px;
        self
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    #[inline]
    pub fn is_hover(&self) -> bool {
        self.is_hover
    }

    #[inline]
    pub fn is_draggable(&self) -> bool {
        self.is_draggable
    }

    #[inline]
    pub fn is_selectable(&self) -> bool {
        self.is_selectable
    }

    #[inline]
    pub fn set_selectable(&mut self, is_selectable: bool) -> &mut Self {
        self.is_selectable = is_selectable;
        self
    }

    #[inline]
    pub fn set_draggable(&mut self, is_draggable: bool) -> &mut Self {
        self.is_draggable = is_draggable;
        self
    }

    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    #[inline]
    pub fn set_pressed(&mut self, is_pressed: bool) -> &mut Self {
        self.is_pressed = is_pressed;
        self
    }

    #[inline]
    pub fn set_hover(&mut self, is_hover: bool) -> &mut Self {
        self.is_hover = is_hover;
        self
    }

    #[inline]
    pub fn get_dragging_position(&self) -> Vector2 {
        self.dragging_pos_in_px
    }

    #[inline]
    pub fn set_dragging_position(&mut self, pos_in_px: Vector2) -> &mut Self {
        self.dragging_pos_in_px = pos_in_px;
        self
    }

    #[inline]
    pub fn set_horizontal_alignment(&mut self, alignment: HorizontalAlignment) -> &mut Self {
        self.horizontal_alignment = alignment;
        self
    }

    #[inline]
    pub fn set_vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = alignment;
        self
    }

    #[inline]
    pub fn get_horizontal_alignment(&self) -> HorizontalAlignment {
        self.horizontal_alignment
    }

    #[inline]
    pub fn get_vertical_alignment(&self) -> VerticalAlignment {
        self.vertical_alignment
    }

    #[inline]
    pub fn is_inside(&self, pos_in_px: Vector2) -> bool {
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
