use nrg_graphics::Renderer;
use nrg_gui::{
    BaseWidget, Button, ContainerFillType, EditableText, HorizontalAlignment, Panel, Screen, Text,
    VerticalAlignment, WidgetDataGetter, WidgetEvent, WidgetStyle, DEFAULT_BUTTON_SIZE,
};
use nrg_math::Vector2;
use nrg_platform::EventsRw;
use nrg_serialize::{Uid, INVALID_UID};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    Waiting,
    Ok,
    Cancel,
}

pub struct FilenameDialog {
    dialog: Panel,
    title_box_uid: Uid,
    title_uid: Uid,
    content_box_uid: Uid,
    label_uid: Uid,
    editable_text_uid: Uid,
    button_box_uid: Uid,
    ok_uid: Uid,
    cancel_uid: Uid,
    result: DialogResult,
}

impl Default for FilenameDialog {
    fn default() -> Self {
        Self {
            dialog: Panel::default(),
            title_box_uid: INVALID_UID,
            title_uid: INVALID_UID,
            content_box_uid: INVALID_UID,
            label_uid: INVALID_UID,
            editable_text_uid: INVALID_UID,
            button_box_uid: INVALID_UID,
            ok_uid: INVALID_UID,
            cancel_uid: INVALID_UID,
            result: DialogResult::Waiting,
        }
    }
}

impl FilenameDialog {
    pub fn get_filename(&mut self) -> String {
        let mut filename = String::new();
        let uid = self.editable_text_uid;
        if let Some(editable_text) = self
            .dialog
            .get_data_mut()
            .node
            .get_child::<EditableText>(uid)
        {
            filename = editable_text.get_text();
        }
        filename
    }
    pub fn get_result(&self) -> DialogResult {
        self.result
    }
    fn add_title(&mut self, renderer: &mut Renderer) {
        let mut title_box = Panel::default();
        title_box.init(renderer);
        title_box
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Vertical)
            .space_between_elements(10)
            .use_space_before_and_after(true);

        let mut title = Text::default();
        title.init(renderer);
        title
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Center)
            .set_text("New file");

        self.title_uid = title_box.add_child(Box::new(title));
        self.title_box_uid = self.dialog.add_child(Box::new(title_box));
    }
    fn add_content(&mut self, renderer: &mut Renderer) {
        let mut content_box = Panel::default();
        content_box.init(renderer);
        content_box
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_height(false)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Stretch);

        let mut label = Text::default();
        label.init(renderer);
        label
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Left)
            .set_text("Filename: ");

        let mut editable_text = EditableText::default();
        editable_text.init(renderer);
        editable_text.vertical_alignment(VerticalAlignment::Center);

        self.label_uid = content_box.add_child(Box::new(label));
        self.editable_text_uid = content_box.add_child(Box::new(editable_text));
        self.content_box_uid = self.dialog.add_child(Box::new(content_box));
    }

    fn add_buttons(&mut self, renderer: &mut Renderer) {
        let mut button_box = Panel::default();
        button_box.init(renderer);

        let default_size: Vector2 = DEFAULT_BUTTON_SIZE.into();
        button_box
            .size(default_size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .horizontal_alignment(HorizontalAlignment::Right)
            .keep_fixed_height(true)
            .space_between_elements(40);

        let mut button_ok = Button::default();
        button_ok.init(renderer);
        button_ok
            .with_text("Ok")
            .horizontal_alignment(HorizontalAlignment::Left);

        let mut button_cancel = Button::default();
        button_cancel.init(renderer);
        button_cancel
            .with_text("Cancel")
            .horizontal_alignment(HorizontalAlignment::Right);

        self.ok_uid = button_box.add_child(Box::new(button_ok));
        self.cancel_uid = button_box.add_child(Box::new(button_cancel));
        self.button_box_uid = self.dialog.add_child(Box::new(button_box));
    }
    fn manage_events(&mut self, events_rw: &mut EventsRw) {
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Pressed(widget_id) = event {
                    if self.ok_uid == *widget_id {
                        self.result = DialogResult::Ok;
                    } else if self.cancel_uid == *widget_id {
                        self.result = DialogResult::Cancel;
                    }
                }
            }
        }
    }

    pub fn init(&mut self, renderer: &mut Renderer) {
        self.dialog.init(renderer);
        self.dialog
            .size([800., 200.].into())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(true)
            .selectable(false)
            .style(WidgetStyle::DefaultBackground)
            .move_to_layer(1.);

        self.add_title(renderer);
        self.add_content(renderer);
        self.add_buttons(renderer);
    }

    pub fn update(&mut self, renderer: &mut Renderer, events_rw: &mut EventsRw) {
        self.manage_events(events_rw);
        self.dialog
            .update(Screen::get_draw_area(), renderer, events_rw);
    }

    pub fn uninit(&mut self, renderer: &mut Renderer) {
        self.dialog.uninit(renderer);
    }
}
