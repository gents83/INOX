use nrg_graphics::{Renderer, INVALID_ID};
use nrg_math::{VecBase, Vector2};
use nrg_platform::EventsRw;
use nrg_serialize::{Deserialize, Serialize, Uid};

use crate::{implement_widget, InternalWidget, TitleBar, TitleBarEvent, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct GraphNode {
    data: WidgetData,
    title_bar: Uid,
    #[serde(skip, default = "nrg_math::VecBase::default_zero")]
    expanded_size: Vector2,
}
implement_widget!(GraphNode);

impl Default for GraphNode {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
            title_bar: INVALID_ID,
            expanded_size: Vector2::default_zero(),
        }
    }
}

impl GraphNode {
    fn manage_events(&mut self, events_rw: &mut EventsRw) -> &mut Self {
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_events::<TitleBarEvent>() {
            for event in widget_events.iter() {
                if let TitleBarEvent::Collapsed(widget_id) = event {
                    if *widget_id == self.title_bar {
                        self.expanded_size = self.get_data().state.get_size();
                        let mut size = self.expanded_size;
                        if let Some(title_bar) =
                            self.get_data_mut().node.get_child::<TitleBar>(*widget_id)
                        {
                            size = title_bar.get_data().state.get_size();
                        }
                        self.size(size);
                    }
                } else if let TitleBarEvent::Expanded(widget_id) = event {
                    if *widget_id == self.title_bar {
                        self.size(self.expanded_size);
                    }
                }
            }
        }
        self
    }
}

impl InternalWidget for GraphNode {
    fn widget_init(&mut self, renderer: &mut Renderer) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [200., 100.].into();
        self.expanded_size = size;
        self.position(Screen::get_center() - size / 2.)
            .size(size)
            .draggable(true)
            .style(WidgetStyle::DefaultBackground);

        let mut title_bar = TitleBar::default();
        title_bar.init(renderer);
        self.title_bar = self.add_child(Box::new(title_bar));
    }

    fn widget_update(&mut self, _renderer: &mut Renderer, events: &mut EventsRw) {
        self.manage_events(events);
    }

    fn widget_uninit(&mut self, _renderer: &mut Renderer) {}
}
