use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFillType {
    Vertical,
    Horizontal,
}

pub struct Container {
    fill_type: ContainerFillType,
    fit_to_content: bool,
    space_between_elements: f32,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            fill_type: ContainerFillType::Vertical,
            fit_to_content: true,
            space_between_elements: 0.,
        }
    }
}

impl Container {
    pub fn set_fill_type(&mut self, fill_type: ContainerFillType) -> &mut Self {
        self.fill_type = fill_type;
        self
    }
    pub fn get_fill_type(&self) -> ContainerFillType {
        self.fill_type
    }
    pub fn set_fit_to_content(&mut self, fit_to_content: bool) -> &mut Self {
        self.fit_to_content = fit_to_content;
        self
    }
    pub fn has_fit_to_content(&self) -> bool {
        self.fit_to_content
    }
    pub fn set_space_between_elements(&mut self, space_in_px: f32) -> &mut Self {
        self.space_between_elements = space_in_px;
        self
    }
    pub fn get_space_between_elements(&self) -> f32 {
        self.space_between_elements
    }

    fn fit_to_content(widget: &mut Widget<Self>) {
        if !widget.get_data().node.has_children() {
            return;
        }

        let fill_type = widget.get().get_fill_type();
        let fit_to_content = widget.get().has_fit_to_content();
        let space = widget.get().get_space_between_elements();

        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let node = &mut data.node;
        let parent_pos = data.state.get_position();
        let parent_size = data.state.get_size();

        let mut children_min_pos: Vector2f = [Float::max_value(), Float::max_value()].into();
        let mut children_size: Vector2f = [0., 0.].into();
        let mut index = 0;
        node.propagate_on_children_mut(|w| {
            let child_stroke =
                screen.convert_size_into_pixels(w.get_data().graphics.get_stroke().into());
            let child_state = &mut w.get_data_mut().state;
            let child_pos = child_state.get_position();
            let child_size = child_state.get_size();
            children_min_pos.x = children_min_pos.x.min(child_pos.x - child_stroke.x);
            children_min_pos.y = children_min_pos.y.min(child_pos.y - child_stroke.y);
            match fill_type {
                ContainerFillType::Vertical => {
                    child_state.set_vertical_alignment(VerticalAlignment::None);
                    if index > 0 {
                        children_size.y += space;
                    }
                    if !child_state.is_dragging() {
                        child_state
                            .set_position([child_pos.x, parent_pos.y + children_size.y].into());
                    }
                    children_size.y += child_size.y + child_stroke.y * 2.;
                    if fit_to_content {
                        children_size.x = children_size.x.max(child_size.x + child_stroke.x * 2.);
                    } else {
                        children_size.x = parent_size.x;
                    }
                }
                ContainerFillType::Horizontal => {
                    child_state.set_horizontal_alignment(HorizontalAlignment::None);
                    if index > 0 {
                        children_size.x += space;
                    }
                    if !child_state.is_dragging() {
                        child_state
                            .set_position([parent_pos.x + children_size.x, child_pos.y].into());
                    }
                    children_size.x += child_size.x + child_stroke.x * 2.;
                    if fit_to_content {
                        children_size.y = children_size.y.max(child_size.y + child_stroke.y * 2.);
                    } else {
                        children_size.y = parent_size.y;
                    }
                }
            }
            index += 1;
        });
        //data.state.set_position(children_min_pos);
        data.state.set_size(children_size);
    }
}

impl WidgetTrait for Container {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let data = widget.get_data_mut();
        data.graphics.init(renderer, "UI");
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        _input_handler: &InputHandler,
    ) {
        Container::fit_to_content(widget);

        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let pos = screen.convert_from_pixels_into_screen_space(data.state.get_position());
        let size = screen.convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn uninit(_widget: &mut Widget<Self>, _renderer: &mut Renderer) {}

    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
