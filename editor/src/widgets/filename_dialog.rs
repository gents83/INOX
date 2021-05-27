use std::any::TypeId;

use nrg_gui::{
    implement_widget_with_custom_members, Button, DialogEvent, InternalWidget, Panel, TextBox,
    TitleBar, WidgetData, WidgetEvent, DEFAULT_BUTTON_SIZE,
};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::Message;
use nrg_serialize::*;

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
    requester_uid: Uid,
}
implement_widget_with_custom_members!(FilenameDialog {
    title_bar_uid: INVALID_UID,
    text_box_uid: INVALID_UID,
    button_box_uid: INVALID_UID,
    requester_uid: INVALID_UID,
    ok_uid: INVALID_UID,
    cancel_uid: INVALID_UID
});

impl FilenameDialog {
    pub fn set_title(&mut self, text: &str) -> &mut Self {
        let uid = self.title_bar_uid;
        if let Some(title_bar) = self.node().get_child_mut::<TitleBar>(uid) {
            title_bar.set_text(text);
        }
        self
    }
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        let uid = self.text_box_uid;
        if let Some(text_box) = self.node().get_child_mut::<TextBox>(uid) {
            text_box.set_text(text);
        }
        self
    }
    pub fn get_text(&mut self) -> String {
        let mut filename = String::new();
        let uid = self.text_box_uid;
        if let Some(text_box) = self.node().get_child_mut::<TextBox>(uid) {
            filename = text_box.get_text();
        }
        filename
    }
    pub fn set_requester_uid(&mut self, requester_uid: Uid) -> &mut Self {
        self.requester_uid = requester_uid;
        self
    }
    fn add_title(&mut self) {
        let mut title_bar = TitleBar::new(self.get_shared_data(), self.get_global_messenger());
        title_bar.collapsible(false);

        self.title_bar_uid = self.add_child(Box::new(title_bar));
    }
    fn add_content(&mut self) {
        let mut text_box = TextBox::new(self.get_shared_data(), self.get_global_messenger());
        text_box
            .with_label("Filename: ")
            .set_text("Insert text here");

        self.text_box_uid = self.add_child(Box::new(text_box));
    }

    fn add_buttons(&mut self) {
        let mut button_box = Panel::new(self.get_shared_data(), self.get_global_messenger());

        let default_size: Vector2 = DEFAULT_BUTTON_SIZE.into();
        button_box
            .size(default_size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Horizontal)
            .horizontal_alignment(HorizontalAlignment::Right)
            .keep_fixed_height(true)
            .space_between_elements(40);

        let mut button_ok = Button::new(self.get_shared_data(), self.get_global_messenger());
        button_ok.with_text("Ok");

        let mut button_cancel = Button::new(self.get_shared_data(), self.get_global_messenger());
        button_cancel
            .with_text("Cancel")
            .horizontal_alignment(HorizontalAlignment::Right);

        self.ok_uid = button_box.add_child(Box::new(button_ok));
        self.cancel_uid = button_box.add_child(Box::new(button_cancel));
        self.button_box_uid = self.add_child(Box::new(button_box));
    }
}

impl InternalWidget for FilenameDialog {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<DialogEvent>()
            .register_to_listen_event::<WidgetEvent>();

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

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<DialogEvent>()
            .unregister_to_listen_event::<WidgetEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = *event {
                if self.ok_uid == widget_id {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(
                            DialogEvent::Confirmed(self.id(), self.requester_uid, self.get_text())
                                .as_boxed(),
                        )
                        .ok();
                } else if self.cancel_uid == widget_id {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(DialogEvent::Canceled(self.id()).as_boxed())
                        .ok();
                }
            }
        }
    }
}
