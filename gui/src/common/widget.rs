use super::*;
use crate::screen::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;
use std::any::Any;

pub const DEFAULT_LAYER_OFFSET: f32 = 0.001;
pub const DEFAULT_WIDGET_SIZE: Vector2f = Vector2f { x: 24., y: 24. };

pub struct WidgetData {
    pub node: WidgetNode,
    pub graphics: WidgetGraphics,
    pub state: WidgetState,
}

impl Default for WidgetData {
    fn default() -> Self {
        Self {
            node: WidgetNode::default(),
            graphics: WidgetGraphics::default(),
            state: WidgetState::default(),
        }
    }
}

pub trait WidgetBase: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_screen(&self) -> Screen;
    fn get_data(&self) -> &WidgetData;
    fn get_data_mut(&mut self) -> &mut WidgetData;
    fn update(
        &mut self,
        parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        input_handler: &InputHandler,
    );
    fn uninit(&mut self, renderer: &mut Renderer);
    fn id(&self) -> UID {
        self.get_data().node.get_id()
    }
    fn translate(&mut self, offset: Vector2f) {
        let data = self.get_data_mut();
        data.state.set_position(data.state.get_position() + offset);

        data.node.propagate_on_children_mut(|w| {
            w.translate(offset);
        });
    }

    fn scale(&mut self, scale: Vector2f) {
        let data = self.get_data_mut();
        data.state.set_size(data.state.get_size() * scale);

        data.node.propagate_on_children_mut(|w| {
            w.scale(scale);
        });
    }

    fn compute_offset_and_scale_from_alignment(&mut self, clip_rect: Vector4f) {
        let state = &self.get_data().state;
        let graphics = &self.get_data().graphics;
        let screen = &self.get_screen();

        let clip_min: Vector2f = [clip_rect.x, clip_rect.y].into();
        let clip_max: Vector2f = [clip_rect.z, clip_rect.w].into();

        let mut pos = state.get_position();
        let mut size = state.get_size();
        let stroke = screen.convert_size_into_pixels(graphics.get_stroke().into());

        match state.get_horizontal_alignment() {
            HorizontalAlignment::Left => {
                pos.x = clip_min.x + stroke.x;
            }
            HorizontalAlignment::Right => {
                pos.x = clip_max.x - (size.x + stroke.x);
            }
            HorizontalAlignment::Center => {
                pos.x = clip_min.x + (clip_max.x - clip_min.x).abs() * 0.5 - size.x * 0.5;
            }
            HorizontalAlignment::Stretch => {
                pos.x = clip_min.x + stroke.x;
                size.x = (clip_max.x - clip_min.x).abs() - stroke.x * 2.;
            }
            _ => {}
        }

        match state.get_vertical_alignment() {
            VerticalAlignment::Top => {
                pos.y = clip_min.y + stroke.y;
            }
            VerticalAlignment::Bottom => {
                pos.y = clip_max.y - (size.y + stroke.y);
            }
            VerticalAlignment::Center => {
                pos.y = clip_min.y + (clip_max.y - clip_min.y).abs() * 0.5 - size.y * 0.5;
            }
            VerticalAlignment::Stretch => {
                pos.y = clip_min.y + stroke.y;
                size.y = (clip_max.y - clip_min.y).abs() - stroke.y * 2.;
            }
            _ => {}
        }

        self.get_data_mut().state.set_position(pos);
        self.get_data_mut().state.set_size(size);
    }

    fn clip_in_area(&mut self, clip_rect: Vector4f) {
        let state = &self.get_data().state;
        let graphics = &self.get_data().graphics;
        let screen = &self.get_screen();

        let clip_min: Vector2f = [clip_rect.x, clip_rect.y].into();
        let clip_max: Vector2f = [clip_rect.z, clip_rect.w].into();

        let mut pos = state.get_position();
        let size = state.get_size();
        let stroke = screen.convert_size_into_pixels(graphics.get_stroke().into());

        pos.x = pos
            .x
            .max(clip_min.x + stroke.x)
            .min(clip_max.x - size.x - stroke.x);
        pos.y = pos
            .y
            .max(clip_min.y + stroke.y)
            .min(clip_max.y - size.y - stroke.y);

        self.get_data_mut().state.set_position(pos);
    }

    fn manage_style(&mut self) {
        let data = self.get_data_mut();

        if data.state.is_hover() {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Hover);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        } else if data.state.is_pressed() {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Pressed);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        } else if data.state.is_active() {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Active);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        } else {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Inactive);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        }
    }

    fn manage_events(&mut self, events: &mut EventsRw) {
        let id = self.id();
        let data = self.get_data_mut();
        let events = events.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                match event {
                    WidgetEvent::Entering(widget_id) => {
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_hover(true);
                        } else {
                            data.state.set_hover(false);
                        }
                    }
                    WidgetEvent::Exiting(widget_id) => {
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_hover(false);
                            data.state.set_pressed(false);
                        }
                    }
                    WidgetEvent::Released(widget_id) => {
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_pressed(false);
                        }
                    }
                    WidgetEvent::Pressed(widget_id) => {
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_pressed(true);
                        } else {
                            data.state.set_pressed(false);
                        }
                    }
                    WidgetEvent::Dragging(widget_id, mouse_in_px) => {
                        if *widget_id == id && data.state.is_draggable() {
                            data.state
                                .set_horizontal_alignment(HorizontalAlignment::None);
                            data.state.set_vertical_alignment(VerticalAlignment::None);
                            let pos = data.state.get_position() + *mouse_in_px;
                            data.state.set_position(pos);
                        }
                    }
                }
            }
        }
    }

    fn manage_input(&mut self, events: &mut EventsRw, input_handler: &InputHandler) {
        let id = self.id();
        let mut events = events.write().unwrap();
        let screen = self.get_screen();
        let data = self.get_data_mut();
        if !data.state.is_active() || !data.state.is_selectable() {
            return;
        }
        let mut is_on_child = false;
        data.node.propagate_on_children(|w| {
            is_on_child |= w.is_hover();
        });
        if is_on_child {
            return;
        }
        let mouse = screen.convert_position_into_pixels(Vector2f {
            x: input_handler.get_mouse_data().get_x() as _,
            y: input_handler.get_mouse_data().get_y() as _,
        });
        let is_inside = data.state.is_inside(mouse) && data.graphics.is_inside(mouse, &screen);
        if is_inside && !data.state.is_hover() {
            events.send_event(WidgetEvent::Entering(id));
            return;
        } else if !is_inside {
            if data.state.is_hover() {
                events.send_event(WidgetEvent::Exiting(id));
            }
            return;
        }
        let is_mouse_down = input_handler.get_mouse_data().is_pressed();
        if is_mouse_down && !data.state.is_pressed() {
            events.send_event(WidgetEvent::Pressed(id));
            return;
        } else if !is_mouse_down {
            if data.state.is_pressed() {
                events.send_event(WidgetEvent::Released(id));
            }
            return;
        }
        if data.state.is_pressed() && data.state.is_draggable() {
            let movement_in_pixels = screen.convert_position_into_pixels(Vector2f {
                x: input_handler.get_mouse_data().movement_x() as _,
                y: input_handler.get_mouse_data().movement_y() as _,
            });
            events.send_event(WidgetEvent::Dragging(id, movement_in_pixels));
        }
    }
    fn move_to_layer(&mut self, layer: f32) {
        let data = self.get_data_mut();
        data.state.set_layer(layer);
        data.graphics.move_to_layer(layer);
    }

    fn compute_clip_area(&self, parent_data: Option<&WidgetState>) -> Vector4f {
        let screen = self.get_screen();
        let clip_area: Vector4f = if let Some(parent_state) = parent_data {
            let parent_pos = parent_state.get_position();
            let parent_size = parent_state.get_size();
            [
                parent_pos.x,
                parent_pos.y,
                parent_pos.x + parent_size.x,
                parent_pos.y + parent_size.y,
            ]
            .into()
        } else {
            let size = screen.get_size();
            [0., 0., size.x, size.y].into()
        };
        clip_area
    }

    fn update_layout(&mut self, parent_data: Option<&WidgetState>) {
        let clip_area = self.compute_clip_area(parent_data);

        if !self.get_data().state.is_pressed() {
            self.compute_offset_and_scale_from_alignment(clip_area);
        }
        self.clip_in_area(clip_area);
        self.update_layers();

        let data = self.get_data_mut();
        let parent_data = Some(&data.state);
        data.node.propagate_on_children_mut(|w| {
            w.update_layout(parent_data);
        });
    }

    fn update_layers(&mut self) {
        let data = self.get_data_mut();
        let layer = data.state.get_layer();

        data.node.propagate_on_children_mut(|w| {
            w.move_to_layer(layer - DEFAULT_LAYER_OFFSET * 2.0);
            w.update_layers();
        });
    }
    fn set_stroke(&mut self, stroke: f32);
    fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn set_border_color(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn set_position(&mut self, pos: Vector2f);
    fn set_size(&mut self, size: Vector2f);
    fn is_hover(&self) -> bool {
        self.get_data().state.is_hover()
    }
    fn set_draggable(&mut self, draggable: bool) {
        self.get_data_mut().state.set_draggable(draggable);
    }
    fn is_draggable(&self) -> bool {
        self.get_data().state.is_draggable()
    }
    fn set_selectable(&mut self, selectable: bool) {
        self.get_data_mut().state.set_selectable(selectable);
    }
    fn is_selectable(&self) -> bool {
        self.get_data().state.is_selectable()
    }
}

pub struct Widget<T>
where
    T: WidgetTrait,
{
    data: WidgetData,
    screen: Screen,
    inner: T,
}

impl<T> WidgetBase for Widget<T>
where
    T: WidgetTrait + Default + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_screen(&self) -> Screen {
        self.screen.clone()
    }

    fn get_data(&self) -> &WidgetData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut WidgetData {
        &mut self.data
    }
    fn update(
        &mut self,
        parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        input_handler: &InputHandler,
    ) {
        let state_data = Some(&self.data.state);
        self.data.node.propagate_on_children_mut(|w| {
            w.update(state_data, renderer, events, input_handler);
        });

        self.manage_input(events, input_handler);
        self.manage_events(events);
        self.manage_style();

        T::update(self, parent_data, renderer, events, input_handler);
        self.update_layout(parent_data);

        self.data.graphics.update(renderer);
    }

    fn uninit(&mut self, renderer: &mut Renderer) {
        self.data
            .node
            .propagate_on_children_mut(|w| w.uninit(renderer));
        T::uninit(self, renderer);
        self.data.graphics.uninit(renderer);
    }
    fn set_stroke(&mut self, stroke: f32) {
        self.stroke(stroke);
    }
    fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color(r, g, b, a);
    }
    fn set_border_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.border_color(r, g, b, a);
    }
    fn set_position(&mut self, pos: Vector2f) {
        self.position(pos);
    }
    fn set_size(&mut self, size: Vector2f) {
        self.size(size);
    }
}

pub trait WidgetTrait: Send + Sync + Sized {
    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer);
    fn update(
        widget: &mut Widget<Self>,
        parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        input_handler: &InputHandler,
    );
    fn uninit(widget: &mut Widget<Self>, renderer: &mut Renderer);
}

impl<T> Widget<T>
where
    T: WidgetTrait + Default + 'static,
{
    pub fn new(screen: Screen) -> Self {
        Self {
            data: WidgetData::default(),
            inner: T::default(),
            screen,
        }
    }

    pub fn get(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn init(&mut self, renderer: &mut Renderer) -> &mut Self {
        T::init(self, renderer);
        self
    }

    pub fn add_child<W>(&mut self, mut widget: Widget<W>) -> UID
    where
        W: WidgetTrait + Default + 'static,
    {
        let id = widget.id();
        let stroke = widget
            .get_screen()
            .convert_size_into_pixels(widget.get_data().graphics.get_stroke().into());
        widget.translate(self.data.state.get_position() + stroke);
        self.data.node.add_child(widget);
        self.update_layout(None);
        id
    }

    pub fn get_child<W>(&mut self, uid: UID) -> Option<&mut Widget<W>>
    where
        W: WidgetTrait + Default + 'static,
    {
        if let Some(widget) = self.data.node.get_child(uid) {
            let w = widget as &mut Widget<W>;
            return Some(w);
        }
        None
    }

    pub fn propagate_on_child<F>(&self, uid: UID, f: F)
    where
        F: Fn(&dyn WidgetBase),
    {
        self.data.node.propagate_on_child(uid, f);
    }
    pub fn propagate_on_child_mut<F>(&mut self, uid: UID, f: F)
    where
        F: FnMut(&mut dyn WidgetBase),
    {
        self.data.node.propagate_on_child_mut(uid, f);
    }

    pub fn get_position(&self) -> Vector2f {
        self.data.state.get_position()
    }
    pub fn horizontal_alignment(&mut self, alignment: HorizontalAlignment) -> &mut Self {
        self.data.state.set_horizontal_alignment(alignment);
        self
    }
    pub fn vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
        self.data.state.set_vertical_alignment(alignment);
        self
    }

    pub fn draggable(&mut self, draggable: bool) -> &mut Self {
        self.set_draggable(draggable);
        self
    }
    pub fn selectable(&mut self, selectable: bool) -> &mut Self {
        self.set_selectable(selectable);
        self
    }

    pub fn position(&mut self, pos: Vector2f) -> &mut Self {
        let offset = pos - self.data.state.get_position();
        self.translate(offset);
        self
    }

    pub fn size(&mut self, size: Vector2f) -> &mut Self {
        let scale = size / self.data.state.get_size();
        self.scale(scale);
        self
    }

    pub fn color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.data.graphics.set_color([r, g, b, a].into());
        self
    }
    pub fn border_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.data.graphics.set_border_color([r, g, b, a].into());
        self
    }
    pub fn stroke(&mut self, stroke: f32) -> &mut Self {
        let stroke: Vector3f = self
            .screen
            .convert_size_from_pixels([stroke, stroke].into())
            .into();
        self.data.graphics.set_stroke(stroke.x);
        self
    }
}
