use nrg_commands::*;
use nrg_graphics::*;
use nrg_gui::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct HistoryPanel {
    history_panel: Panel,
    history_text_widget_id: UID,
    history_redo_button: UID,
    history_undo_button: UID,
    history_clear_button: UID,
}

impl Default for HistoryPanel {
    fn default() -> Self {
        Self {
            history_panel: Panel::default(),
            history_text_widget_id: INVALID_ID,
            history_redo_button: INVALID_ID,
            history_undo_button: INVALID_ID,
            history_clear_button: INVALID_ID,
        }
    }
}

impl HistoryPanel {
    fn create_history_widget(&mut self, renderer: &mut Renderer) -> (Panel, UID, UID, UID, UID) {
        let mut history_panel = Panel::default();
        history_panel.init(renderer);
        history_panel
            .size([500, 100].into())
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .selectable(false)
            .draggable(false)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements(5);

        let mut label = Text::default();
        label.init(renderer);
        label
            .size([0, 16].into())
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Left)
            .set_text("Command History:");
        history_panel.add_child(Box::new(label));

        let mut button_box = Panel::default();
        button_box.init(renderer);
        button_box
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .selectable(false)
            .draggable(false)
            .fit_to_content(true)
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements(25);

        let mut history_undo = Button::default();
        history_undo.init(renderer);
        history_undo.size([150, 100].into()).stroke(10);
        let mut text = Text::default();
        text.init(renderer);
        text.size([0, 20].into())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("Undo");
        history_undo.add_child(Box::new(text));

        let mut history_redo = Button::default();
        history_redo.init(renderer);
        history_redo.size([150, 100].into()).stroke(10);
        let mut text = Text::default();
        text.init(renderer);
        text.size([0, 20].into())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("Redo");
        history_redo.add_child(Box::new(text));

        let mut history_clear = Button::default();
        history_clear.init(renderer);
        history_clear.size([150, 100].into()).stroke(10);
        let mut text = Text::default();
        text.init(renderer);
        text.size([0, 20].into())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("Clear");
        history_clear.add_child(Box::new(text));

        let history_undo_button_id = button_box.add_child(Box::new(history_undo));
        let history_redo_button_id = button_box.add_child(Box::new(history_redo));
        let history_clear_button_id = button_box.add_child(Box::new(history_clear));

        history_panel.add_child(Box::new(button_box));

        let mut separator = Separator::default();
        separator.init(renderer);
        history_panel.add_child(Box::new(separator));

        let mut history_commands_box = Panel::default();
        history_commands_box.init(renderer);
        history_commands_box
            .size([300, 20].into())
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .selectable(false)
            .draggable(false)
            .fit_to_content(true)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements(10);

        let history_text_id = history_panel.add_child(Box::new(history_commands_box));

        let mut separator = Separator::default();
        separator.init(renderer);
        history_panel.add_child(Box::new(separator));

        (
            history_panel,
            history_text_id,
            history_undo_button_id,
            history_redo_button_id,
            history_clear_button_id,
        )
    }

    fn update_history_widget(
        &mut self,
        renderer: &mut Renderer,
        history: &CommandsHistory,
    ) -> &mut Self {
        if let Some(history_commands_box) = self
            .history_panel
            .get_data_mut()
            .node
            .get_child::<Panel>(self.history_text_widget_id)
        {
            history_commands_box.remove_children(renderer);
            if let Some(history_debug_commands) = history.get_undoable_commands_history_as_string()
            {
                for (index, str) in history_debug_commands.iter().enumerate() {
                    let mut text = Text::default();
                    text.init(renderer);
                    text.position(
                        [
                            0,
                            20 * history_commands_box.get_data_mut().node.get_num_children() as u32,
                        ]
                        .into(),
                    )
                    .size([300, 20].into())
                    .set_text(str);
                    if index >= history_debug_commands.len() - 1 {
                        text.get_data_mut()
                            .graphics
                            .set_style(WidgetStyle::FullHighlight)
                            .set_border_style(WidgetStyle::FullHighlight);
                        let mut string = String::from("-> ");
                        string.push_str(str);
                        text.set_text(string.as_str());
                    }
                    history_commands_box.add_child(Box::new(text));
                }
            }
            if let Some(history_debug_commands) = history.get_redoable_commands_history_as_string()
            {
                for str in history_debug_commands.iter().rev() {
                    let mut text = Text::default();
                    text.init(renderer);
                    text.position(
                        [
                            0,
                            20 * history_commands_box.get_data_mut().node.get_num_children() as u32,
                        ]
                        .into(),
                    )
                    .size([300, 20].into())
                    .set_text(str);
                    history_commands_box.add_child(Box::new(text));
                }
            }
        }
        self
    }
    fn manage_history_interactions(
        &mut self,
        events_rw: &mut EventsRw,
        history: &mut CommandsHistory,
    ) -> &mut Self {
        let events = events_rw.read().unwrap();
        if let Some(button_events) = events.read_events::<WidgetEvent>() {
            for event in button_events.iter() {
                if let WidgetEvent::Pressed(widget_id) = event {
                    if *widget_id == self.history_redo_button {
                        history.redo_last_command();
                    } else if *widget_id == self.history_undo_button {
                        history.undo_last_command();
                    } else if *widget_id == self.history_clear_button {
                        history.clear();
                    }
                }
            }
        }
        self
    }

    pub fn init(&mut self, renderer: &mut Renderer) {
        self.history_panel.init(renderer);
        self.history_panel
            .size([600, 800].into())
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Left)
            .fill_type(ContainerFillType::Vertical)
            .fit_to_content(true);

        let (
            history_panel,
            history_text_id,
            history_undo_button_id,
            history_redo_button_id,
            history_clear_button_id,
        ) = self.create_history_widget(renderer);
        self.history_panel.add_child(Box::new(history_panel));
        self.history_text_widget_id = history_text_id;
        self.history_undo_button = history_undo_button_id;
        self.history_redo_button = history_redo_button_id;
        self.history_clear_button = history_clear_button_id;
    }

    pub fn update(
        &mut self,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        input_handler: &InputHandler,
        history: &mut CommandsHistory,
    ) {
        self.update_history_widget(renderer, &history);

        self.history_panel
            .update(Screen::get_draw_area(), renderer, events, input_handler);

        self.manage_history_interactions(events, history);
    }
}
