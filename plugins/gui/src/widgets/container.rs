use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;

pub struct Container {
    fit_to_content: bool,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            fit_to_content: false,
        }
    }
}
impl Container {
    pub fn has_to_fit_content(&self) -> bool {
        self.fit_to_content
    }

    pub fn set_fit_to_content(&mut self, has_to_fit_content: bool) -> &mut Self {
        self.fit_to_content = has_to_fit_content;
        self
    }

    fn fit_to_content(widget: &mut Widget<Self>) {
        if !widget.get().has_to_fit_content() || !widget.get_data().node.has_children() {
            return;
        }

        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let node = &data.node;

        let mut children_min_pos: Vector2f = [Float::max_value(), Float::max_value()].into();
        let mut children_size: Vector2f = [1., 1.].into();
        node.propagate_on_children(|w| {
            let child_stroke =
                screen.convert_size_into_pixels(w.get_data().graphics.get_stroke().into());
            children_min_pos.x = children_min_pos
                .x
                .min(w.get_data().state.get_position().x - child_stroke.x);
            children_min_pos.y = children_min_pos
                .y
                .min(w.get_data().state.get_position().y - child_stroke.y);
            children_size.x = children_size
                .x
                .max(w.get_data().state.get_size().x + child_stroke.x * 2.);
            children_size.y = children_size
                .y
                .max(w.get_data().state.get_size().y + child_stroke.y * 2.);
        });
        data.state.set_position(children_min_pos);
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
