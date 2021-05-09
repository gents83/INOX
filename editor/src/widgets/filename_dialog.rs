use nrg_gui::{
    implement_widget_with_custom_members, Button, InternalWidget, Panel, TextBox, TitleBar,
    WidgetData, WidgetEvent, DEFAULT_BUTTON_SIZE,
};
use nrg_math::Vector2;
use nrg_serialize::*;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub enum DialogResult {
    Waiting,
    Ok,
    Cancel,
}

impl Default for DialogResult {
    fn default() -> Self {
        Self::Waiting
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct FilenameDialog {
    data: WidgetData,
    #[serde(skip)]
    title_bar_uid: Uid,
    #[serde(skip)]
    text_box_uid: Uid,
    #[serde(skip)]
    button_box_uid: Uid,
    #[serde(skip)]
    ok_uid: Uid,
    #[serde(skip)]
    cancel_uid: Uid,
    #[serde(skip)]
    result: DialogResult,
}
implement_widget_with_custom_members!(FilenameDialog {
    title_bar_uid: INVALID_UID,
    text_box_uid: INVALID_UID,
    button_box_uid: INVALID_UID,
    ok_uid: INVALID_UID,
    cancel_uid: INVALID_UID,
    result: DialogResult::Waiting
});

impl FilenameDialog {
    pub fn get_filename(&mut self) -> String {
        let mut filename = String::new();
        let uid = self.text_box_uid;
        if let Some(text_box) = self.node_mut().get_child::<TextBox>(uid) {
            filename = text_box.get_text();
        }
        filename
    }
    pub fn get_result(&self) -> DialogResult {
        self.result
    }
    fn add_title(&mut self) {
        let mut title_bar = TitleBar::new(self.get_shared_data(), self.get_events());
        title_bar.collapsible(false);

        self.title_bar_uid = self.add_child(Box::new(title_bar));
    }
    fn add_content(&mut self) {
        let mut text_box = TextBox::new(self.get_shared_data(), self.get_events());
        text_box
            .with_label("Filename: ")
            .set_text("Insert text here");

        self.text_box_uid = self.add_child(Box::new(text_box));
    }

    fn add_buttons(&mut self) {
        let mut button_box = Panel::new(self.get_shared_data(), self.get_events());

        let default_size: Vector2 = DEFAULT_BUTTON_SIZE.into();
        button_box
            .size(default_size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .horizontal_alignment(HorizontalAlignment::Right)
            .keep_fixed_height(true)
            .space_between_elements(40);

        let mut button_ok = Button::new(self.get_shared_data(), self.get_events());
        button_ok.with_text("Ok");

        let mut button_cancel = Button::new(self.get_shared_data(), self.get_events());
        button_cancel
            .with_text("Cancel")
            .horizontal_alignment(HorizontalAlignment::Right);

        self.ok_uid = button_box.add_child(Box::new(button_ok));
        self.cancel_uid = button_box.add_child(Box::new(button_cancel));
        self.button_box_uid = self.add_child(Box::new(button_box));
    }
    fn manage_events(&mut self) {
        let result = {
            let mut result = DialogResult::Waiting;
            let events = self.get_events().read().unwrap();
            if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
                for event in widget_events.iter() {
                    if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
                        if self.ok_uid == *widget_id {
                            result = DialogResult::Ok;
                        } else if self.cancel_uid == *widget_id {
                            result = DialogResult::Cancel;
                        }
                    }
                }
            }
            result
        };
        if result != DialogResult::Waiting {
            self.result = result;
        }
    }
}

impl InternalWidget for FilenameDialog {
    fn widget_init(&mut self) {
        let size: Vector2 = [500., 200.].into();
        self.size(size * Screen::get_scale_factor())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(true)
            .selectable(false)
            .space_between_elements(20)
            .style(WidgetStyle::DefaultBackground)
            .move_to_layer(1.);

        self.add_title();
        self.add_content();
        self.add_buttons();
    }

    fn widget_update(&mut self) {
        self.manage_events();
    }

    fn widget_uninit(&mut self) {}
}
