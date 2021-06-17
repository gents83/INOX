use std::any::TypeId;

use nrg_math::{VecBase, Vector2, Vector4};
use nrg_messenger::Message;
use nrg_platform::MouseEvent;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, TitleBar, TitleBarEvent, WidgetData,
    WidgetEvent,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    data: WidgetData,
    title_bar: Uid,
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
                if let Some(title_bar) = self.node().get_child_mut::<TitleBar>(uid) {
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
        self.register_to_listen_event::<TitleBarEvent>()
            .register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

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

        let mut title_bar = TitleBar::new(self.get_shared_data(), self.get_global_messenger());
        title_bar.selectable(true);
        self.title_bar = self.add_child(Box::new(title_bar));
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<TitleBarEvent>()
            .unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>();
    }
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
    fn widget_on_layout_changed(&mut self) {}
}
