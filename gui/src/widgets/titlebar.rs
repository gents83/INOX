use nrg_events::{implement_event, Event, EventsRw};
use nrg_graphics::{
    utils::{create_triangle_down, create_triangle_up},
    MeshData,
};
use nrg_math::Vector2;
use nrg_resources::SharedDataRw;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget, Canvas, InternalWidget, Text, WidgetData, WidgetEvent, DEFAULT_WIDGET_HEIGHT,
    DEFAULT_WIDGET_WIDTH,
};
pub const DEFAULT_ICON_SIZE: [f32; 2] = [
    DEFAULT_WIDGET_WIDTH * 2. / 3.,
    DEFAULT_WIDGET_HEIGHT * 2. / 3.,
];

#[derive(Clone, Copy)]
pub enum TitleBarEvent {
    Collapsed(Uid),
    Expanded(Uid),
}
implement_event!(TitleBarEvent);

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct TitleBar {
    title_widget: Uid,
    collapse_icon_widget: Uid,
    data: WidgetData,
    is_collapsible: bool,
    is_collapsed: bool,
    is_dirty: bool,
}
implement_widget!(TitleBar);

impl Default for TitleBar {
    fn default() -> Self {
        Self {
            data: WidgetData::default(),
            title_widget: INVALID_UID,
            collapse_icon_widget: INVALID_UID,
            is_collapsible: true,
            is_collapsed: false,
            is_dirty: true,
        }
    }
}

impl TitleBar {
    pub fn collapsible(&mut self, can_collapse: bool) -> &mut Self {
        self.is_collapsible = can_collapse;
        self
    }
    pub fn is_collapsed(&self) -> bool {
        self.is_collapsed
    }
    fn collapse(&mut self) -> &mut Self {
        if !self.is_collapsed {
            self.is_collapsed = true;
            self.is_dirty = true;
        }
        self
    }
    fn expand(&mut self) -> &mut Self {
        if self.is_collapsed {
            self.is_collapsed = false;
            self.is_dirty = true;
        }
        self
    }

    fn change_collapse_icon(&mut self, shared_data: &SharedDataRw) -> &mut Self {
        if !self.is_dirty || !self.is_collapsible {
            return self;
        }
        self.is_dirty = false;

        let (vertices, indices) = if self.is_collapsed {
            create_triangle_down()
        } else {
            create_triangle_up()
        };
        let mut mesh_data = MeshData::default();
        mesh_data.append_mesh(&vertices, &indices);

        let icon_id = self.collapse_icon_widget;
        if let Some(collapse_icon) = self.get_data_mut().node.get_child::<Canvas>(icon_id) {
            collapse_icon
                .get_data_mut()
                .graphics
                .set_mesh_data(shared_data, mesh_data);
        }
        self
    }

    fn create_collapse_icon(&mut self, shared_data: &SharedDataRw) -> &mut Self {
        if self.is_collapsible {
            let icon_size: Vector2 = DEFAULT_ICON_SIZE.into();
            let mut collapse_icon = Canvas::default();
            collapse_icon.init(shared_data);
            collapse_icon
                .size(icon_size * Screen::get_scale_factor())
                .vertical_alignment(VerticalAlignment::Center)
                .horizontal_alignment(HorizontalAlignment::Left)
                .selectable(true)
                .style(WidgetStyle::FullActive);
            self.collapse_icon_widget = self.add_child(Box::new(collapse_icon));

            self.change_collapse_icon(shared_data);
        }

        self
    }

    fn manage_interactions(&mut self, shared_data: &SharedDataRw) -> &mut Self {
        let read_data = shared_data.read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        {
            let events = events_rw.read().unwrap();
            if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
                for event in widget_events.iter() {
                    if let WidgetEvent::Pressed(widget_id, _mouse_in_px) = event {
                        if self.id() == *widget_id || self.collapse_icon_widget == *widget_id {
                            if self.is_collapsed {
                                self.expand();
                            } else {
                                self.collapse();
                            }
                        }
                    }
                }
            }
        }
        if self.is_dirty {
            let mut events = events_rw.write().unwrap();
            if self.is_collapsed {
                events.send_event(TitleBarEvent::Collapsed(self.id()));
            } else {
                events.send_event(TitleBarEvent::Expanded(self.id()));
            }
        }
        self
    }
}

impl InternalWidget for TitleBar {
    fn widget_init(&mut self, shared_data: &SharedDataRw) {
        if self.is_initialized() {
            return;
        }

        let size: Vector2 = [400., DEFAULT_WIDGET_WIDTH as _].into();

        self.position(Screen::get_center() - size / 2.)
            .size(size * Screen::get_scale_factor())
            .keep_fixed_height(true)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .space_between_elements(10)
            .use_space_before_and_after(true)
            .draggable(false)
            .selectable(false)
            .style(WidgetStyle::DefaultTitleBar);

        self.create_collapse_icon(shared_data);

        let mut title = Text::default();
        title.init(shared_data);
        title
            .draggable(false)
            .selectable(false)
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center);
        title.set_text("Title");

        self.title_widget = self.add_child(Box::new(title));
    }

    fn widget_update(&mut self, shared_data: &SharedDataRw) {
        self.manage_interactions(shared_data)
            .change_collapse_icon(shared_data);
    }

    fn widget_uninit(&mut self, _shared_data: &SharedDataRw) {}
}
