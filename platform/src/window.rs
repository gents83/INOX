use std::any::TypeId;

use crate::{handle::*, KeyEvent, KeyTextEvent, MouseEvent};
use nrg_messenger::{implement_message, read_messages, MessageChannel, MessengerRw};

pub const DEFAULT_DPI: f32 = 96.0;

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum WindowEvent {
    None,
    DpiChanged(f32, f32),
    SizeChanged(u32, u32),
    PosChanged(u32, u32),
    RequestChangePos(u32, u32),
    RequestChangeSize(u32, u32),
    Close,
}
implement_message!(WindowEvent);

pub struct Window {
    handle: Handle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    scale_factor: f32,
    message_channel: MessageChannel,
    can_continue: bool,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Window {
    pub fn create(
        title: String,
        x: u32,
        y: u32,
        mut width: u32,
        mut height: u32,
        global_messenger: MessengerRw,
    ) -> Self {
        let mut global_dispatcher = global_messenger.write().unwrap();

        global_dispatcher.register_type::<WindowEvent>();
        global_dispatcher.register_type::<KeyEvent>();
        global_dispatcher.register_type::<KeyTextEvent>();
        global_dispatcher.register_type::<MouseEvent>();

        let message_channel = MessageChannel::default();
        global_dispatcher.register_messagebox::<WindowEvent>(message_channel.get_messagebox());

        let mut scale_factor = 1.0;
        let handle = Window::create_handle(
            title,
            x,
            y,
            &mut width,
            &mut height,
            &mut scale_factor,
            global_dispatcher.get_dispatcher(),
        );
        Self {
            handle,
            x,
            y,
            width,
            height,
            scale_factor,
            message_channel,
            can_continue: true,
        }
    }

    #[inline]
    pub fn get_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[inline]
    pub fn get_x(&self) -> u32 {
        self.x
    }
    #[inline]
    pub fn get_y(&self) -> u32 {
        self.y
    }

    #[inline]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn get_heigth(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }

    pub fn update(&mut self) -> bool {
        Window::internal_update(&self.handle);
        self.manage_window_events();
        self.can_continue
    }

    fn manage_window_events(&mut self) {
        let mut scale_factor = self.scale_factor;
        let mut can_continue = self.can_continue;
        let mut width = self.width;
        let mut height = self.height;
        let mut x = self.x;
        let mut y = self.y;

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<WindowEvent>() {
                let e = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match *e {
                    WindowEvent::DpiChanged(new_x, _y) => {
                        scale_factor = new_x / DEFAULT_DPI;
                    }
                    WindowEvent::SizeChanged(new_width, new_height) => {
                        width = new_width;
                        height = new_height;
                    }
                    WindowEvent::PosChanged(new_x, new_y) => {
                        x = new_x;
                        y = new_y;
                    }
                    WindowEvent::Close => {
                        can_continue = false;
                    }
                    WindowEvent::RequestChangePos(new_x, new_y) => {
                        Window::change_position(&self.handle, new_x, new_y);
                    }
                    WindowEvent::RequestChangeSize(new_width, new_height) => {
                        Window::change_size(&self.handle, new_width, new_height);
                    }
                    WindowEvent::None => {}
                }
            }
        });

        self.scale_factor = scale_factor;
        self.can_continue = can_continue;
        self.width = width;
        self.height = height;
        self.x = x;
        self.y = y;
    }
}
