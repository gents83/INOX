use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

use super::screen::*;

const LAYER_OFFSET: f32 = 0.001;

pub struct WidgetState {
    pub pos: Vector2f,
    pub size: Vector2f,
    pub is_active: bool,
    pub is_draggable: bool,
    pub is_hover: bool,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            pos: Vector2f::default(),
            size: Vector2f::default(),
            is_active: true,
            is_draggable: false,
            is_hover: false,
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

pub struct WidgetGraphics {
    pub material_id: MaterialId,
    pub mesh_id: MeshId,
    pub mesh_data: MeshData,
}

impl Default for WidgetGraphics {
    fn default() -> Self {
        Self {
            material_id: INVALID_ID,
            mesh_id: INVALID_ID,
            mesh_data: MeshData::default(),
        }
    }
}

impl WidgetGraphics {
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.mesh_data.set_vertex_color([r, g, b].into());
        self
    }
    pub fn move_to_up_layer(&mut self) -> &mut Self {
        self.mesh_data.translate([0.0, 0.0, LAYER_OFFSET].into());
        self
    }

    pub fn translate(&mut self, movement: Vector2f) -> &mut Self {
        self.mesh_data.translate(movement.into());
        self
    }
    pub fn scale(&mut self, scale: Vector2f) -> &mut Self {
        self.mesh_data.scale(scale.into());
        self
    }
    pub fn is_inside(&self, pos: Vector2f) -> bool {
        let mut i = 0;
        let count = self.mesh_data.indices.len();
        while i < count {
            let v1 = self.mesh_data.vertices[self.mesh_data.indices[i] as usize].pos;
            let v2 = self.mesh_data.vertices[self.mesh_data.indices[i + 1] as usize].pos;
            let v3 = self.mesh_data.vertices[self.mesh_data.indices[i + 2] as usize].pos;
            if is_point_in_triangle(v1.into(), v2.into(), v3.into(), pos.x, pos.y) {
                return true;
            }
            i += 3;
        }
        false
    }

    pub fn update(&mut self, renderer: &mut Renderer) -> &mut Self {
        renderer.update_mesh(self.material_id, self.mesh_id, &self.mesh_data);
        self
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) -> &mut Self {
        renderer.remove_mesh(self.material_id, self.mesh_id);
        renderer.remove_material(self.material_id);
        self.material_id = INVALID_ID;
        self.mesh_id = INVALID_ID;
        self.mesh_data.clear();
        self
    }
}

pub struct WidgetNode {
    pub id: UID,
    pub children: Vec<Box<dyn WidgetBase>>,
}

impl Default for WidgetNode {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            children: Vec::new(),
        }
    }
}

impl WidgetNode {
    pub fn add_child<W: 'static + WidgetTrait>(&mut self, widget: Widget<W>) -> &mut Self {
        self.children.push(Box::new(widget));
        self
    }
    pub fn propagate_on_children<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut dyn WidgetBase),
    {
        self.children.iter_mut().for_each(|w| f(w.as_mut()));
        self
    }
}

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

pub struct Widget<T> {
    data: WidgetData,
    inner: T,
    screen: Screen,
}

pub trait WidgetTrait: Send + Sync {
    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn init(&mut self, data: &mut WidgetData, screen: &Screen, renderer: &mut Renderer);
    fn update(
        &mut self,
        data: &mut WidgetData,
        screen: &Screen,
        renderer: &mut Renderer,
        input_handler: &InputHandler,
    );
    fn uninit(&mut self, data: &mut WidgetData, screen: &Screen, renderer: &mut Renderer);
}

pub trait WidgetBase: Send + Sync {
    fn update(&mut self, renderer: &mut Renderer, input_handler: &InputHandler);
    fn uninit(&mut self, renderer: &mut Renderer);
    fn scale(&mut self, scale: Vector2f);
    fn translate(&mut self, offset: Vector2f);
    fn manage_input(&mut self, input_handler: &InputHandler) -> bool;
    fn update_layout(&mut self);
}

impl<T> WidgetBase for Widget<T>
where
    T: WidgetTrait,
{
    fn update(&mut self, renderer: &mut Renderer, input_handler: &InputHandler) {
        self.data
            .node
            .propagate_on_children(|w| w.update(renderer, input_handler));
        self.manage_input(input_handler);
        self.inner
            .update(&mut self.data, &self.screen, renderer, input_handler);
        self.data.graphics.update(renderer);
    }

    fn uninit(&mut self, renderer: &mut Renderer) {
        self.data.node.propagate_on_children(|w| w.uninit(renderer));
        self.inner.uninit(&mut self.data, &self.screen, renderer);
        self.data.graphics.uninit(renderer);
    }
    fn translate(&mut self, offset: Vector2f) {
        self.data
            .node
            .propagate_on_children(|w| w.translate(offset));
        let screen_old_pos = self
            .screen
            .convert_into_screen_space(self.data.state.get_position());
        self.data.graphics.translate(-screen_old_pos);
        self.data
            .state
            .set_position(self.data.state.get_position() + offset);
        let screen_pos = self
            .screen
            .convert_into_screen_space(self.data.state.get_position());
        self.data.graphics.translate(screen_pos);
    }
    fn scale(&mut self, scale: Vector2f) {
        self.data.node.propagate_on_children(|w| w.translate(scale));
        self.data.state.set_size(self.data.state.get_size() * scale);
        let pos = self
            .screen
            .convert_into_screen_space(self.data.state.get_position());
        self.data.graphics.translate(-pos);
        self.data.graphics.scale(scale);
        self.data.graphics.translate(pos);
    }

    fn manage_input(&mut self, input_handler: &InputHandler) -> bool {
        let mut managed_by_children = false;
        self.data
            .node
            .propagate_on_children(|w| managed_by_children |= w.manage_input(input_handler));
        if managed_by_children {
            return true;
        }
        if !self.is_draggable() {
            return false;
        }
        let mouse = self.screen.convert_into_pixels(Vector2f {
            x: input_handler.get_mouse_data().get_x() as _,
            y: input_handler.get_mouse_data().get_y() as _,
        });
        self.data.state.is_hover = self.data.state.is_inside(mouse);
        if !self.data.state.is_hover {
            return false;
        }
        let mouse_in_screen_space = self.screen.convert_into_screen_space(mouse);
        if !self.data.graphics.is_inside(mouse_in_screen_space) {
            return false;
        }
        if !input_handler.get_mouse_data().is_dragging() {
            return false;
        }
        let movement = Vector2f {
            x: input_handler.get_mouse_data().movement_x() as _,
            y: input_handler.get_mouse_data().movement_y() as _,
        };
        let movement_in_pixels = self.screen.convert_into_pixels(movement);
        let pos = self.data.state.get_position() + movement_in_pixels;
        self.set_position(pos);
        true
    }

    fn update_layout(&mut self) {
        let pos = self.get_position();
        self.data.node.propagate_on_children(|w| {
            w.translate(pos);
            w.update_layout();
        });
    }
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
    pub fn add_child<W: 'static + WidgetTrait>(&mut self, widget: Widget<W>) -> &mut Self {
        let mut widget = widget;
        widget.data.graphics.move_to_up_layer();
        self.data.node.add_child(widget);
        self.update_layout();
        self
    }

    pub fn is_hover(&self) -> bool {
        self.data.state.is_hover
    }
    pub fn is_draggable(&self) -> bool {
        self.data.state.is_active
    }

    pub fn init(&mut self, renderer: &mut Renderer) -> &mut Self {
        self.inner.init(&mut self.data, &self.screen, renderer);
        self
    }

    pub fn get_position(&self) -> Vector2f {
        self.data.state.get_position()
    }

    pub fn get_size(&self) -> Vector2f {
        self.data.state.get_size()
    }

    pub fn set_position(&mut self, pos: Vector2f) -> &mut Self {
        let offset = pos - self.data.state.get_position();
        self.translate(offset);
        self
    }

    pub fn set_size(&mut self, size: Vector2f) -> &mut Self {
        let old_screen_scale = self.screen.convert_from_pixels(self.data.state.get_size());
        let screen_size = self.screen.convert_from_pixels(size);
        let scale = screen_size / old_screen_scale;
        self.scale(scale);
        self
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.data.graphics.set_color(r, g, b);
        self
    }
}
