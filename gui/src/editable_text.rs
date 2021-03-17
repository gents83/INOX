use super::*;
use nrg_graphics::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct EditableText {
    container_data: ContainerData,
    text_widget: UID,
    indicator_widget: UID,
}

unsafe impl Send for EditableText {}
unsafe impl Sync for EditableText {}

impl ContainerTrait for EditableText {
    fn get_container_data(&self) -> &ContainerData {
        &self.container_data
    }
    fn get_container_data_mut(&mut self) -> &mut ContainerData {
        &mut self.container_data
    }
}

impl Default for EditableText {
    fn default() -> Self {
        Self {
            container_data: ContainerData::default(),
            text_widget: INVALID_ID,
            indicator_widget: INVALID_ID,
        }
    }
}

impl EditableText {
    fn check_focus(id: UID, events_rw: &mut EventsRw) -> bool {
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Pressed(widget_id) = event {
                    if id == *widget_id {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl WidgetTrait for EditableText {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();
        let size = DEFAULT_WIDGET_SIZE * screen.get_scale_factor();

        data.graphics.init(renderer, "UI");
        widget
            .size(size)
            .draggable(false)
            .selectable(true)
            .stroke(2.)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .get_mut()
            .set_fill_type(ContainerFillType::Horizontal)
            .set_space_between_elements(2.)
            .set_fit_to_content(false);

        let mut text = Widget::<Text>::new(screen.clone());
        text.init(renderer)
            .draggable(false)
            .size(size)
            .vertical_alignment(VerticalAlignment::Stretch)
            .horizontal_alignment(HorizontalAlignment::Left);
        text.get_mut().set_text("Edit me");
        widget.get_mut().text_widget = widget.add_child(text);

        let mut indicator = Widget::<Indicator>::new(screen);
        indicator.init(renderer);
        widget.get_mut().indicator_widget = widget.add_child(indicator);
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        _events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        Self::fit_to_content(widget);

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
