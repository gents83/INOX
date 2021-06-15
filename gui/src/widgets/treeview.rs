use std::{any::TypeId, path::Path};

use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_message, Message};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, CollapsibleItem, InternalWidget, Panel, TitleBar,
    TitleBarEvent, WidgetData, WidgetEvent, DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
};
pub const DEFAULT_TREE_VIEW_SIZE: [f32; 2] =
    [DEFAULT_WIDGET_WIDTH * 10., DEFAULT_WIDGET_HEIGHT * 20.];

#[derive(Clone, Copy)]
pub enum TreeItemEvent {
    Collapsed(Uid),
    Expanded(Uid),
}
implement_message!(TreeItemEvent);

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct TreeView {
    data: WidgetData,
    selected_uid: Uid,
    title_widget: Uid,
    is_collapsed: bool,
    #[serde(skip)]
    is_dirty: bool,
}
implement_widget_with_custom_members!(TreeView {
    title_widget: INVALID_UID,
    selected_uid: INVALID_UID,
    is_collapsed: false,
    is_dirty: true
});

impl TreeView {
    pub fn get_selected(&self) -> Uid {
        self.selected_uid
    }

    pub fn populate_with_folders(parent_widget: &mut dyn Widget, root: &Path) {
        if let Ok(dir) = std::fs::read_dir(root) {
            dir.for_each(|entry| {
                if let Ok(dir_entry) = entry {
                    let path = dir_entry.path();
                    if path.is_dir() {
                        let mut has_children = false;
                        if let Ok(dir) = std::fs::read_dir(path.clone()) {
                            dir.for_each(|entry| {
                                if let Ok(dir_entry) = entry {
                                    let path = dir_entry.path();
                                    has_children |= path.is_dir();
                                }
                            });
                        }
                        let mut entry = CollapsibleItem::new(
                            parent_widget.get_shared_data(),
                            parent_widget.get_global_messenger(),
                        );
                        entry
                            .draggable(false)
                            .size(parent_widget.state().get_size())
                            .selectable(false)
                            .collapsible(has_children)
                            .horizontal_alignment(HorizontalAlignment::Stretch)
                            .with_text(path.file_name().unwrap().to_str().unwrap())
                            .set_name(path.to_str().unwrap());

                        let title_uid = entry.get_titlebar();
                        if let Some(title) = entry.node().get_child_mut::<TitleBar>(title_uid) {
                            let mut size = DEFAULT_WIDGET_WIDTH;
                            if !has_children {
                                size *= Screen::get_scale_factor();
                            } else {
                                size *= 0.5;
                            }
                            title
                                .space_between_elements(size as _)
                                .use_space_before_and_after(!has_children);
                        }

                        if has_children {
                            let mut inner_tree = TreeView::new(
                                entry.get_shared_data(),
                                entry.get_global_messenger(),
                            );
                            let mut inner_size = entry.state().get_size();
                            inner_size.x = (inner_size.x
                                - DEFAULT_WIDGET_WIDTH * Screen::get_scale_factor())
                            .max(0.);
                            inner_tree
                                .size(inner_size)
                                .selectable(false)
                                .horizontal_alignment(HorizontalAlignment::Right)
                                .vertical_alignment(VerticalAlignment::Top);
                            TreeView::populate_with_folders(&mut inner_tree, path.as_path());
                            entry.add_child(Box::new(inner_tree));
                        }

                        entry.collapse(true);
                        parent_widget.add_child(Box::new(entry));
                    }
                }
            });
        }
    }

    pub fn select(&mut self, widget_uid: Uid) -> &mut Self {
        if self.selected_uid != widget_uid {
            let mut new_selection = false;
            if let Some(titlebar) = self.node().get_child_mut::<TitleBar>(widget_uid) {
                new_selection = true;
                titlebar.set_selected(true);
            }
            if new_selection {
                if let Some(child) = self.node().get_child(self.selected_uid) {
                    child.write().unwrap().set_selected(false);
                }
                self.selected_uid = widget_uid;
            }
        }
        self
    }

    pub fn expand_to_selected(&mut self) -> &mut Self {
        if let Some(titlebar) = self
            .node()
            .get_child_mut::<CollapsibleItem>(self.selected_uid)
        {
            self.expand_parent(titlebar.node().get_parent());
        }
        self
    }

    fn expand_parent(&self, widget_id: Uid) {
        let mut item_id = widget_id;
        if let Some(panel) = self.node().get_child_mut::<Panel>(item_id) {
            item_id = panel.node().get_parent();
        }
        if let Some(item) = self.node().get_child_mut::<CollapsibleItem>(item_id) {
            item.collapse(false);
            if item.node().get_parent() != self.id() {
                let treeview_id = item.node().get_parent();
                if let Some(treeview) = self.node().get_child_mut::<TreeView>(treeview_id) {
                    self.expand_parent(treeview.node().get_parent());
                }
            }
        }
    }
}

impl InternalWidget for TreeView {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<TitleBarEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_TREE_VIEW_SIZE.into();

        self.size(size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Vertical)
            .vertical_alignment(VerticalAlignment::Top)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .space_between_elements(1)
            .use_space_before_and_after(true)
            .selectable(false)
            .style(WidgetStyle::DefaultCanvas);
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<TitleBarEvent>();
    }
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Released(widget_id, _mouse_in_px) = *event {
                self.select(widget_id);
            }
        }
    }
}
