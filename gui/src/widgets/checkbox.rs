use std::any::TypeId;

use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_undoable_message, Message};
use nrg_platform::MouseEvent;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, Screen, Text, WidgetData,
    WidgetEvent, DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_SIZE, DEFAULT_WIDGET_WIDTH,
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
implement_undoable_message!(CheckboxEvent, undo_event, debug_info_event);
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
    outer_widget: Uid,
    checked_widget: Uid,
    label_widget: Uid,
}
implement_widget_with_custom_members!(Checkbox {
    is_checked: false,
    outer_widget: INVALID_UID,
    checked_widget: INVALID_UID,
    label_widget: INVALID_UID
});

impl Checkbox {
    pub fn with_label(&mut self, text: &str) -> &mut Self {
        if !self.label_widget.is_nil() {
            let uid = self.label_widget;
            self.node_mut().remove_child(uid);
            self.label_widget = INVALID_UID;
        }
        let mut label = Text::new(self.get_shared_data(), self.get_global_messenger());
        label
            .vertical_alignment(VerticalAlignment::Center)
            .set_text(text);
        self.label_widget = self.add_child(Box::new(label));
        self
    }
    pub fn checked(&mut self, checked: bool) -> &mut Self {
        if self.is_checked != checked {
            self.is_checked = checked;
            let checked_id = self.checked_widget;
            if let Some(inner_widget) = self.node().get_child_mut::<Panel>(checked_id) {
                if checked {
                    inner_widget.style(WidgetStyle::DefaultText);
                } else {
                    inner_widget.style(WidgetStyle::Invisible);
                }
            }
        }
        self
    }

    pub fn is_checked(&self) -> bool {
        self.is_checked
    }
}

impl InternalWidget for Checkbox {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<CheckboxEvent>()
            .register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

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

        let mut outer_widget = Panel::new(self.get_shared_data(), self.get_global_messenger());
        outer_widget
            .size(default_size)
            .selectable(true)
            .vertical_alignment(VerticalAlignment::Center)
            .style(WidgetStyle::Default);

        let inner_size = default_size / 4. * 3.;
        let mut inner_check = Panel::new(self.get_shared_data(), self.get_global_messenger());
        inner_check
            .size(inner_size)
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .style(WidgetStyle::Invisible);

        self.checked_widget = outer_widget.add_child(Box::new(inner_check));
        self.outer_widget = self.add_child(Box::new(outer_widget));
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<CheckboxEvent>()
            .unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Released(widget_id, _mouse_in_px) = *event {
                if self.outer_widget == widget_id {
                    let checkbox_event = if !self.is_checked {
                        CheckboxEvent::Checked(self.outer_widget)
                    } else {
                        CheckboxEvent::Unchecked(self.outer_widget)
                    };

                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(checkbox_event.as_boxed())
                        .ok();
                }
            }
        } else if msg.type_id() == TypeId::of::<CheckboxEvent>() {
            let event = msg.as_any().downcast_ref::<CheckboxEvent>().unwrap();
            if let CheckboxEvent::Checked(widget_id) = *event {
                if widget_id == self.outer_widget {
                    self.checked(true);
                }
            } else if let CheckboxEvent::Unchecked(widget_id) = *event {
                if widget_id == self.outer_widget {
                    self.checked(false);
                }
            }
        }
    }
    fn widget_on_layout_changed(&mut self) {}
}
