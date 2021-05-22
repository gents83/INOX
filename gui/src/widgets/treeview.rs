use nrg_math::Vector2;
use nrg_messenger::{implement_message, Message};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, CollapsibleItem, InternalWidget, WidgetData, WidgetEvent,
    DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
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
    title_widget: Uid,
    is_collapsed: bool,
    #[serde(skip)]
    is_dirty: bool,
}
implement_widget_with_custom_members!(TreeView {
    title_widget: INVALID_UID,
    is_collapsed: false,
    is_dirty: true
});

impl TreeView {
    pub fn populate_with_folders(parent_widget: &mut dyn Widget, root: &str) {
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
                            .selectable(true)
                            .collapsible(has_children)
                            .vertical_alignment(VerticalAlignment::Top)
                            .horizontal_alignment(HorizontalAlignment::Left)
                            .style(WidgetStyle::DefaultBackground)
                            .with_text(path.file_name().unwrap().to_str().unwrap());

                        if has_children {
                            let mut inner_tree = TreeView::new(
                                entry.get_shared_data(),
                                entry.get_global_messenger(),
                            );
                            TreeView::populate_with_folders(
                                &mut inner_tree,
                                path.as_path().to_str().unwrap(),
                            );
                            inner_tree.vertical_alignment(VerticalAlignment::Top);
                            entry.add_child(Box::new(inner_tree));
                        }
                        parent_widget.add_child(Box::new(entry));
                    }
                }
            });
        }
    }
}

impl InternalWidget for TreeView {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_TREE_VIEW_SIZE.into();

        self.size(size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Vertical)
            .vertical_alignment(VerticalAlignment::Top)
            .keep_fixed_width(false)
            .space_between_elements(1)
            .use_space_before_and_after(true)
            .selectable(false)
            .style(WidgetStyle::Default);
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>();
    }
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
}
