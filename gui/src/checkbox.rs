use super::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub enum CheckboxEvent {
    Checked(UID),
    Unchecked(UID),
}
impl Event for CheckboxEvent {}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Checkbox {
    is_checked: bool,
    #[serde(skip)]
    checked_widget: UID,
    #[serde(skip)]
    data: WidgetData,
}
implement_widget!(Checkbox);

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            is_checked: false,
            checked_widget: INVALID_ID,
            data: WidgetData::default(),
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

    pub fn update_checked(&mut self, events_rw: &mut EventsRw) {
        let id = self.id();
        let (changed, new_state) = Self::check_state_change(id, self.is_checked, events_rw);

        let mut events = events_rw.write().unwrap();
        if changed {
            let checked_id = self.checked_widget;
            if let Some(inner_widget) = self.get_data_mut().node.get_child::<Panel>(checked_id) {
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
                self.set_checked(new_state);
            }
        }
    }
}

impl InternalWidget for Checkbox {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        let data = self.get_data_mut();
        let default_size = DEFAULT_WIDGET_SIZE * Screen::get_scale_factor();

        data.graphics
            .init(renderer, "UI")
            .set_style(WidgetStyle::default());
        self.size(default_size)
            .draggable(false)
            .selectable(true)
            .stroke(2);

        let inner_size = default_size - default_size / 4;
        let mut panel = Panel::default();
        panel
            .init(renderer)
            .draggable(false)
            .size(inner_size)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .selectable(false)
            .fit_to_content(false)
            .get_data_mut()
            .graphics
            .set_style(WidgetStyle::full_inactive());
        self.checked_widget = self.add_child(Box::new(panel));
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        self.update_checked(events);

        let data = self.get_data_mut();
        let pos = Screen::convert_from_pixels_into_screen_space(data.state.get_position());
        let size = Screen::convert_size_from_pixels(data.state.get_size());
        let mut mesh_data = MeshData::default();
        mesh_data
            .add_quad_default([0.0, 0.0, size.x, size.y].into(), data.state.get_layer())
            .set_vertex_color(data.graphics.get_color());
        mesh_data.translate([pos.x, pos.y, 0.0].into());
        data.graphics.set_mesh_data(mesh_data);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
