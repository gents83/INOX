use std::{
    any::{Any, TypeId},
    sync::{Arc, RwLock},
};

use nrg_math::{Vector2, Vector4};
use nrg_messenger::{read_messages, Message};
use nrg_platform::{MouseEvent, MouseState};
use nrg_serialize::{typetag, Uid};

use crate::{
    add_space_before_after, add_widget_size, compute_child_clip_area, hex_to_rgba, Color,
    ContainerFillType, Gui, HorizontalAlignment, VerticalAlignment, WidgetDataGetter, WidgetEvent,
    WidgetGraphics, COLOR_ENGRAY,
};

pub type RefcountedWidget = Arc<RwLock<Box<dyn Widget>>>;

pub const DEFAULT_LAYER_OFFSET: f32 = 0.1;
pub const DEFAULT_WIDGET_WIDTH: f32 = 12.;
pub const DEFAULT_WIDGET_HEIGHT: f32 = 12.;
pub const DEFAULT_WIDGET_SIZE: [f32; 2] = [DEFAULT_WIDGET_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[typetag::serde(tag = "widget")]
pub trait Widget: BaseWidget + InternalWidget + Send + Sync {}

pub trait WidgetCreator: Widget {
    fn create_widget(
        shared_data: &nrg_resources::SharedDataRw,
        global_messenger: &nrg_messenger::MessengerRw,
    ) -> Box<dyn Widget>;
    fn new(
        shared_data: &nrg_resources::SharedDataRw,
        global_messenger: &nrg_messenger::MessengerRw,
    ) -> Self;
    fn load(
        shared_data: &nrg_resources::SharedDataRw,
        global_messenger: &nrg_messenger::MessengerRw,
        filepath: std::path::PathBuf,
    ) -> Self;
}

pub trait InternalWidget: Any {
    fn widget_init(&mut self);
    fn widget_update(&mut self, drawing_area_in_px: Vector4);
    fn widget_uninit(&mut self);
    fn widget_process_message(&mut self, _msg: &dyn Message);
    fn widget_on_layout_changed(&mut self);
}

pub trait BaseWidget: InternalWidget + WidgetDataGetter {
    #[inline]
    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    #[inline]
    fn get_type_id(&self) -> TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn init(&mut self) {
        self.graphics_mut().init("Default");

        if self.is_initialized() {
            let shared_data = self.get_shared_data().clone();
            let global_messenger = self.get_global_messenger().clone();
            self.node_mut().propagate_on_children_mut(|w| {
                w.load_override(shared_data.clone(), global_messenger.clone());
                w.init();
            });
        }

        self.widget_init();

        self.move_to_layer(self.graphics().get_layer());
        self.mark_as_dirty();
        self.mark_as_initialized();
    }
    fn update(&mut self, parent_data: Vector4, drawing_area_in_px: Vector4) {
        self.process_messages(drawing_area_in_px);

        if self.is_dirty() {
            self.mark_as_dirty();
            self.update_layout(parent_data);
            self.manage_style();
        }

        self.update_childrens(drawing_area_in_px);

        {
            nrg_profiler::scoped_profile!("widget::widget_update");
            self.widget_update(drawing_area_in_px);
        }

        self.graphics_mut().update(drawing_area_in_px);
    }
    #[inline]
    fn uninit(&mut self) {
        self.node_mut().propagate_on_children_mut(|w| w.uninit());
        self.widget_uninit();
        self.graphics_mut().uninit();
    }
    fn update_childrens(&mut self, drawing_area_in_px: Vector4) {
        nrg_profiler::scoped_profile!("widget::update_childrens");

        let filltype = self.state().get_fill_type();
        let space = self.state().get_space_between_elements() as f32;
        let use_space_before_after = self.state().should_use_space_before_and_after();
        let child_drawing_area = self.compute_children_drawing_area(drawing_area_in_px);
        let mut widget_space = self.compute_area_data();
        if use_space_before_after {
            widget_space = add_space_before_after(widget_space, filltype, space);
        }
        let children = self.node_mut().get_children_mut();
        for i in 0..children.len() {
            widget_space = compute_child_clip_area(
                widget_space,
                filltype,
                i,
                children,
                space,
                use_space_before_after,
            );
            let child = children[i].clone();
            Gui::add_additional_job("Update children", move || {
                child
                    .write()
                    .unwrap()
                    .update(widget_space, child_drawing_area);
            });
            widget_space = add_widget_size(
                widget_space,
                filltype,
                i,
                children,
                space,
                use_space_before_after,
            );
            widget_space = add_space_before_after(widget_space, filltype, space);
        }
    }
    #[inline]
    fn process_messages(&mut self, drawing_area_in_px: Vector4) {
        nrg_profiler::scoped_profile!("widget::process_messages");
        read_messages(self.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<MouseEvent>() {
                let e = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                self.manage_input(e, drawing_area_in_px);
            }
            if msg.type_id() == TypeId::of::<WidgetEvent>() {
                let e = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
                self.manage_events(e, drawing_area_in_px);
            }
            self.widget_process_message(msg);
        });
    }

    #[inline]
    fn mark_as_dirty(&mut self) {
        self.state_mut().set_dirty(true);
        self.node_mut()
            .propagate_on_children_mut(|w| w.mark_as_dirty());
    }
    #[inline]
    fn is_dirty(&self) -> bool {
        let mut is_dirty = self.state().is_dirty();
        if !is_dirty {
            self.node()
                .propagate_on_children(|w| is_dirty |= w.is_dirty());
        }
        is_dirty
    }

    #[inline]
    fn id(&self) -> Uid {
        self.node().get_id()
    }

    #[inline]
    fn set_selected(&mut self, is_selected: bool) {
        self.state_mut().set_selected(is_selected);
        self.mark_as_dirty();
    }

    #[inline]
    fn set_position(&mut self, pos_in_px: Vector2) {
        if pos_in_px != self.state().get_position() {
            self.state_mut().set_position(pos_in_px);
            self.graphics_mut().set_position(pos_in_px);
            self.mark_as_dirty();
        }
    }
    #[inline]
    fn set_size(&mut self, size_in_px: Vector2) {
        if size_in_px != self.state().get_size() {
            self.state_mut().set_size(size_in_px);
            self.graphics_mut().set_size(size_in_px);
            self.mark_as_dirty();
        }
    }

    #[inline]
    fn compute_area_data(&self) -> Vector4 {
        let pos = self.state().get_position();
        let size = self.state().get_size();
        [pos.x, pos.y, size.x, size.y].into()
    }

    #[inline]
    fn compute_children_drawing_area(&self, parent_drawing_area: Vector4) -> Vector4 {
        let pos = self.state().get_position();
        let size = self.state().get_size();
        let x = pos.x.max(parent_drawing_area.x);
        let y = pos.y.max(parent_drawing_area.y);
        [
            x,
            y,
            (size.x).min((parent_drawing_area.x + parent_drawing_area.z).min(pos.x + size.x) - x),
            (size.y).min((parent_drawing_area.y + parent_drawing_area.w).min(pos.y + size.y) - y),
        ]
        .into()
    }

    fn compute_offset_and_scale_from_alignment(
        &mut self,
        clip_rect: Vector4,
        actual_position: Vector2,
        actual_size: Vector2,
    ) -> (Vector2, Vector2) {
        nrg_profiler::scoped_profile!("widget::compute_offset_and_scale_from_alignment");

        let clip_pos: Vector2 = [clip_rect.x, clip_rect.y].into();
        let clip_size: Vector2 = [clip_rect.z, clip_rect.w].into();

        let mut pos = actual_position;
        let mut size = actual_size;

        match self.state().get_horizontal_alignment() {
            HorizontalAlignment::Left => {
                pos.x = clip_pos.x;
            }
            HorizontalAlignment::Right => {
                pos.x = clip_pos.x + clip_size.x - size.x;
            }
            HorizontalAlignment::Center => {
                pos.x = clip_pos.x + clip_size.x / 2. - size.x / 2.;
            }
            HorizontalAlignment::Stretch => {
                pos.x = clip_pos.x;
                size.x = clip_size.x;
            }
            _ => {}
        }

        match self.state().get_vertical_alignment() {
            VerticalAlignment::Top => {
                pos.y = clip_pos.y;
            }
            VerticalAlignment::Bottom => {
                pos.y = clip_pos.y + clip_size.y - size.y;
            }
            VerticalAlignment::Center => {
                pos.y = clip_pos.y + clip_size.y / 2. - size.y / 2.;
            }
            VerticalAlignment::Stretch => {
                pos.y = clip_pos.y;
                size.y = clip_size.y;
            }
            _ => {}
        }

        (pos, size)
    }

    fn compute_children_size(&mut self, actual_size: Vector2) -> Vector2 {
        nrg_profiler::scoped_profile!("widget::compute_children_size");

        let fill_type = self.state().get_fill_type();
        let space = self.state().get_space_between_elements() as f32;
        let use_space_before_after = self.state().should_use_space_before_and_after();

        let parent_size = actual_size;
        let node = &mut self.node_mut();

        let mut children_size: Vector2 = [0., 0.].into();
        let mut index = 0;
        node.propagate_on_children_mut(|w| {
            let child_state = &mut w.state_mut();
            let child_size = child_state.get_size();
            match fill_type {
                ContainerFillType::Vertical => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.y += space;
                    }
                    children_size.y += child_size.y;
                    children_size.x = children_size.x.max(child_size.x);
                }
                ContainerFillType::Horizontal => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.x += space;
                    }
                    children_size.x += child_size.x;
                    children_size.y = children_size.y.max(child_size.y);
                }
                _ => {
                    children_size.x = parent_size.x;
                    children_size.y = parent_size.y;
                }
            }
            index += 1;
        });
        if use_space_before_after && fill_type == ContainerFillType::Vertical {
            children_size.y += space;
        }
        if use_space_before_after && fill_type == ContainerFillType::Horizontal {
            children_size.x += space;
        }
        children_size
    }

    #[inline]
    fn manage_style(&mut self) {
        nrg_profiler::scoped_profile!("widget::manage_style");

        let mut color = self.state().get_color();
        let mut border_color = self.state().get_border_color();
        if self.state().is_selected() || self.state().is_pressed() {
            color = color.remove_color(hex_to_rgba(COLOR_ENGRAY));
            border_color = border_color.remove_color(hex_to_rgba(COLOR_ENGRAY));
        } else if self.state().is_hover() {
            color = color.add_color(hex_to_rgba(COLOR_ENGRAY));
            border_color = border_color.add_color(hex_to_rgba(COLOR_ENGRAY));
        } else if !self.state().is_active() {
            color = color
                .remove_color(hex_to_rgba(COLOR_ENGRAY))
                .remove_color(hex_to_rgba(COLOR_ENGRAY));
            border_color = border_color
                .remove_color(hex_to_rgba(COLOR_ENGRAY))
                .remove_color(hex_to_rgba(COLOR_ENGRAY));
        }
        self.graphics_mut()
            .set_color(color)
            .set_border_color(border_color);
    }

    #[inline]
    fn manage_events(&mut self, event: &WidgetEvent, drawing_area_in_px: Vector4) {
        nrg_profiler::scoped_profile!("widget::manage_events");
        if !self.graphics().is_visible()
            || !WidgetGraphics::is_valid_drawing_area(drawing_area_in_px)
        {
            return;
        }
        let id = self.id();
        match *event {
            WidgetEvent::InvalidateLayout(widget_id) => {
                if widget_id == id {
                    self.mark_as_dirty();
                }
            }
            WidgetEvent::Entering(widget_id) => {
                if widget_id == id && self.state().is_active() && !self.state().is_hover() {
                    self.state_mut().set_hover(true);
                    if self.state().is_selectable() {
                        self.manage_style();
                    }
                }
            }
            WidgetEvent::Exiting(widget_id) => {
                if widget_id == id && self.state().is_active() && self.state().is_hover() {
                    self.state_mut().set_hover(false);
                    self.state_mut().set_pressed(false);
                    if self.state().is_selectable() {
                        self.manage_style();
                    }
                }
            }
            WidgetEvent::Released(widget_id, mouse_in_px) => {
                if widget_id == id
                    && self.state().is_selectable()
                    && self.state().is_active()
                    && self.state().is_pressed()
                {
                    self.state_mut().set_pressed(false);
                    if self.state().is_draggable() {
                        self.state_mut().set_dragging_position(mouse_in_px);
                    }
                    self.manage_style();
                }
            }
            WidgetEvent::Pressed(widget_id, mouse_in_px) => {
                if widget_id == id
                    && self.state().is_selectable()
                    && self.state().is_active()
                    && !self.state().is_pressed()
                {
                    self.state_mut().set_pressed(true);
                    if self.state().is_draggable() {
                        self.state_mut().set_dragging_position(mouse_in_px);
                    }
                    self.manage_style();
                }
            }
            WidgetEvent::Dragging(widget_id, mouse_in_px) => {
                if widget_id == id
                    && self.state().is_draggable()
                    && self.state().is_selectable()
                    && self.state().is_active()
                {
                    self.state_mut()
                        .set_horizontal_alignment(HorizontalAlignment::None);
                    self.state_mut()
                        .set_vertical_alignment(VerticalAlignment::None);
                    let old_mouse_pos = self.state().get_dragging_position();
                    let offset = mouse_in_px - old_mouse_pos;
                    self.state_mut().set_dragging_position(mouse_in_px);
                    let current_pos = self.state().get_position();
                    self.set_position(current_pos + offset);
                }
            }
        }
    }

    #[inline]
    fn manage_input(&mut self, event: &MouseEvent, drawing_area_in_px: Vector4) {
        nrg_profiler::scoped_profile!("widget::manage_input");

        if !self.graphics().is_visible()
            || !WidgetGraphics::is_valid_drawing_area(drawing_area_in_px)
            || !self.state().is_active()
        {
            return;
        }
        let mut is_on_child = false;
        self.node().propagate_on_children(|w| {
            is_on_child |= w.state().is_hover();
        });
        if is_on_child {
            return;
        }

        let id = self.id();
        let mouse_in_px: Vector2 = [event.x as _, event.y as _].into();
        let is_inside = self.state().is_inside(mouse_in_px);

        let widget_event = if event.state == MouseState::Move {
            if is_inside && !self.state().is_hover() {
                Some(WidgetEvent::Entering(id))
            } else if !is_inside && self.state().is_hover() {
                Some(WidgetEvent::Exiting(id))
            } else if self.state().is_selectable()
                && self.state().is_pressed()
                && self.state().is_draggable()
            {
                Some(WidgetEvent::Dragging(id, mouse_in_px))
            } else {
                None
            }
        } else if event.state == MouseState::Down
            && is_inside
            && self.state().is_selectable()
            && !self.state().is_pressed()
        {
            Some(WidgetEvent::Pressed(id, mouse_in_px))
        } else if event.state == MouseState::Up
            && self.state().is_selectable()
            && self.state().is_pressed()
        {
            Some(WidgetEvent::Released(id, mouse_in_px))
        } else {
            None
        };

        if let Some(event) = widget_event {
            self.get_global_dispatcher()
                .write()
                .unwrap()
                .send(event.as_boxed())
                .ok();
        }
    }

    #[inline]
    fn move_to_layer(&mut self, layer: f32) {
        if (layer - self.graphics().get_layer()).abs() > f32::EPSILON {
            self.graphics_mut().set_layer(layer);
        }
    }

    #[inline]
    fn update_layout(&mut self, clip_rect: Vector4) {
        nrg_profiler::scoped_profile!("widget::update_layout");
        self.state_mut().set_dirty(false);
        let mut fit_size = self.compute_children_size(self.state().get_size());
        if self.state().should_keep_fixed_width() {
            fit_size.x = self.state().get_size().x;
        }
        if self.state().should_keep_fixed_height() {
            fit_size.y = self.state().get_size().y;
        }
        let (pos, size) = self.compute_offset_and_scale_from_alignment(
            clip_rect,
            self.state().get_position(),
            fit_size,
        );
        self.set_position(pos);
        self.set_size(size);
        self.update_layers();

        self.widget_on_layout_changed();

        self.graphics_mut().mark_as_dirty();
    }

    #[inline]
    fn update_layers(&mut self) {
        let layer = self.graphics().get_layer();
        self.node_mut().propagate_on_children_mut(|w| {
            w.move_to_layer(layer + DEFAULT_LAYER_OFFSET);
            w.update_layers();
        });
    }

    #[inline]
    fn is_hover(&self) -> bool {
        let mut is_hover = self.state().is_hover();
        if !is_hover {
            self.node().propagate_on_children(|w| {
                if w.is_hover() {
                    is_hover = true;
                }
            });
        }
        is_hover
    }

    #[inline]
    fn is_draggable(&self) -> bool {
        self.state().is_draggable()
    }

    #[inline]
    fn is_selectable(&self) -> bool {
        self.state().is_selectable()
    }

    #[inline]
    fn add_child(&mut self, widget: Box<dyn Widget>) -> Uid {
        let id = widget.id();
        self.node_mut().add_child(widget);

        self.mark_as_dirty();
        id
    }

    #[inline]
    fn set_visible(&mut self, visible: bool) {
        if self.graphics().is_visible() != visible {
            self.node_mut().propagate_on_children_mut(|w| {
                w.set_visible(visible);
            });
            self.graphics_mut().set_visible(visible);
            self.mark_as_dirty();
        }
    }
}
