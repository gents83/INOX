use std::any::TypeId;

use nrg_math::Vector2;
use nrg_messenger::Message;
use nrg_platform::{MouseEvent, MouseState};
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, InternalWidget, Panel, WidgetData, WidgetEvent,
    DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
};

pub const DEFAULT_SCROLLBAR_SIZE: [f32; 2] = [DEFAULT_WIDGET_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Scrollbar {
    data: WidgetData,
    percentage: f32,
    cursor: Uid,
}
implement_widget_with_custom_members!(Scrollbar {
    cursor: INVALID_UID,
    percentage: 0.
});

impl Scrollbar {
    pub fn horizontal(&mut self) -> &mut Self {
        self.horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Bottom);
        let cursor_uid = self.cursor;
        if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
            cursor
                .horizontal_alignment(HorizontalAlignment::None)
                .vertical_alignment(VerticalAlignment::Center);
        }
        self.percentage = 0.;
        self
    }
    pub fn vertical(&mut self) -> &mut Self {
        self.horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Stretch);
        let cursor_uid = self.cursor;
        if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
            cursor
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::None);
        }
        self.percentage = 0.;
        self
    }

    fn manage_cursor_interaction(&mut self) -> &mut Self {
        if self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch {
            let cursor_uid = self.cursor;
            let pos = self.state().get_position();
            let size = self.state().get_size();
            if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
                let mut cursor_pos = cursor.state().get_position();
                cursor_pos.y = pos.y;
                let mut cursor_size = size;
                cursor_size.x /= 10.;
                cursor.set_position(cursor_pos);
                cursor.set_size(cursor_size);
            }
        } else if self.state().get_vertical_alignment() == VerticalAlignment::Stretch {
            let cursor_uid = self.cursor;
            let pos = self.state().get_position();
            let size = self.state().get_size();
            if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
                let mut cursor_pos = cursor.state().get_position();
                cursor_pos.x = pos.x;
                let mut cursor_size = size;
                cursor_size.y /= 10.;
                cursor.set_position(cursor_pos);
                cursor.set_size(cursor_size);
            }
        }
        self
    }

    fn compute_percentage_from_cursor(&mut self) -> &mut Self {
        let cursor_uid = self.cursor;
        let pos = self.state().get_position();
        let size = self.state().get_size();
        let mut cursor_pos = pos;
        let mut cursor_size = size;
        if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
            cursor_pos = cursor.state().get_position();
            cursor_size = cursor.state().get_size();
        }
        self.percentage = if self.state().get_horizontal_alignment() == HorizontalAlignment::Stretch
        {
            cursor_pos.x / (pos.x + (size.x - cursor_size.x))
        } else if self.state().get_vertical_alignment() == VerticalAlignment::Stretch {
            cursor_pos.y / (pos.y + (size.y - cursor_size.y))
        } else {
            self.percentage
        };
        self
    }

    fn compute_cursor_from_percentage(&mut self) -> &mut Self {
        let cursor_uid = self.cursor;
        let pos = self.state().get_position();
        let size = self.state().get_size();
        let percentage = self.percentage;
        let horizontal_alignment = self.state().get_horizontal_alignment();
        let vertical_alignment = self.state().get_vertical_alignment();
        if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
            let mut cursor_pos = cursor.state().get_position();
            let cursor_size = cursor.state().get_size();
            if horizontal_alignment == HorizontalAlignment::Stretch {
                cursor_pos.x = percentage * (pos.x + (size.x - cursor_size.x)) - cursor_size.x / 2.;
            } else if vertical_alignment == VerticalAlignment::Stretch {
                cursor_pos.y = percentage * (pos.y + (size.y - cursor_size.y)) - cursor_size.y / 2.;
            }
            cursor.set_position(cursor_pos);
        }
        self
    }
}

impl InternalWidget for Scrollbar {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

        if self.is_initialized() {
            return;
        }

        let size: Vector2 = DEFAULT_SCROLLBAR_SIZE.into();
        self.position(Screen::get_center() - size / 2.)
            .size(size * Screen::get_scale_factor())
            .selectable(true)
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Stretch)
            .style(WidgetStyle::Default);

        let mut cursor = Panel::new(self.get_shared_data(), self.get_global_messenger());
        cursor
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Top)
            .style(WidgetStyle::FullHighlight)
            .selectable(true)
            .draggable(true)
            .size(size);

        self.cursor = self.add_child(Box::new(cursor));

        self.vertical();
    }

    fn widget_update(&mut self) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>();
    }
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            match *event {
                WidgetEvent::InvalidateLayout(widget_id) => {
                    if widget_id == self.id() {
                        self.manage_cursor_interaction();
                    }
                }
                WidgetEvent::Dragging(widget_id, _mouse_pos_in_px) => {
                    if widget_id == self.cursor {
                        self.compute_percentage_from_cursor();
                    }
                }
                _ => {}
            }
        } else if msg.type_id() == TypeId::of::<MouseEvent>() {
            let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
            if event.state == MouseState::Down {
                let mouse_pos: Vector2 = [event.x as _, event.y as _].into();
                if self.state().is_inside(mouse_pos) {
                    let cursor_uid = self.cursor;
                    let pos = self.state().get_position();
                    let size = self.state().get_size();
                    let mut cursor_size = size;
                    if let Some(cursor) = self.node_mut().get_child::<Panel>(cursor_uid) {
                        cursor_size = cursor.state().get_size();
                    }
                    self.percentage = if self.state().get_horizontal_alignment()
                        == HorizontalAlignment::Stretch
                    {
                        mouse_pos.x / (pos.x + (size.x - cursor_size.x))
                    } else if self.state().get_vertical_alignment() == VerticalAlignment::Stretch {
                        mouse_pos.y / (pos.y + (size.y - cursor_size.y))
                    } else {
                        self.percentage
                    };
                    self.compute_cursor_from_percentage();
                }
            }
        }
    }
}
