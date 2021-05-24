use std::any::TypeId;

use nrg_events::{EventsHistory, EventsHistoryOperation};
use nrg_gui::*;
use nrg_math::Vector4;
use nrg_messenger::Message;
use nrg_platform::{InputState, Key, KeyEvent};
use nrg_serialize::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct HistoryPanel {
    data: WidgetData,
    #[serde(skip)]
    history: EventsHistory,
    #[serde(skip)]
    history_events_box_id: Uid,
    #[serde(skip)]
    history_text_widget_id: Uid,
    #[serde(skip)]
    history_redo_button: Uid,
    #[serde(skip)]
    history_undo_button: Uid,
    #[serde(skip)]
    history_clear_button: Uid,
    #[serde(skip)]
    is_ctrl_pressed: bool,
}
implement_widget_with_custom_members!(HistoryPanel {
    history: EventsHistory::default(),
    history_events_box_id: INVALID_UID,
    history_text_widget_id: INVALID_UID,
    history_redo_button: INVALID_UID,
    history_undo_button: INVALID_UID,
    history_clear_button: INVALID_UID,
    is_ctrl_pressed: false
});

impl HistoryPanel {
    pub fn get_history(&mut self) -> &mut EventsHistory {
        &mut self.history
    }
    fn create_history_widget(&mut self) -> (Uid, Uid, Uid, Uid, Uid) {
        let mut label = Text::new(self.get_shared_data(), self.get_global_messenger());
        label.set_text("Event History:");
        self.add_child(Box::new(label));

        let mut button_box = Panel::new(self.get_shared_data(), self.get_global_messenger());
        button_box
            .fill_type(ContainerFillType::Horizontal)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .keep_fixed_height(false)
            .size([DEFAULT_BUTTON_SIZE[0] * 4., DEFAULT_BUTTON_SIZE[1] * 2.].into())
            .space_between_elements((DEFAULT_WIDGET_WIDTH * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(true);

        let mut history_undo = Button::new(self.get_shared_data(), self.get_global_messenger());
        history_undo.with_text("Undo");

        let mut history_redo = Button::new(self.get_shared_data(), self.get_global_messenger());
        history_redo.with_text("Redo");

        let mut history_clear = Button::new(self.get_shared_data(), self.get_global_messenger());
        history_clear.with_text("Clear");

        let history_undo_button_id = button_box.add_child(Box::new(history_undo));
        let history_redo_button_id = button_box.add_child(Box::new(history_redo));
        let history_clear_button_id = button_box.add_child(Box::new(history_clear));

        self.add_child(Box::new(button_box));

        let separator = Separator::new(self.get_shared_data(), self.get_global_messenger());
        self.add_child(Box::new(separator));

        let mut history_events_box =
            Panel::new(self.get_shared_data(), self.get_global_messenger());
        history_events_box
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements((2. * Screen::get_scale_factor()) as u32)
            .style(WidgetStyle::Invisible);

        let mut text = Text::new(self.get_shared_data(), self.get_global_messenger());
        text.set_text("Prova1\nProva2 \nProva3");

        let history_text_id = history_events_box.add_child(Box::new(text));

        let history_events_box_id = self.add_child(Box::new(history_events_box));

        (
            history_text_id,
            history_events_box_id,
            history_undo_button_id,
            history_redo_button_id,
            history_clear_button_id,
        )
    }

    fn update_history_widget(&self) -> String {
        let mut text = String::new();

        if let Some(history_debug_events) = self.history.get_undoable_events_history_as_string() {
            for str in history_debug_events.iter() {
                text += str;
                text += "\n";
            }
        }
        if let Some(history_debug_events) = self.history.get_redoable_events_history_as_string() {
            for str in history_debug_events.iter() {
                text += str;
                text += "\n";
            }
        }
        text
    }
}

impl InternalWidget for HistoryPanel {
    fn widget_init(&mut self) {
        self.get_global_messenger()
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(self.get_messagebox());

        self.size([450., 1000.].into())
            .vertical_alignment(VerticalAlignment::Bottom)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements((DEFAULT_WIDGET_WIDTH / 2. * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(true)
            .style(WidgetStyle::DefaultBackground)
            .move_to_layer(0.5);

        let (
            history_text_id,
            history_events_box_id,
            history_undo_button_id,
            history_redo_button_id,
            history_clear_button_id,
        ) = self.create_history_widget();
        self.history_text_widget_id = history_text_id;
        self.history_events_box_id = history_events_box_id;
        self.history_undo_button = history_undo_button_id;
        self.history_redo_button = history_redo_button_id;
        self.history_clear_button = history_clear_button_id;
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {
        self.history.update();

        if self.graphics().is_visible() {
            let widget_id = self.history_text_widget_id;
            let text = self.update_history_widget();
            if let Some(history_text) = self.node_mut().get_child::<Text>(widget_id) {
                history_text.set_text(text.as_str());
            }
        }
    }

    fn widget_uninit(&mut self) {}

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = *event {
                if widget_id == self.history_redo_button {
                    self.history.push(EventsHistoryOperation::Redo);
                } else if widget_id == self.history_undo_button {
                    self.history.push(EventsHistoryOperation::Undo);
                } else if widget_id == self.history_clear_button {
                    self.history.push(EventsHistoryOperation::Clear);
                }
            }
        } else if msg.type_id() == TypeId::of::<KeyEvent>() {
            let event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
            if event.code == Key::Control {
                if event.state == InputState::Pressed || event.state == InputState::JustPressed {
                    self.is_ctrl_pressed = true;
                } else if event.state == InputState::Released
                    || event.state == InputState::JustReleased
                {
                    self.is_ctrl_pressed = false;
                }
            } else if self.is_ctrl_pressed
                && event.code == Key::Z
                && event.state == InputState::JustPressed
            {
                self.history.push(EventsHistoryOperation::Undo);
            } else if self.is_ctrl_pressed
                && event.code == Key::Y
                && event.state == InputState::JustPressed
            {
                self.history.push(EventsHistoryOperation::Redo);
            }
        }
    }
}
