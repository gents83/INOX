use nrg_events::EventsRw;
use nrg_graphics::Renderer;
use nrg_gui::{
    BaseWidget, Button, ContainerFillType, HorizontalAlignment, Panel, Screen, TextBox, TitleBar,
    VerticalAlignment, WidgetDataGetter, WidgetEvent, WidgetStyle, DEFAULT_BUTTON_SIZE,
};
use nrg_math::Vector2;
use nrg_serialize::{Uid, INVALID_UID};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    Waiting,
    Ok,
    Cancel,
}

pub struct FilenameDialog {
    dialog: Panel,
    title_bar_uid: Uid,
    text_box_uid: Uid,
    button_box_uid: Uid,
    ok_uid: Uid,
    cancel_uid: Uid,
    result: DialogResult,
}

impl Default for FilenameDialog {
    fn default() -> Self {
        Self {
            dialog: Panel::default(),
            title_bar_uid: INVALID_UID,
            text_box_uid: INVALID_UID,
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
        let uid = self.text_box_uid;
        if let Some(text_box) = self.dialog.get_data_mut().node.get_child::<TextBox>(uid) {
            filename = text_box.get_text();
        }
        filename
    }
    pub fn get_result(&self) -> DialogResult {
        self.result
    }
    fn add_title(&mut self, renderer: &mut Renderer) {
        let mut title_bar = TitleBar::default();
        title_bar.collapsible(false);
        title_bar.init(renderer);

        self.title_bar_uid = self.dialog.add_child(Box::new(title_bar));
    }
    fn add_content(&mut self, renderer: &mut Renderer) {
        let mut text_box = TextBox::default();
        text_box.init(renderer);
        text_box
            .with_label("Filename: ")
            .set_text("Insert text here");

        self.text_box_uid = self.dialog.add_child(Box::new(text_box));
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
        if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
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
            .space_between_elements(20)
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
