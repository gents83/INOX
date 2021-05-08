use nrg_events::{implement_undoable_event, Event, EventsRw};
use nrg_math::Vector2;
use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget, InternalWidget, Panel, Text, WidgetData, WidgetEvent, DEFAULT_WIDGET_HEIGHT,
    DEFAULT_WIDGET_SIZE, DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_CHECKBOX_SIZE: [f32; 2] = [
    DEFAULT_WIDGET_WIDTH / 2. * 3.,
    DEFAULT_WIDGET_HEIGHT / 2. * 3.,
];

#[derive(Clone, Copy)]
pub enum CheckboxEvent {
    Checked(Uid),
    Unchecked(Uid),
}
implement_undoable_event!(CheckboxEvent, undo_event, debug_info_event);
fn undo_event(event: &CheckboxEvent) -> CheckboxEvent {
    match event {
        CheckboxEvent::Checked(widget_id) => CheckboxEvent::Unchecked(*widget_id),
        CheckboxEvent::Unchecked(widget_id) => CheckboxEvent::Checked(*widget_id),
    }
}
fn debug_info_event(event: &CheckboxEvent) -> String {
    match event {
        CheckboxEvent::Checked(_widget_id) => String::from("Checked"),
        CheckboxEvent::Unchecked(_widget_id) => String::from("Unchecked"),
    }
}

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
    pub fn with_label(&mut self, shared_data: &SharedDataRw, text: &str) -> &mut Self {
        if !self.label_widget.is_nil() {
            let uid = self.label_widget;
            self.get_data_mut().node.remove_child(uid);
            self.label_widget = INVALID_UID;
        }
        let mut label = Text::default();
        label.init(shared_data);
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

    fn check_state_change(&self, id: Uid, events_rw: &mut EventsRw) {
        let mut changed = false;
        let mut new_state = false;
        {
            let events = events_rw.read().unwrap();
            if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
                for event in widget_events.iter() {
                    if let WidgetEvent::Released(widget_id, _mouse_in_px) = event {
                        if id == *widget_id {
                            if !self.is_checked {
                                changed = true;
                                new_state = true;
                            } else if self.is_checked {
                                changed = true;
                                new_state = false;
                            }
                        }
                    }
                }
            }
        }
        {
            let mut events = events_rw.write().unwrap();
            if changed {
                if new_state {
                    events.send_event(CheckboxEvent::Checked(id));
                } else {
                    events.send_event(CheckboxEvent::Unchecked(id));
                }
            }
        }
    }

    pub fn update_checked(&mut self, shared_data: &SharedDataRw) {
        let id = self.outer_widget;
        let read_data = shared_data.read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        self.check_state_change(id, events_rw);

        let events = events_rw.read().unwrap();
        if let Some(checkbox_events) = events.read_all_events::<CheckboxEvent>() {
            for event in checkbox_events.iter() {
                if let CheckboxEvent::Checked(widget_id) = event {
                    if *widget_id == id {
                        self.checked(true);
                    }
                } else if let CheckboxEvent::Unchecked(widget_id) = event {
                    if *widget_id == id {
                        self.checked(false);
                    }
                }
            }
        }
    }
}

impl InternalWidget for Checkbox {
    fn widget_init(&mut self, shared_data: &SharedDataRw) {
        if self.is_initialized() {
            return;
        }

        let default_size: Vector2 = DEFAULT_CHECKBOX_SIZE.into();
        self.size(default_size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements((DEFAULT_WIDGET_SIZE[0] / 2. * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(false)
            .keep_fixed_height(false)
            .style(WidgetStyle::Invisible);

        let mut outer_widget = Panel::default();
        outer_widget.init(shared_data);
        outer_widget
            .size(default_size)
            .selectable(true)
            .vertical_alignment(VerticalAlignment::Center)
            .style(WidgetStyle::Default);

        let inner_size = default_size / 4. * 3.;
        let mut inner_check = Panel::default();
        inner_check.init(shared_data);
        inner_check
            .size(inner_size)
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .style(WidgetStyle::Invisible);

        self.checked_widget = outer_widget.add_child(Box::new(inner_check));
        self.outer_widget = self.add_child(Box::new(outer_widget));
    }

    fn widget_update(&mut self, shared_data: &SharedDataRw) {
        self.update_checked(shared_data);
    }

    fn widget_uninit(&mut self, _shared_data: &SharedDataRw) {}
}
