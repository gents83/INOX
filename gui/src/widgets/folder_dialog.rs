use std::{any::TypeId, path::PathBuf};

use crate::{
    implement_widget_with_custom_members, Button, Icon, InternalWidget, List, Panel,
    ScrollbarEvent, Separator, TitleBar, TreeView, WidgetData, WidgetEvent, DEFAULT_BUTTON_SIZE,
    DEFAULT_WIDGET_HEIGHT,
};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_message, Message};
use nrg_resources::DATA_RAW_FOLDER;
use nrg_serialize::*;

#[derive(Clone)]
pub enum DialogEvent {
    Confirmed(Uid, Uid, String), //my uid, requester uid, text
    Canceled(Uid),
}
implement_message!(DialogEvent);

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct FolderDialog {
    data: WidgetData,
    folder: PathBuf,
    folder_treeview_uid: Uid,
    list: Uid,
    title_bar_uid: Uid,
    button_box_uid: Uid,
    ok_uid: Uid,
    cancel_uid: Uid,
    #[serde(skip)]
    requester_uid: Uid,
}
implement_widget_with_custom_members!(FolderDialog {
    folder_treeview_uid: INVALID_UID,
    folder: PathBuf::from(DATA_RAW_FOLDER),
    list: INVALID_UID,
    title_bar_uid: INVALID_UID,
    button_box_uid: INVALID_UID,
    requester_uid: INVALID_UID,
    ok_uid: INVALID_UID,
    cancel_uid: INVALID_UID
});

impl FolderDialog {
    pub fn set_title(&mut self, text: &str) -> &mut Self {
        let uid = self.title_bar_uid;
        if let Some(title_bar) = self.node().get_child_mut::<TitleBar>(uid) {
            title_bar.set_text(text);
        }
        self
    }
    pub fn set_requester_uid(&mut self, requester_uid: Uid) -> &mut Self {
        self.requester_uid = requester_uid;
        self
    }
    fn add_title(&mut self) {
        let mut title_bar = TitleBar::new(self.get_shared_data(), self.get_global_messenger());
        title_bar.collapsible(false).set_text("Folder Dialog");

        self.title_bar_uid = self.add_child(Box::new(title_bar));
    }
    fn add_content(&mut self) {
        let mut content_size = self.state().get_size();
        content_size.y -= DEFAULT_WIDGET_HEIGHT * 2. * Screen::get_scale_factor();

        let mut horizontal_panel = Panel::new(self.get_shared_data(), self.get_global_messenger());
        horizontal_panel
            .fill_type(ContainerFillType::Horizontal)
            .selectable(false)
            .space_between_elements(2)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .size(content_size)
            .style(WidgetStyle::DefaultCanvas);

        let mut treeview = TreeView::new(self.get_shared_data(), self.get_global_messenger());

        TreeView::populate_with_folders(&mut treeview, self.folder.as_path());
        treeview
            .horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Stretch);

        self.folder_treeview_uid = horizontal_panel.add_child(Box::new(treeview));

        let mut separator = Separator::new(self.get_shared_data(), self.get_global_messenger());
        separator
            .vertical_alignment(VerticalAlignment::Stretch)
            .horizontal_alignment(HorizontalAlignment::Left);
        horizontal_panel.add_child(Box::new(separator));

        let mut list = List::new(self.get_shared_data(), self.get_global_messenger());
        list.vertical();
        if let Some(icon_panel) = list.get_scrollable_panel() {
            Icon::create_icons(self.folder.as_path(), icon_panel);
        }

        self.list = horizontal_panel.add_child(Box::new(list));

        self.add_child(Box::new(horizontal_panel));
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

impl InternalWidget for FolderDialog {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<DialogEvent>()
            .register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<ScrollbarEvent>();

        let size: Vector2 = [500., 400.].into();
        self.size(size * Screen::get_scale_factor())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center)
            .fill_type(ContainerFillType::Vertical)
            .keep_fixed_width(false)
            .keep_fixed_height(true)
            .use_space_before_and_after(true)
            .space_between_elements((2. * Screen::get_scale_factor()) as _)
            .selectable(false)
            .style(WidgetStyle::DefaultBackground)
            .move_to_layer(1.);

        self.graphics_mut()
            .set_border_color([1., 1., 1., 2. * Screen::get_scale_factor()].into());

        self.add_title();
        self.add_content();
        self.add_buttons();
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<DialogEvent>()
            .unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<ScrollbarEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Released(widget_id, _mouse_in_px) = *event {
                if self.ok_uid == widget_id {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(
                            DialogEvent::Confirmed(
                                self.id(),
                                self.requester_uid,
                                String::from("WhichEntry?"),
                            )
                            .as_boxed(),
                        )
                        .ok();
                } else if self.cancel_uid == widget_id {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(DialogEvent::Canceled(self.id()).as_boxed())
                        .ok();
                } else {
                    let mut folder = String::from(self.folder.to_str().unwrap());
                    let mut should_change = false;
                    if let Some(child) = self.node().get_child_mut::<TitleBar>(widget_id) {
                        let name = child.node().get_name();
                        if !name.is_empty() {
                            folder = String::from(name);
                            should_change = true;
                        }
                    }
                    if should_change {
                        let list_uid = self.list;
                        if let Some(list) = self.node().get_child_mut::<List>(list_uid) {
                            list.clear();
                            if let Some(iconpanel) = list.get_scrollable_panel() {
                                Icon::create_icons(PathBuf::from(folder).as_path(), iconpanel);
                            }
                            list.vertical();
                        }
                    }
                }
            }
        }
    }
}
