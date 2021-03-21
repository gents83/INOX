use super::*;
use nrg_graphics::*;
use nrg_platform::*;
use nrg_serialize::*;

pub enum CheckboxEvent {
    Checked(UID),
    Unchecked(UID),
}
impl Event for CheckboxEvent {}

pub struct Checkbox {
    container_data: ContainerData,
    is_checked: bool,
    checked_widget: UID,
}

unsafe impl Send for Checkbox {}
unsafe impl Sync for Checkbox {}

impl ContainerTrait for Checkbox {
    fn get_container_data(&self) -> &ContainerData {
        &self.container_data
    }
    fn get_container_data_mut(&mut self) -> &mut ContainerData {
        &mut self.container_data
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            container_data: ContainerData::default(),
            is_checked: false,
            checked_widget: INVALID_ID,
        }
    }
}

impl Checkbox {
    pub fn set_checked(&mut self, checked: bool) -> &mut Self {
        self.is_checked = checked;
        self
    }

    fn check_state_change(id: UID, old_state: bool, events_rw: &mut EventsRw) -> (bool, bool) {
        let mut changed = false;
        let mut new_state = false;

        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Released(widget_id) = event {
                    if id == *widget_id {
                        if !old_state {
                            changed = true;
                            new_state = true;
                        } else if old_state {
                            changed = true;
                            new_state = false;
                        }
                    }
                }
            }
        }

        (changed, new_state)
    }

    pub fn update_checked(widget: &mut Widget<Self>, events_rw: &mut EventsRw) {
        let id = widget.id();
        let (changed, new_state) = Self::check_state_change(id, widget.get().is_checked, events_rw);

        let mut events = events_rw.write().unwrap();
        if changed {
            if let Some(inner_widget) = widget.get_child::<Panel>(widget.get().checked_widget) {
                if new_state {
                    inner_widget
                        .get_data_mut()
                        .graphics
                        .set_style(WidgetStyle::full_active());

                    events.send_event(CheckboxEvent::Checked(id));
                } else {
                    inner_widget
                        .get_data_mut()
                        .graphics
                        .set_style(WidgetStyle::full_inactive());

                    events.send_event(CheckboxEvent::Unchecked(id));
                }
                widget.get_mut().set_checked(new_state);
            }
        }
    }
}

impl WidgetTrait for Checkbox {
    fn init(widget: &mut Widget<Self>, renderer: &mut Renderer) {
        let screen = widget.get_screen();
        let data = widget.get_data_mut();

        data.graphics
            .init(renderer, "UI")
            .set_style(WidgetStyle::default());
        widget
            .size(DEFAULT_WIDGET_SIZE)
            .draggable(false)
            .selectable(true)
            .stroke(2)
            .get_mut()
            .set_fill_type(ContainerFillType::None)
            .set_fit_to_content(false);

        let inner_size = widget.get_data().state.get_size() - [8, 8].into();
        let mut panel = Widget::<Panel>::new(screen);
        panel
            .init(renderer)
            .draggable(false)
            .size(inner_size)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .selectable(false)
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::full_inactive());
        widget.get_mut().checked_widget = widget.add_child(panel);
    }

    fn update(
        widget: &mut Widget<Self>,
        _parent_data: Option<&WidgetState>,
        _renderer: &mut Renderer,
        events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        Self::fit_to_content(widget);
        Self::update_checked(widget, events);

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
