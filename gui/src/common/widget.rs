use super::*;
use crate::screen::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
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

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Widget<T>
where
    T: WidgetTrait,
{
    data: WidgetData,
    #[serde(skip)]
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

    fn set_stroke(&mut self, stroke: u32) {
        self.stroke(stroke);
    }
    fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color(r, g, b, a);
    }
    fn set_border_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.border_color(r, g, b, a);
    }
    fn set_position(&mut self, pos_in_px: Vector2u) {
        self.position(pos_in_px);
    }
    fn set_size(&mut self, size_in_px: Vector2u) {
        self.size(size_in_px);
    }
}

impl<T> Widget<T>
where
    T: WidgetTrait,
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

    pub fn add_child<W: 'static>(&mut self, mut widget: Widget<W>) -> UID
    where
        W: WidgetTrait,
    {
        let id = widget.id();
        let stroke = widget
            .get_screen()
            .convert_size_into_pixels(widget.get_data().graphics.get_stroke().into());
        widget.translate((self.data.state.get_position() + stroke).convert());
        self.data.node.add_child(widget);
        id
    }

    pub fn remove_children(&mut self, renderer: &mut Renderer) {
        self.data.node.propagate_on_children_mut(|w| {
            w.get_data_mut().graphics.remove_meshes(renderer);
        });
        self.data.node.remove_children();
    }

    pub fn get_num_children(&self) -> usize {
        self.data.node.get_num_children()
    }

    pub fn get_child<W>(&mut self, uid: UID) -> Option<&mut Widget<W>>
    where
        W: WidgetTrait,
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

    pub fn get_position(&self) -> Vector2u {
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

    pub fn position(&mut self, pos_in_px: Vector2u) -> &mut Self {
        let offset: Vector2i = pos_in_px.convert() - self.data.state.get_position().convert();
        self.translate(offset);
        self
    }

    pub fn size(&mut self, size_in_px: Vector2u) -> &mut Self {
        let scale: Vector2f = [
            size_in_px.x as f32 / self.data.state.get_size().x as f32,
            size_in_px.y as f32 / self.data.state.get_size().y as f32,
        ]
        .into();
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
    pub fn stroke(&mut self, stroke: u32) -> &mut Self {
        let stroke: Vector3f = self
            .screen
            .convert_size_from_pixels([stroke, stroke].into())
            .into();
        self.data.graphics.set_stroke(stroke.x);
        self
    }
}
