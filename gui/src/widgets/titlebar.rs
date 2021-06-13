use std::{any::TypeId, path::Path};

use nrg_graphics::{
    utils::{create_triangle_down, create_triangle_right},
    MeshData,
};
use nrg_math::{Vector2, Vector4};
use nrg_messenger::{implement_message, Message};
use nrg_platform::MouseEvent;
use nrg_serialize::{Deserialize, Serialize, Uid, INVALID_UID};

use crate::{
    implement_widget_with_custom_members, Button, Canvas, InternalWidget, Text, WidgetData,
    WidgetEvent, DEFAULT_TEXT_SIZE, DEFAULT_WIDGET_HEIGHT, DEFAULT_WIDGET_WIDTH,
};

pub const ICON_SCALE: f32 = 0.75;
pub const DEFAULT_ICON_SIZE: [f32; 2] = [
    DEFAULT_WIDGET_WIDTH * 2. / 3.,
    DEFAULT_WIDGET_HEIGHT * 2. / 3.,
];

#[derive(Clone, Copy)]
pub enum TitleBarEvent {
    Collapsed(Uid),
    Expanded(Uid),
    Close(Uid),
}
implement_message!(TitleBarEvent);

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct TitleBar {
    data: WidgetData,
    title_widget: Uid,
    can_be_closed: bool,
    close_icon_widget: Uid,
    collapse_icon_widget: Uid,
    is_collapsible: bool,
    is_collapsed: bool,
    #[serde(skip)]
    is_dirty: bool,
}
implement_widget_with_custom_members!(TitleBar {
    title_widget: INVALID_UID,
    can_be_closed: false,
    close_icon_widget: INVALID_UID,
    collapse_icon_widget: INVALID_UID,
    is_collapsible: false,
    is_collapsed: false,
    is_dirty: true
});

impl TitleBar {
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        let uid = self.title_widget;
        if let Some(text_box) = self.node().get_child_mut::<Text>(uid) {
            text_box.set_text(text);
        }
        self
    }
    pub fn set_text_alignment(
        &mut self,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> &mut Self {
        let uid = self.title_widget;
        if let Some(text_box) = self.node().get_child_mut::<Text>(uid) {
            text_box
                .horizontal_alignment(horizontal_alignment)
                .vertical_alignment(vertical_alignment);
        }
        self
    }
    pub fn closeable(&mut self, can_be_closed: bool, texture_path: &Path) -> &mut Self {
        if self.can_be_closed != can_be_closed {
            self.can_be_closed = can_be_closed;
            if self.can_be_closed {
                self.create_close_icon(texture_path);
            } else {
                let icon_id = self.close_icon_widget;
                self.node_mut().remove_child(icon_id);
                self.close_icon_widget = INVALID_UID;
            }
        }
        self
    }
    pub fn collapsible(&mut self, can_collapse: bool) -> &mut Self {
        if self.is_collapsible != can_collapse {
            self.is_collapsible = can_collapse;
            if !self.is_collapsible {
                let icon_id = self.collapse_icon_widget;
                self.node_mut().remove_child(icon_id);
                self.collapse_icon_widget = INVALID_UID;
            } else {
                self.create_collapse_icon();
            }
        }
        self
    }
    pub fn is_collapsed(&self) -> bool {
        self.is_collapsed
    }
    pub fn collapse(&mut self) -> &mut Self {
        if !self.is_collapsed {
            self.is_collapsed = true;
            self.is_dirty = true;
        }
        self
    }
    pub fn expand(&mut self) -> &mut Self {
        if self.is_collapsed {
            self.is_collapsed = false;
            self.is_dirty = true;
        }
        self
    }

    pub fn change_collapse_icon(&mut self) -> &mut Self {
        if !self.is_dirty || !self.is_collapsible {
            return self;
        }
        self.is_dirty = false;

        let (vertices, indices) = if self.is_collapsed {
            create_triangle_right()
        } else {
            create_triangle_down()
        };
        let mut mesh_data = MeshData::default();
        mesh_data.append_mesh(&vertices, &indices);

        let icon_id = self.collapse_icon_widget;
        if let Some(collapse_icon) = self.node().get_child_mut::<Canvas>(icon_id) {
            collapse_icon.graphics_mut().set_mesh_data(mesh_data);
        }
        self
    }

    fn create_collapse_icon(&mut self) -> &mut Self {
        if self.is_collapsible {
            let icon_size: Vector2 = DEFAULT_ICON_SIZE.into();
            let mut collapse_icon =
                Canvas::new(self.get_shared_data(), self.get_global_messenger());
            collapse_icon
                .size(icon_size * Screen::get_scale_factor())
                .vertical_alignment(VerticalAlignment::Center)
                .horizontal_alignment(HorizontalAlignment::Left)
                .keep_fixed_height(true)
                .keep_fixed_width(true)
                .selectable(true)
                .style(WidgetStyle::DefaultText);
            self.collapse_icon_widget = self.add_child(Box::new(collapse_icon));

            self.change_collapse_icon();
        }

        self
    }

    fn create_close_icon(&mut self, texture_path: &Path) -> &mut Self {
        if self.can_be_closed {
            let icon_size: Vector2 = DEFAULT_ICON_SIZE.into();
            let mut close_icon = Button::new(self.get_shared_data(), self.get_global_messenger());

            close_icon
                .size(icon_size * Screen::get_scale_factor())
                .vertical_alignment(VerticalAlignment::Center)
                .horizontal_alignment(HorizontalAlignment::Right)
                .fill_type(ContainerFillType::None)
                .keep_fixed_width(true)
                .keep_fixed_height(true)
                .selectable(true)
                .with_text(" ")
                .with_texture(texture_path)
                .style(WidgetStyle::DefaultText);

            self.close_icon_widget = self.add_child(Box::new(close_icon));
        }

        self
    }
}

impl InternalWidget for TitleBar {
    fn widget_init(&mut self) {
        self.register_to_listen_event::<WidgetEvent>()
            .register_to_listen_event::<MouseEvent>();

        if self.is_initialized() {
            self.is_dirty = true;
            self.change_collapse_icon();

            return;
        }

        let size: Vector2 = [400., DEFAULT_WIDGET_HEIGHT].into();

        self.position(Screen::get_center() - size / 2.)
            .size(size * Screen::get_scale_factor())
            .keep_fixed_height(true)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::None)
            .draggable(false)
            .selectable(false)
            .style(WidgetStyle::DefaultTitleBar)
            .collapsible(true);

        let mut title = Text::new(self.get_shared_data(), self.get_global_messenger());
        let title_size: Vector2 = [DEFAULT_TEXT_SIZE[0], size.y].into();
        title
            .draggable(false)
            .selectable(false)
            .size(title_size * Screen::get_scale_factor())
            .vertical_alignment(VerticalAlignment::Center)
            .horizontal_alignment(HorizontalAlignment::Center);
        title.set_text("Title");

        self.title_widget = self.add_child(Box::new(title));
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {
        self.unregister_to_listen_event::<WidgetEvent>()
            .unregister_to_listen_event::<MouseEvent>();
    }
    fn widget_process_message(&mut self, msg: &dyn Message) {
        if msg.type_id() == TypeId::of::<WidgetEvent>() {
            let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
            if let WidgetEvent::Released(widget_id, _mouse_in_px) = *event {
                let events_dispatcher = self.get_global_dispatcher();
                if self.id() == widget_id || self.collapse_icon_widget == widget_id {
                    let titlebar_event = if self.is_collapsed {
                        self.expand();
                        TitleBarEvent::Expanded(self.id())
                    } else {
                        self.collapse();
                        TitleBarEvent::Collapsed(self.id())
                    };
                    self.change_collapse_icon();

                    events_dispatcher
                        .write()
                        .unwrap()
                        .send(titlebar_event.as_boxed())
                        .ok();
                } else if self.close_icon_widget == widget_id {
                    events_dispatcher
                        .write()
                        .unwrap()
                        .send(TitleBarEvent::Close(self.id()).as_boxed())
                        .ok();
                }
            }
        }
    }
}
