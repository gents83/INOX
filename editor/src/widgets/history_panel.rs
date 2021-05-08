use nrg_events::*;
use nrg_gui::*;
use nrg_math::*;
use nrg_resources::SharedDataRw;
use nrg_serialize::*;

pub struct HistoryPanel {
    history_panel: Panel,
    history_events_box_id: Uid,
    history_text_widget_id: Uid,
    history_redo_button: Uid,
    history_undo_button: Uid,
    history_clear_button: Uid,
}

impl Default for HistoryPanel {
    fn default() -> Self {
        Self {
            history_panel: Panel::default(),
            history_events_box_id: INVALID_UID,
            history_text_widget_id: INVALID_UID,
            history_redo_button: INVALID_UID,
            history_undo_button: INVALID_UID,
            history_clear_button: INVALID_UID,
        }
    }
}

impl HistoryPanel {
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.history_panel.visible(visible);
        self
    }
    fn create_history_widget(&mut self, shared_data: &SharedDataRw) -> (Uid, Uid, Uid, Uid, Uid) {
        let mut label = Text::default();
        label.init(shared_data);
        label.set_text("Event History:");
        self.history_panel.add_child(Box::new(label));

        let mut button_box = Panel::default();
        button_box.init(shared_data);
        button_box
            .fill_type(ContainerFillType::Horizontal)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .keep_fixed_height(false)
            .size([DEFAULT_BUTTON_SIZE[0] * 4., DEFAULT_BUTTON_SIZE[1] * 2.].into())
            .space_between_elements((DEFAULT_WIDGET_WIDTH * Screen::get_scale_factor()) as _)
            .use_space_before_and_after(true);

        let mut history_undo = Button::default();
        history_undo.init(shared_data);
        history_undo.with_text("Undo");

        let mut history_redo = Button::default();
        history_redo.init(shared_data);
        history_redo.with_text("Redo");

        let mut history_clear = Button::default();
        history_clear.init(shared_data);
        history_clear.with_text("Clear");

        let history_undo_button_id = button_box.add_child(Box::new(history_undo));
        let history_redo_button_id = button_box.add_child(Box::new(history_redo));
        let history_clear_button_id = button_box.add_child(Box::new(history_clear));

        self.history_panel.add_child(Box::new(button_box));

        let mut separator = Separator::default();
        separator.init(shared_data);
        self.history_panel.add_child(Box::new(separator));

        let mut history_events_box = Panel::default();
        history_events_box.init(shared_data);
        history_events_box
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements((2. * Screen::get_scale_factor()) as u32)
            .style(WidgetStyle::Invisible);

        let mut text = Text::default();
        text.init(shared_data);
        text.set_text("Prova1\nProva2 \nProva3");

        let history_text_id = history_events_box.add_child(Box::new(text));

        let history_events_box_id = self.history_panel.add_child(Box::new(history_events_box));

        (
            history_text_id,
            history_events_box_id,
            history_undo_button_id,
            history_redo_button_id,
            history_clear_button_id,
        )
    }

    fn update_history_widget(&mut self, history: &EventsHistory) -> &mut Self {
        if let Some(history_text) = self
            .history_panel
            .get_data_mut()
            .node
            .get_child::<Text>(self.history_text_widget_id)
        {
            let mut text = String::new();

            if let Some(history_debug_events) = history.get_undoable_events_history_as_string() {
                for str in history_debug_events.iter() {
                    text += str;
                    text += "\n";
                }
            }
            if let Some(history_debug_events) = history.get_redoable_events_history_as_string() {
                for str in history_debug_events.iter() {
                    text += str;
                    text += "\n";
                }
            }
            history_text.set_text(text.as_str());
        }
        self
    }
    fn manage_history_interactions(
        &mut self,
        shared_data: &SharedDataRw,
        history: &mut EventsHistory,
    ) -> &mut Self {
        let read_data = shared_data.read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        let events = events_rw.read().unwrap();
        if let Some(button_events) = events.read_all_events::<WidgetEvent>() {
            for event in button_events.iter() {
                if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
                    if *widget_id == self.history_redo_button {
                        history.redo_last_event();
                    } else if *widget_id == self.history_undo_button {
                        history.undo_last_event();
                    } else if *widget_id == self.history_clear_button {
                        history.clear();
                    }
                }
            }
        }
        self
    }

    pub fn init(&mut self, shared_data: &SharedDataRw) {
        self.history_panel.init(shared_data);
        self.history_panel
            .size([450., 1000.].into())
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
        ) = self.create_history_widget(shared_data);
        self.history_text_widget_id = history_text_id;
        self.history_events_box_id = history_events_box_id;
        self.history_undo_button = history_undo_button_id;
        self.history_redo_button = history_redo_button_id;
        self.history_clear_button = history_clear_button_id;
    }

    pub fn update(
        &mut self,
        drawing_area_in_px: Vector4,
        shared_data: &SharedDataRw,
        history: &mut EventsHistory,
    ) {
        if self.history_panel.get_data().graphics.is_visible() {
            self.update_history_widget(&history);
            self.manage_history_interactions(shared_data, history);
        }
        self.history_panel.update(drawing_area_in_px, shared_data);
    }

    pub fn uninit(&mut self, shared_data: &SharedDataRw) {
        self.history_panel.uninit(shared_data);
    }
}
