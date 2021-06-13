use std::any::TypeId;

use nrg_math::{VecBase, Vector2, Vector4};
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, Scrollbar, ScrollbarEvent,
    WidgetData, WidgetEvent, DEFAULT_WIDGET_HEIGHT,
};

pub const DEFAULT_LIST_SIZE: [f32; 2] = [DEFAULT_WIDGET_HEIGHT * 10., DEFAULT_WIDGET_HEIGHT * 10.];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct List {
    data: WidgetData,
    selected: Uid,
    base_panel: Uid,
    scrollable_panel: Uid,
    scrollbar: Uid,
}
implement_widget_with_custom_members!(List {
    base_panel: INVALID_UID,
    scrollable_panel: INVALID_UID,
    scrollbar: INVALID_UID,
    selected: INVALID_UID
});

impl List {
    pub fn clear(&mut self) -> &mut Self {
        let scrollable_panel_uid = self.scrollable_panel;
        if let Some(scrollable_panel) = self.node().get_child_mut::<Panel>(scrollable_panel_uid) {
            scrollable_panel.node_mut().remove_children();
            scrollable_panel
                .vertical_alignment(VerticalAlignment::Stretch)
                .horizontal_alignment(HorizontalAlignment::Stretch);
        }
        self
    }
    pub fn select_first(&mut self) -> &mut Self {
        let mut selected_uid = self.selected;
        if let Some(scrollable_panel) = self.node().get_child_mut::<Panel>(self.scrollable_panel) {
            let mut is_selected = true;
            scrollable_panel.node_mut().propagate_on_children_mut(|w| {
                w.set_selected(is_selected);
                if is_selected {
                    selected_uid = w.id();
                    is_selected = false;
                }
            });
        }
        self.selected = selected_uid;
        self
    }

    pub fn get_selected(&self) -> Uid {
        self.selected
    }

    pub fn get_scrollable_panel(&mut self) -> Option<&mut Panel> {
        let scrollable_panel_uid = self.scrollable_panel;
        self.node().get_child_mut::<Panel>(scrollable_panel_uid)
    }
    pub fn vertical(&mut self) -> &mut Self {
        self.fill_type(ContainerFillType::Horizontal);

        let mut view_size = 0.;
        let mut children_size = 0.;
        let draw_area = self.compute_area_data();

        let base_panel_uid = self.base_panel;
        if let Some(base_panel) = self.node().get_child_mut::<Panel>(base_panel_uid) {
            view_size = base_panel.compute_children_drawing_area(draw_area).w;
        }
        let scrollable_panel_uid = self.scrollable_panel;
        if let Some(scrollable_panel) = self.node().get_child_mut::<Panel>(scrollable_panel_uid) {
            scrollable_panel.fill_type(ContainerFillType::Vertical);
            children_size = scrollable_panel
                .compute_children_size(scrollable_panel.state().get_size())
                .y;
        }
        let scrollbar_uid = self.scrollbar;
        if let Some(scrollbar) = self.node().get_child_mut::<Scrollbar>(scrollbar_uid) {
            if view_size <= 0. || children_size <= view_size {
                scrollbar.vertical().visible(false).selectable(false);
            } else {
                scrollbar.vertical().visible(true).selectable(true);
            }
        }

        self
    }
    pub fn horizontal(&mut self) -> &mut Self {
        self.fill_type(ContainerFillType::Vertical);

        let mut view_size = 0.;
        let mut children_size = 0.;
        let draw_area = self.compute_area_data();

        let base_panel_uid = self.base_panel;
        if let Some(base_panel) = self.node().get_child_mut::<Panel>(base_panel_uid) {
            view_size = base_panel.compute_children_drawing_area(draw_area).z;
        }
        let scrollable_panel_uid = self.scrollable_panel;
        if let Some(scrollable_panel) = self.node().get_child_mut::<Panel>(scrollable_panel_uid) {
            scrollable_panel.fill_type(ContainerFillType::Horizontal);
            children_size = scrollable_panel
                .compute_children_size(scrollable_panel.state().get_size())
                .x;
        }
        let scrollbar_uid = self.scrollbar;
        if let Some(scrollbar) = self.node().get_child_mut::<Scrollbar>(scrollbar_uid) {
            if view_size <= 0. || children_size <= view_size {
                scrollbar.horizontal().visible(false).selectable(false);
            } else {
                scrollbar.horizontal().visible(true).selectable(true);
            }
        }

        self
    }
}

impl InternalWidget for List {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<ScrollbarEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_LIST_SIZE.into();
        self.size(size * Screen::get_scale_factor())
            .selectable(false)
            .fill_type(ContainerFillType::Horizontal)
            .keep_fixed_height(false)
            .keep_fixed_width(false)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .style(WidgetStyle::DefaultBackground);

        let mut base_panel = Panel::new(self.get_shared_data(), self.get_global_messenger());
        base_panel
            .fill_type(ContainerFillType::None)
            .selectable(false)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch);

        let mut scrollable_panel = Panel::new(self.get_shared_data(), self.get_global_messenger());
        scrollable_panel
            .fill_type(ContainerFillType::Vertical)
            .selectable(false)
            .space_between_elements(2)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch);

        self.scrollable_panel = base_panel.add_child(Box::new(scrollable_panel));
        self.base_panel = self.add_child(Box::new(base_panel));

        let mut scrollbar = Scrollbar::new(self.get_shared_data(), self.get_global_messenger());
        scrollbar.vertical().visible(false).selectable(false);
        self.scrollbar = self.add_child(Box::new(scrollbar));
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<ScrollbarEvent>();
    }
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<ScrollbarEvent>() {
            let event = msg.as_any().downcast_ref::<ScrollbarEvent>().unwrap();
            let ScrollbarEvent::Changed(widget_id, percentage) = *event;
            if widget_id == self.scrollbar {
                let mut pos = Vector2::default_zero();
                let mut view_size = 0.;
                let base_panel_uid = self.base_panel;
                let draw_area = self.compute_area_data();
                if let Some(base_panel) = self.node().get_child_mut::<Panel>(base_panel_uid) {
                    pos = base_panel.state().get_position();
                    view_size = base_panel.compute_children_drawing_area(draw_area).w;
                }
                let scrollable_panel_uid = self.scrollable_panel;
                if let Some(scrollable_panel) =
                    self.node().get_child_mut::<Panel>(scrollable_panel_uid)
                {
                    let children_size = scrollable_panel
                        .compute_children_size(scrollable_panel.state().get_size())
                        .y;
                    let space = (children_size - view_size) * percentage;
                    pos.y -= space;
                    scrollable_panel
                        .vertical_alignment(VerticalAlignment::None)
                        .set_position(pos);
                }
            }
        } else if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Released(widget_id, _mouse_in_px) = *event {
                let mut selected_uid = self.selected;
                let scrollable_panel_uid = self.scrollable_panel;
                if let Some(scrollable_panel) =
                    self.node().get_child_mut::<Panel>(scrollable_panel_uid)
                {
                    if let Some(child) = scrollable_panel.node().get_child(widget_id) {
                        selected_uid = widget_id;
                        child.write().unwrap().set_selected(true);
                    }
                    if selected_uid != self.selected {
                        if let Some(child) = scrollable_panel.node().get_child(self.selected) {
                            child.write().unwrap().set_selected(false);
                        }
                    }
                }
                self.selected = selected_uid;
            }
        }
    }
}
