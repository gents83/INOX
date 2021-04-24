use nrg_commands::*;
use nrg_graphics::*;
use nrg_gui::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub struct HistoryPanel {
    history_panel: Panel,
    history_text_widget_id: Uid,
    history_redo_button: Uid,
    history_undo_button: Uid,
    history_clear_button: Uid,
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
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.history_panel.visible(visible);
        self
    }
    fn create_history_widget(&mut self, renderer: &mut Renderer) -> (Uid, Uid, Uid, Uid) {
        let mut label = Text::default();
        label.init(renderer);
        label
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Left)
            .set_text("Command History:");
        self.history_panel.add_child(Box::new(label));

        let mut button_box = Panel::default();
        button_box.init(renderer);
        button_box
            .fill_type(ContainerFillType::Horizontal)
            .horizontal_alignment(HorizontalAlignment::Center)
            .size([DEFAULT_BUTTON_SIZE[0] * 4., DEFAULT_BUTTON_SIZE[1] * 2.].into())
            .space_between_elements(
                (DEFAULT_WIDGET_SIZE[0] * 4. * Screen::get_scale_factor()) as _,
            );

        let mut history_undo = Button::default();
        history_undo.init(renderer);
        history_undo.with_text("Undo");

        let mut history_redo = Button::default();
        history_redo.init(renderer);
        history_redo.with_text("Redo");

        let mut history_clear = Button::default();
        history_clear.init(renderer);
        history_clear.with_text("Clear");

        let history_undo_button_id = button_box.add_child(Box::new(history_undo));
        let history_redo_button_id = button_box.add_child(Box::new(history_redo));
        let history_clear_button_id = button_box.add_child(Box::new(history_clear));

        self.history_panel.add_child(Box::new(button_box));

        let mut separator = Separator::default();
        separator.init(renderer);
        self.history_panel.add_child(Box::new(separator));

        let mut history_commands_box = Panel::default();
        history_commands_box.init(renderer);
        history_commands_box
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements(2)
            .style(WidgetStyle::Invisible);

        let history_text_id = self.history_panel.add_child(Box::new(history_commands_box));

        let mut separator = Separator::default();
        separator.init(renderer);
        self.history_panel.add_child(Box::new(separator));

        (
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
                            0.,
                            20. * history_commands_box.get_data_mut().node.get_num_children()
                                as f32,
                        ]
                        .into(),
                    )
                    .set_text(str);
                    if index >= history_debug_commands.len() - 1 {
                        text.style(WidgetStyle::FullHighlight)
                            .border_style(WidgetStyle::FullHighlight);
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
                            0.,
                            20. * history_commands_box.get_data_mut().node.get_num_children()
                                as f32,
                        ]
                        .into(),
                    )
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
            .size([650., 100.].into())
            .vertical_alignment(VerticalAlignment::Bottom)
            .horizontal_alignment(HorizontalAlignment::Left)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements(5)
            .move_to_layer(0.5);

        let (
            history_text_id,
            history_undo_button_id,
            history_redo_button_id,
            history_clear_button_id,
        ) = self.create_history_widget(renderer);
        self.history_text_widget_id = history_text_id;
        self.history_undo_button = history_undo_button_id;
        self.history_redo_button = history_redo_button_id;
        self.history_clear_button = history_clear_button_id;
    }

    pub fn update(
        &mut self,
        drawing_area_in_px: Vector4,
        renderer: &mut Renderer,
        events: &mut EventsRw,
        history: &mut CommandsHistory,
    ) {
        self.update_history_widget(renderer, &history);

        self.history_panel
            .update(drawing_area_in_px, renderer, events);

        self.manage_history_interactions(events, history);
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) {
        self.history_panel.uninit(renderer);
    }
}
