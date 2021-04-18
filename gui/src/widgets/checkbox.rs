use nrg_graphics::Renderer;
use nrg_math::{const_vec2, Vector2};
use nrg_platform::{Event, EventsRw};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget, InternalWidget, Panel, Text, WidgetData, WidgetEvent, DEFAULT_WIDGET_HEIGHT,
    DEFAULT_WIDGET_SIZE,
};

pub const DEFAULT_CHECKBOX_SIZE: Vector2 =
    const_vec2!([DEFAULT_WIDGET_HEIGHT as _, DEFAULT_WIDGET_HEIGHT as _]);

pub enum CheckboxEvent {
    Checked(Uid),
    Unchecked(Uid),
}
impl Event for CheckboxEvent {}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Checkbox {
    data: WidgetData,
    is_checked: bool,
    #[serde(skip)]
    outer_widget: Uid,
    #[serde(skip)]
    checked_widget: Uid,
    #[serde(skip)]
    label_widget: Uid,
}
implement_widget!(Checkbox);

impl Default for Checkbox {
    fn default() -> Self {
        Self {
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
            .set_text(text);
        self.label_widget = self.add_child(Box::new(label));
        self
    }
    pub fn checked(&mut self, checked: bool) -> &mut Self {
        self.is_checked = checked;
        let checked_id = self.checked_widget;
        if let Some(inner_widget) = self.get_data_mut().node.get_child::<Panel>(checked_id) {
            if checked {
                inner_widget.style(WidgetStyle::FullActive);
            } else {
                inner_widget.style(WidgetStyle::Invisible);
            }
        }
        self
    }

    pub fn is_checked(&self) -> bool {
        self.is_checked
    }

    fn check_state_change(id: Uid, old_state: bool, events_rw: &mut EventsRw) -> (bool, bool) {
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
        let id = self.outer_widget;
        let (changed, new_state) = Self::check_state_change(id, self.is_checked, events_rw);

        let mut events = events_rw.write().unwrap();
        if changed {
            if new_state {
                events.send_event(CheckboxEvent::Checked(id));
            } else {
                events.send_event(CheckboxEvent::Unchecked(id));
            }
            self.checked(new_state);
        }
    }
}

impl InternalWidget for Checkbox {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        let default_size = DEFAULT_CHECKBOX_SIZE * Screen::get_scale_factor();
        self.size(default_size)
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements((DEFAULT_WIDGET_SIZE.x / 2. * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(false)
            .style(WidgetStyle::Invisible);

        let mut outer_widget = Panel::default();
        outer_widget.init(renderer);
        outer_widget
            .size(default_size)
            .selectable(true)
            .style(WidgetStyle::Default);

        let inner_size = default_size / 4. * 3.;
        let mut inner_check = Panel::default();
        inner_check.init(renderer);
        inner_check
            .size(inner_size)
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .style(WidgetStyle::Invisible);

        self.checked_widget = outer_widget.add_child(Box::new(inner_check));
        self.outer_widget = self.add_child(Box::new(outer_widget));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, events: &mut EventsRw) {
        self.update_checked(events);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
