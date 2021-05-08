use nrg_events::EventsRw;
use nrg_math::{VecBase, Vector2};
use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{implement_widget, InternalWidget, TitleBar, TitleBarEvent, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    data: WidgetData,
    title_bar: Uid,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    expanded_size: Vector2,
    is_collapsed: bool,
}
implement_widget!(GraphNode);

impl GraphNode {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        let mut w = Self {
            data: WidgetData::new(shared_data),
            title_bar: INVALID_UID,
            expanded_size: Vector2::default_zero(),
            is_collapsed: false,
        };
        w.init();
        w
    }
}

impl GraphNode {
    fn collapse(&mut self, is_collapsed: bool) {
        if self.is_collapsed != is_collapsed {
            self.is_collapsed = is_collapsed;
            let uid = self.title_bar;
            if is_collapsed {
                self.expanded_size = self.get_data().state.get_size();
                let mut size = self.expanded_size;
                if let Some(title_bar) = self.get_data_mut().node.get_child::<TitleBar>(uid) {
                    size = title_bar.get_data().state.get_size();
                }
                self.size(size);
            } else {
                self.size(self.expanded_size);
            }
        }
    }
    fn manage_events(&mut self) -> &mut Self {
        let is_collapsed = {
            let mut collapse = self.is_collapsed;
            let read_data = self.get_shared_data().read().unwrap();
            let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
            let events = events_rw.read().unwrap();
            if let Some(widget_events) = events.read_all_events::<TitleBarEvent>() {
                for event in widget_events.iter() {
                    if let TitleBarEvent::Collapsed(widget_id) = event {
                        if *widget_id == self.title_bar {
                            collapse = true;
                        }
                    } else if let TitleBarEvent::Expanded(widget_id) = event {
                        if *widget_id == self.title_bar {
                            collapse = false;
                        }
                    }
                }
            }
            collapse
        };
        self.collapse(is_collapsed);
        self
    }
}

impl InternalWidget for GraphNode {
    fn widget_init(&mut self) {
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

        let title_bar = TitleBar::new(self.get_shared_data());
        self.title_bar = self.add_child(Box::new(title_bar));
    }

    fn widget_update(&mut self) {
        self.manage_events();
    }

    fn widget_uninit(&mut self) {}
}
