use nrg_graphics::{MeshData, Renderer};
use nrg_math::Vector4u;
use nrg_platform::{Event, EventsRw, InputHandler};
use nrg_serialize::{Deserialize, Serialize, INVALID_UID, UID};

use crate::{
    implement_container, implement_widget, ContainerData, ContainerFillType, InternalWidget, Panel,
    Text, WidgetData, WidgetEvent, DEFAULT_WIDGET_SIZE,
};

pub enum CheckboxEvent {
    Checked(UID),
    Unchecked(UID),
}
impl Event for CheckboxEvent {}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Checkbox {
    #[serde(skip)]
    container: ContainerData,
    data: WidgetData,
    is_checked: bool,
    #[serde(skip)]
    outer_widget: UID,
    #[serde(skip)]
    checked_widget: UID,
    #[serde(skip)]
    label_widget: UID,
}
implement_widget!(Checkbox);
implement_container!(Checkbox);

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            container: ContainerData::default(),
            data: WidgetData::default(),
            is_checked: false,
            outer_widget: INVALID_UID,
            checked_widget: INVALID_UID,
            label_widget: INVALID_UID,
        }
    }
}

impl Checkbox {
    pub fn with_label(&mut self, renderer: &mut Renderer, text: &str) -> &mut Self {
        if !self.label_widget.is_nil() {
            let uid = self.label_widget;
            self.get_data_mut().node.remove_child(uid);
            self.label_widget = INVALID_UID;
        }
        let mut label = Text::default();
        label.init(renderer);
        label
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text(text);
        self.label_widget = self.add_child(Box::new(label));
        self
    }
    pub fn checked(&mut self, checked: bool) -> &mut Self {
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
                    inner_widget.style(WidgetStyle::FullActive);

                    events.send_event(CheckboxEvent::Checked(id));
                } else {
                    inner_widget.style(WidgetStyle::FullInactive);

                    events.send_event(CheckboxEvent::Unchecked(id));
                }
                self.checked(new_state);
            }
        }
    }
}

impl InternalWidget for Checkbox {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().graphics.init(renderer, "UI");
        if self.is_initialized() {
            return;
        }

        let default_size = DEFAULT_WIDGET_SIZE * Screen::get_scale_factor();
        self.size(default_size)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements(40)
            .use_space_before_and_after(false)
            .style(WidgetStyle::Invisible);

        let mut outer_widget = Panel::default();
        outer_widget
            .size(default_size)
            .stroke(4)
            .style(WidgetStyle::DefaultBackground);

        let inner_size = default_size - default_size / 4;
        let mut inner_check = Panel::default();
        inner_check.init(renderer);
        inner_check
            .size(inner_size)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .fit_to_content(false)
            .style(WidgetStyle::FullInactive);

        self.checked_widget = outer_widget.add_child(Box::new(inner_check));
        self.outer_widget = self.add_child(Box::new(outer_widget));
    }

    fn widget_update(
        &mut self,
        _drawing_area_in_px: Vector4u,
        _renderer: &mut Renderer,
        events: &mut EventsRw,
        _input_handler: &InputHandler,
    ) {
        self.apply_fit_to_content();
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
