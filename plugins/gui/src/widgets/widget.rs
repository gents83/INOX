use super::{align::*, graphics::*, node::*, screen::*, state::*, style::*};
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

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

pub trait WidgetBase: Send + Sync {
    fn get_screen(&self) -> Screen;
    fn get_data(&self) -> &WidgetData;
    fn get_data_mut(&mut self) -> &mut WidgetData;
    fn update(
        &mut self,
        parent_data: Option<&WidgetState>,
        renderer: &mut Renderer,
        input_handler: &InputHandler,
    ) -> bool;
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

    fn compute_offset_and_scale_from_alignment(&mut self, clip_area: Vector4f) {
        let state = &self.get_data().state;
        let node = &self.get_data().node;
        let screen = &self.get_screen();

        let old_pos = state.get_position();
        let old_size = state.get_size();
        let mut pos = screen.convert_from_pixels_into_screen_space(old_pos);
        let mut size = screen.convert_size_from_pixels(old_size);

        if !state.is_dragging() {
            match state.get_horizontal_alignment() {
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
                HorizontalAlignment::FitToContent => {
                    if node.has_children() {
                        let mut children_min_pos: f32 = Float::max_value();
                        let mut children_size: f32 = Float::max_value();
                        node.propagate_on_children(|w| {
                            let stroke = screen.convert_size_into_pixels(
                                w.get_data().graphics.get_stroke().into(),
                            );
                            children_min_pos = children_min_pos
                                .min(w.get_data().state.get_position().x - stroke.x);
                            children_size =
                                children_size.min(w.get_data().state.get_size().x + stroke.x * 2.);
                        });
                        pos.x = screen
                            .convert_from_pixels_into_screen_space([children_min_pos, 0.].into())
                            .x;
                        if children_size >= 0. {
                            size.x = screen
                                .convert_size_from_pixels([children_size, 1.].into())
                                .x;
                        }
                    }
                }
                _ => {}
            }
            if pos.x < clip_area.x {
                pos.x = clip_area.x;
            } else if pos.x > clip_area.z - size.x {
                pos.x = clip_area.z - size.x;
            }

            match state.get_vertical_alignment() {
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
                VerticalAlignment::FitToContent => {
                    if node.has_children() {
                        let mut children_min_pos: f32 = Float::max_value();
                        let mut children_size: f32 = Float::max_value();
                        node.propagate_on_children(|w| {
                            let stroke = screen.convert_size_into_pixels(
                                w.get_data().graphics.get_stroke().into(),
                            );
                            children_min_pos = children_min_pos
                                .min(w.get_data().state.get_position().y - stroke.y);
                            children_size =
                                children_size.min(w.get_data().state.get_size().y + stroke.y * 2.);
                        });
                        pos.y = screen
                            .convert_from_pixels_into_screen_space([0., children_min_pos].into())
                            .y;
                        if children_size >= 0. {
                            size.y = screen
                                .convert_size_from_pixels([1., children_size].into())
                                .y;
                        }
                    }
                }
                _ => {}
            }
        }
        if pos.y < clip_area.y {
            pos.y = clip_area.y;
        } else if pos.y > clip_area.w - size.y {
            pos.y = clip_area.w - size.y;
        }

        self.get_data_mut()
            .state
            .set_position(screen.convert_from_screen_space_into_pixels(pos));
        self.get_data_mut()
            .state
            .set_size(screen.convert_size_into_pixels(size));
    }

    fn manage_input(&mut self, input_managed: bool, input_handler: &InputHandler) -> bool {
        let screen = self.get_screen();
        let data = self.get_data_mut();
        if !data.state.is_active() {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Inactive);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
            return false;
        }
        let mouse = screen.convert_position_into_pixels(Vector2f {
            x: input_handler.get_mouse_data().get_x() as _,
            y: input_handler.get_mouse_data().get_y() as _,
        });
        data.state.set_hover(data.state.is_inside(mouse));
        if input_managed || !data.state.is_hover() {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Active);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
            return false;
        }
        let mouse_in_screen_space = screen.convert_from_pixels_into_screen_space(mouse);
        if !data.graphics.is_inside(mouse_in_screen_space) {
            data.state.set_hover(false);
            return false;
        } else {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Hover);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        }
        if !data.state.is_draggable() {
            return true;
        }
        if !input_handler.get_mouse_data().is_dragging() {
            data.state.set_dragging(false);
            return true;
        } else {
            let (color, border_color) = data.graphics.get_colors(WidgetInteractiveState::Dragging);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        }
        data.state.set_dragging(true);
        let movement = Vector2f {
            x: input_handler.get_mouse_data().movement_x() as _,
            y: input_handler.get_mouse_data().movement_y() as _,
        };
        let movement_in_pixels = screen.convert_position_into_pixels(movement);
        let pos = data.state.get_position() + movement_in_pixels;
        self.set_position(pos);
        true
    }
    fn move_to_layer(&mut self, layer: f32) {
        let data = self.get_data_mut();
        data.state.set_layer(layer);
        data.graphics.move_to_layer(layer);
    }

    fn compute_clip_area(&self, parent_data: Option<&WidgetState>) -> Vector4f {
        let screen = self.get_screen();
        let clip_area: Vector4f = if let Some(parent_state) = parent_data {
            let parent_pos =
                screen.convert_from_pixels_into_screen_space(parent_state.get_position());
            let parent_size = screen.convert_size_from_pixels(parent_state.get_size());
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
        clip_area
    }

    fn update_layout(&mut self, parent_data: Option<&WidgetState>) {
        let clip_area = self.compute_clip_area(parent_data);
        self.compute_offset_and_scale_from_alignment(clip_area);
        self.update_layers();
        /*
        println!("{}", std::any::type_name::<Self>());
        println!("{:?}", self.get_data().state.get_position());
        println!("{:?}", self.get_data().state.get_size());
        */
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
            w.move_to_layer(layer - LAYER_OFFSET * 2.0);
            w.update_layers();
        });
    }
    fn set_stroke(&mut self, stroke: f32);
    fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn set_border_color(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn set_position(&mut self, pos: Vector2f);
    fn set_size(&mut self, size: Vector2f);
    fn set_margins(&mut self, top: f32, left: f32, right: f32, bottom: f32);

    fn set_draggable(&mut self, draggable: bool) {
        self.get_data_mut().state.set_draggable(draggable);
    }
    fn is_hover(&self) -> bool {
        self.get_data().state.is_hover()
    }
    fn is_draggable(&self) -> bool {
        self.get_data().state.is_draggable()
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
    T: WidgetTrait,
{
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
        input_handler: &InputHandler,
    ) -> bool {
        let clip_area = self.compute_clip_area(parent_data);

        T::update(self, parent_data, renderer, input_handler);
        self.data.graphics.update(renderer, clip_area);

        let mut input_managed = false;
        let parent_data = Some(&self.data.state);
        self.data.node.propagate_on_children_mut(|w| {
            input_managed |= w.update(parent_data, renderer, input_handler);
        });
        input_managed |= self.manage_input(input_managed, input_handler);
        input_managed
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
    fn set_margins(&mut self, top: f32, left: f32, right: f32, bottom: f32) {
        self.margins(top, left, right, bottom);
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
        input_handler: &InputHandler,
    );
    fn uninit(widget: &mut Widget<Self>, renderer: &mut Renderer);
}

impl<T> Widget<T>
where
    T: WidgetTrait,
{
    pub fn new(inner: T, screen: Screen) -> Self {
        Self {
            data: WidgetData::default(),
            inner,
            screen,
        }
    }

    #[allow(dead_code)]
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

    pub fn add_child<W: 'static + WidgetTrait>(&mut self, mut widget: Widget<W>) -> UID {
        let id = widget.id();
        let stroke = widget
            .get_screen()
            .convert_size_into_pixels(widget.get_data().graphics.get_stroke().into());
        widget.translate(self.data.state.get_position() + stroke);
        self.data.node.add_child(widget);
        self.update_layout(None);
        id
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
    pub fn margins(&mut self, top: f32, left: f32, right: f32, bottom: f32) -> &mut Self {
        self.data.state.set_margins(WidgetMargins {
            top,
            left,
            right,
            bottom,
        });
        self
    }
    pub fn horizontal_alignment(&mut self, alignment: HorizontalAlignment) -> &mut Self {
        self.data.state.set_horizontal_alignment(alignment);
        self.update_layout(None);
        self
    }
    pub fn vertical_alignment(&mut self, alignment: VerticalAlignment) -> &mut Self {
        self.data.state.set_vertical_alignment(alignment);
        self.update_layout(None);
        self
    }

    pub fn draggable(&mut self, draggable: bool) -> &mut Self {
        self.set_draggable(draggable);
        self
    }

    pub fn position(&mut self, pos: Vector2f) -> &mut Self {
        let offset = pos - self.data.state.get_position();
        self.translate(offset);
        self.update_layout(None);
        self
    }

    pub fn size(&mut self, size: Vector2f) -> &mut Self {
        let old_screen_scale = self
            .screen
            .convert_size_from_pixels(self.data.state.get_size());
        let screen_size = self.screen.convert_size_from_pixels(size);
        let scale = screen_size / old_screen_scale;
        self.scale(scale);
        self.update_layout(None);
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
