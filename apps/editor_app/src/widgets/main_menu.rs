use std::{any::TypeId, path::PathBuf};

use nrg_gui::{
    implement_widget_with_custom_members, Button, DialogEvent, FolderDialog, InternalWidget, Menu,
    RefcountedWidget, ScrollableItem, WidgetData, WidgetEvent,
};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::Message;
use nrg_platform::WindowEvent;
use nrg_resources::DATA_RAW_FOLDER;
use nrg_serialize::*;

use crate::widget_registry::{NodesEvent, WidgetRegistry};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct MainMenu {
    data: WidgetData,
    menu: Option<Menu>,
    #[serde(skip)]
    file_id: Uid,
    #[serde(skip)]
    new_id: Uid,
    #[serde(skip)]
    open_id: Uid,
    #[serde(skip)]
    save_id: Uid,
    #[serde(skip)]
    exit_id: Uid,
    #[serde(skip)]
    edit_id: Uid,
    #[serde(skip)]
    nodes_id: Uid,
    #[serde(skip)]
    add_id: Uid,
    #[serde(skip)]
    list_id: Uid,
    #[serde(skip)]
    filename_dialog: Option<FolderDialog>,
}
implement_widget_with_custom_members!(MainMenu {
    menu: None,
    file_id: INVALID_UID,
    new_id: INVALID_UID,
    save_id: INVALID_UID,
    open_id: INVALID_UID,
    exit_id: INVALID_UID,
    edit_id: INVALID_UID,
    nodes_id: INVALID_UID,
    add_id: INVALID_UID,
    list_id: INVALID_UID,
    filename_dialog: None
});

impl MainMenu {
    pub fn menu(&self) -> &Menu {
        self.menu.as_ref().unwrap()
    }
    fn menu_mut(&mut self) -> &mut Menu {
        self.menu.as_mut().unwrap()
    }
    pub fn get_size(&self) -> Vector2 {
        self.menu().state().get_size()
    }
    pub fn is_new_uid(&self, entry_uid: Uid) -> bool {
        self.new_id == entry_uid
    }
    pub fn is_open_uid(&self, entry_uid: Uid) -> bool {
        self.open_id == entry_uid
    }
    pub fn is_save_uid(&self, entry_uid: Uid) -> bool {
        self.save_id == entry_uid
    }
    pub fn fill_nodes_from_registry(&mut self, registry: &WidgetRegistry) -> &mut Self {
        let edit_id = self.edit_id;
        let nodes_id = self.nodes_id;
        let add_id = self.add_id;
        let list_id = self.list_id;
        let menu = self.menu_mut();
        if let Some(edit) = menu.get_submenu(edit_id) {
            if let Some(menu) = edit.node().get_child_mut::<Menu>(nodes_id) {
                if let Some(add) = menu.get_submenu(add_id) {
                    if let Some(list) = add.node().get_child_mut::<ScrollableItem>(list_id) {
                        list.clear();
                        if let Some(scrollable_panel) = list.get_scrollable_panel() {
                            for i in 0..registry.count() {
                                let mut button = Button::new(
                                    &add.get_shared_data(),
                                    &add.get_global_messenger(),
                                );
                                let name = registry.get_name_from_index(i);
                                button
                                    .with_text(name)
                                    .text_alignment(
                                        VerticalAlignment::Center,
                                        HorizontalAlignment::Left,
                                    )
                                    .horizontal_alignment(HorizontalAlignment::Stretch)
                                    .vertical_alignment(VerticalAlignment::Top)
                                    .fill_type(ContainerFillType::Horizontal)
                                    .node_mut()
                                    .set_name(name);
                                scrollable_panel.add_child(Box::new(button));
                            }
                        }
                        list.vertical();
                    }
                }
            }
        }
        self
    }

    fn get_node_in_list(&mut self, uid: Uid) -> Option<RefcountedWidget> {
        let edit_id = self.edit_id;
        let nodes_id = self.nodes_id;
        let add_id = self.add_id;
        let list_id = self.list_id;
        let menu = self.menu_mut();
        if let Some(edit) = menu.get_submenu(edit_id) {
            if let Some(menu) = edit.node().get_child_mut::<Menu>(nodes_id) {
                if let Some(add) = menu.get_submenu(add_id) {
                    if let Some(list) = add.node().get_child_mut::<ScrollableItem>(list_id) {
                        if let Some(node) = list.node().get_child(uid) {
                            return Some(node);
                        }
                    }
                }
            }
        }
        None
    }
}

impl InternalWidget for MainMenu {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<DialogEvent>()
            .register_to_listen_event::<WidgetEvent>();

        self.menu = Some(Menu::new(
            self.get_shared_data(),
            self.get_global_messenger(),
        ));
        self.menu_mut().move_to_layer(0.5);

        let file_id = self.menu_mut().add_menu_item("File");
        self.file_id = file_id;

        self.new_id = self.menu_mut().add_submenu_entry_default(file_id, "New");
        self.open_id = self.menu_mut().add_submenu_entry_default(file_id, "Open");
        self.save_id = self.menu_mut().add_submenu_entry_default(file_id, "Save");
        self.exit_id = self.menu_mut().add_submenu_entry_default(file_id, "Exit");

        let edit_id = self.menu_mut().add_menu_item("Edit");
        self.edit_id = edit_id;

        let mut new_menu = Menu::new(self.get_shared_data(), self.get_global_messenger());
        new_menu.vertical();
        self.add_id = new_menu.add_menu_item("Add ->");
        let mut list = ScrollableItem::new(self.get_shared_data(), self.get_global_messenger());
        list.clear()
            .vertical()
            .style(WidgetStyle::DefaultBackground);
        self.list_id = new_menu.add_submenu_entry(self.add_id, Box::new(list));

        self.nodes_id = self
            .menu_mut()
            .add_submenu_entry(edit_id, Box::new(new_menu));
    }

    fn widget_update(&mut self, drawing_area_in_px: Vector4) {
        self.menu_mut()
            .update(drawing_area_in_px, drawing_area_in_px);

        if let Some(dialog) = &mut self.filename_dialog {
            dialog.update(drawing_area_in_px, drawing_area_in_px);
        }
    }

    fn widget_uninit(&mut self) {
        if let Some(dialog) = &mut self.filename_dialog {
            dialog.uninit();
            self.filename_dialog = None;
        }
        self.menu_mut().uninit();

        self.unregister_to_listen_event::<DialogEvent>()
            .unregister_to_listen_event::<WidgetEvent>();
    }

    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<DialogEvent>() {
            if let Some(dialog) = &mut self.filename_dialog {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match event {
                    DialogEvent::Confirmed(widget_id, _requester_uid, _text) => {
                        if *widget_id == dialog.id() {
                            dialog.uninit();
                            self.filename_dialog = None;
                        }
                    }
                    DialogEvent::Canceled(widget_id) => {
                        if *widget_id == dialog.id() {
                            dialog.uninit();
                            self.filename_dialog = None;
                        }
                    }
                }
            }
        } else if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = *event {
                if self.new_id == widget_id {
                    self.filename_dialog = Some(FolderDialog::new(
                        self.get_shared_data(),
                        self.get_global_messenger(),
                    ));
                    let dialog = self.filename_dialog.as_mut().unwrap();
                    dialog
                        .set_requester_uid(self.new_id)
                        .set_title("New Widget")
                        .set_filename("new_widget.widget")
                        .editable(true);
                } else if self.open_id == widget_id && self.filename_dialog.is_none() {
                    self.filename_dialog = Some(FolderDialog::new(
                        self.get_shared_data(),
                        self.get_global_messenger(),
                    ));
                    let dialog = self.filename_dialog.as_mut().unwrap();
                    dialog
                        .set_requester_uid(self.open_id)
                        .set_title("Open Widget")
                        .set_folder(PathBuf::from(DATA_RAW_FOLDER).as_path())
                        .editable(false);
                } else if self.save_id == widget_id && self.filename_dialog.is_none() {
                    self.filename_dialog = Some(FolderDialog::new(
                        self.get_shared_data(),
                        self.get_global_messenger(),
                    ));
                    let dialog = self.filename_dialog.as_mut().unwrap();
                    dialog
                        .set_requester_uid(self.save_id)
                        .set_title("Save Widget")
                        .set_filename("old_widget.widget")
                        .editable(true);
                } else if self.exit_id == widget_id {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(WindowEvent::Close.as_boxed())
                        .ok();
                } else if let Some(child) = self.get_node_in_list(widget_id) {
                    self.get_global_dispatcher()
                        .write()
                        .unwrap()
                        .send(
                            NodesEvent::Create(String::from(
                                child.read().unwrap().node().get_name(),
                            ))
                            .as_boxed(),
                        )
                        .ok();
                }
            }
        }
    }
    fn widget_on_layout_changed(&mut self) {}
}
