use std::any::TypeId;

use nrg_math::{VecBase, Vector2};
use nrg_messenger::Message;
use nrg_platform::MouseEvent;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, TitleBar, TitleBarEvent,
    WidgetData, WidgetEvent,
};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct CollapsibleItem {
    data: WidgetData,
    title_bar: Uid,
    panel: Uid,
    expanded_size: Vector2,
    expanded_fill_type: ContainerFillType,
    is_collapsed: bool,
}
implement_widget_with_custom_members!(CollapsibleItem {
    title_bar: INVALID_UID,
    panel: INVALID_UID,
    expanded_fill_type: ContainerFillType::Vertical,
    expanded_size: Vector2::default_zero(),
    is_collapsed: false
});

impl CollapsibleItem {
    pub fn with_text(&mut self, text: &str) -> &mut Self {
        let uid = self.title_bar;
        if let Some(title_bar) = self.node_mut().get_child::<TitleBar>(uid) {
            title_bar.set_text(text);
        }
        self
    }
    pub fn collapsible(&mut self, can_collapse: bool) -> &mut Self {
        let uid = self.title_bar;
        if let Some(title_bar) = self.node_mut().get_child::<TitleBar>(uid) {
            title_bar.collapsible(can_collapse);
        }
        self
    }
    pub fn collapse(&mut self, is_collapsed: bool) -> &mut Self {
        if self.is_collapsed != is_collapsed {
            self.is_collapsed = is_collapsed;
            let uid = self.panel;
            let mut expanded_size = self.expanded_size;
            let mut expanded_fill_type = self.expanded_fill_type;
            if let Some(panel) = self.node_mut().get_child::<Panel>(uid) {
                if is_collapsed {
                    expanded_size = panel.state().get_size();
                    expanded_fill_type = panel.state().get_fill_type();
                    panel
                        .size(Vector2::default_zero())
                        .fill_type(ContainerFillType::None);
                } else {
                    panel.size(expanded_size);
                    panel.fill_type(expanded_fill_type);
                }
            }
            let size = if is_collapsed {
                let uid = self.title_bar;
                if let Some(title_bar) = self.node_mut().get_child::<TitleBar>(uid) {
                    title_bar.state().get_size()
                } else {
                    Vector2::default_zero()
                }
            } else {
                if let Some(title_bar) = self.node_mut().get_child::<TitleBar>(uid) {
                    expanded_size.y += title_bar.state().get_size().y;
                }
                expanded_size
            };
            self.size(size);
            self.expanded_size = expanded_size;
            self.expanded_fill_type = expanded_fill_type;
        }
        self
    }
    pub fn add_child(&mut self, widget: Box<dyn Widget>) -> Uid {
        let uid = self.panel;
        if let Some(panel) = self.node_mut().get_child::<Panel>(uid) {
            panel.add_child(widget)
        } else {
            <dyn Widget>::add_child(self, widget)
        }
    }
}

impl InternalWidget for CollapsibleItem {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<TitleBarEvent>()
            .register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [150., 100.].into();
        self.expanded_size = size * Screen::get_scale_factor();
        self.position(Screen::get_center() - size * Screen::get_scale_factor() / 2.)
            .size(size * Screen::get_scale_factor())
            .selectable(true)
            .draggable(true)
            .fill_type(ContainerFillType::Vertical)
            .style(WidgetStyle::Invisible);

        let mut title_bar = TitleBar::new(self.get_shared_data(), self.get_global_messenger());
        title_bar
            .style(WidgetStyle::DefaultBackground)
            .collapsible(true)
            .set_text_alignment(HorizontalAlignment::Left, VerticalAlignment::Center);
        self.title_bar = self.add_child(Box::new(title_bar));

        let mut panel = Panel::new(self.get_shared_data(), self.get_global_messenger());
        panel
            .size(size * Screen::get_scale_factor())
            .fill_type(ContainerFillType::Vertical)
            .style(WidgetStyle::Invisible);
        self.panel = self.add_child(Box::new(panel));
    }

    fn widget_update(&mut self) {}

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
}
