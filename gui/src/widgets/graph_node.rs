use std::any::TypeId;

use nrg_math::{VecBase, Vector2};
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, TitleBar, TitleBarEvent, WidgetData,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    data: WidgetData,
    title_bar: Uid,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    expanded_size: Vector2,
    is_collapsed: bool,
}
implement_widget_with_custom_members!(GraphNode {
    title_bar: INVALID_UID,
    expanded_size: Vector2::default_zero(),
    is_collapsed: false
});

impl GraphNode {
    fn collapse(&mut self, is_collapsed: bool) {
        if self.is_collapsed != is_collapsed {
            self.is_collapsed = is_collapsed;
            let uid = self.title_bar;
            if is_collapsed {
                self.expanded_size = self.state().get_size();
                let mut size = self.expanded_size;
                if let Some(title_bar) = self.node_mut().get_child::<TitleBar>(uid) {
                    size = title_bar.state().get_size();
                }
                self.size(size);
            } else {
                self.size(self.expanded_size);
            }
        }
    }
}

impl InternalWidget for GraphNode {
    fn widget_init(&mut self) {
        self.get_global_messenger()
            .write()
            .unwrap()
            .register_messagebox::<TitleBarEvent>(self.get_messagebox());

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [300., 200.].into();
        self.expanded_size = size;
        self.position(Screen::get_center() - size / 2.)
            .size(size)
            .draggable(true)
            .horizontal_alignment(HorizontalAlignment::None)
            .vertical_alignment(VerticalAlignment::None)
            .style(WidgetStyle::DefaultBackground);

        let title_bar = TitleBar::new(self.get_shared_data(), self.get_global_messenger());
        self.title_bar = self.add_child(Box::new(title_bar));
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<TitleBarEvent>() {
            let event = msg.as_any().downcast_ref::<TitleBarEvent>().unwrap();
            if let TitleBarEvent::Collapsed(widget_id) = *event {
                if widget_id == self.title_bar {
                    self.collapse(true);
                }
            } else if let TitleBarEvent::Expanded(widget_id) = *event {
                if widget_id == self.title_bar {
                    self.collapse(false);
                }
            }
        }
    }
}
