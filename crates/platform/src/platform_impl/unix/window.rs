use std::path::Path;
use std::ptr::null_mut;
use std::ptr::addr_of_mut;

use inox_messenger::MessageHubRc;
use x11_dl::xlib::{
    Display, Xlib, Atom,
};
use crate::handle::Handle;
use crate::window::*;
use super::handle::HandleImpl;

// Constants missing in x11-dl or needing explicit definition
const CLIENT_MESSAGE: i32 = 33;
const KEY_PRESS: i32 = 2;
const KEY_RELEASE: i32 = 3;
const BUTTON_PRESS: i32 = 4;
const BUTTON_RELEASE: i32 = 5;
const MOTION_NOTIFY: i32 = 6;
const KEY_PRESS_MASK: i64 = 1 << 0;
const KEY_RELEASE_MASK: i64 = 1 << 1;
const BUTTON_PRESS_MASK: i64 = 1 << 2;
const BUTTON_RELEASE_MASK: i64 = 1 << 3;
const POINTER_MOTION_MASK: i64 = 1 << 6;
const STRUCTURE_NOTIFY_MASK: i64 = 1 << 17;
const EXPOSURE_MASK: i64 = 1 << 15;
const FOCUS_CHANGE_MASK: i64 = 1 << 21;

static mut XLIB: Option<Xlib> = None;
static mut EVENTS_DISPATCHER: Option<MessageHubRc> = None;
static mut WM_DELETE_WINDOW: Atom = 0;
static mut WM_PROTOCOLS: Atom = 0;

impl Window {
    pub fn create_handle(
        title: String,
        x: u32,
        y: u32,
        width: &mut u32,
        height: &mut u32,
        scale_factor: &mut f32,
        _icon_path: &Path,
        events_dispatcher: &MessageHubRc,
    ) -> Handle {
        unsafe {
            EVENTS_DISPATCHER = Some(events_dispatcher.clone());

            if XLIB.is_none() {
                let xlib = Xlib::open().expect("failed to load xlib");
                XLIB = Some(xlib);
            }
            let xlib = XLIB.as_ref().unwrap();

            let display = (xlib.XOpenDisplay)(null_mut());
            if display.is_null() {
                panic!("failed to open display");
            }

            let screen = (xlib.XDefaultScreen)(display);
            let root = (xlib.XRootWindow)(display, screen);

            *scale_factor = 1.0;

            let mut w = *width;
            let mut h = *height;

            let window = (xlib.XCreateSimpleWindow)(
                display,
                root,
                x as _,
                y as _,
                w,
                h,
                1,
                0,
                0,
            );

            let title_c = std::ffi::CString::new(title).unwrap();
            (xlib.XStoreName)(display, window, title_c.as_ptr());

            let mask = KEY_PRESS_MASK
                | KEY_RELEASE_MASK
                | BUTTON_PRESS_MASK
                | BUTTON_RELEASE_MASK
                | POINTER_MOTION_MASK
                | STRUCTURE_NOTIFY_MASK
                | EXPOSURE_MASK
                | FOCUS_CHANGE_MASK;
            (xlib.XSelectInput)(display, window, mask);

            let wm_protocols_str = std::ffi::CString::new("WM_PROTOCOLS").unwrap();
            let wm_delete_window_str = std::ffi::CString::new("WM_DELETE_WINDOW").unwrap();
            WM_PROTOCOLS = (xlib.XInternAtom)(display, wm_protocols_str.as_ptr(), 0);
            WM_DELETE_WINDOW = (xlib.XInternAtom)(display, wm_delete_window_str.as_ptr(), 0);
            let mut protocols = [WM_DELETE_WINDOW];
            (xlib.XSetWMProtocols)(display, window, protocols.as_mut_ptr(), 1);

            Handle {
                handle_impl: HandleImpl {
                    window,
                    display: display as _,
                },
            }
        }
    }

    pub fn change_title(handle: &Handle, title: &str) {
        unsafe {
             if let Some(xlib) = XLIB.as_ref() {
                let display = handle.handle_impl.display as *mut Display;
                let title_c = std::ffi::CString::new(title).unwrap();
                (xlib.XStoreName)(display, handle.handle_impl.window, title_c.as_ptr());
             }
        }
    }
    pub fn change_visibility(handle: &Handle, is_visible: bool) {
        unsafe {
             if let Some(xlib) = XLIB.as_ref() {
                let display = handle.handle_impl.display as *mut Display;
                if is_visible {
                    (xlib.XMapWindow)(display, handle.handle_impl.window);
                } else {
                    (xlib.XUnmapWindow)(display, handle.handle_impl.window);
                }
                (xlib.XFlush)(display);
             }
        }
    }
    pub fn change_position(handle: &Handle, x: u32, y: u32) {
         unsafe {
             if let Some(xlib) = XLIB.as_ref() {
                let display = handle.handle_impl.display as *mut Display;
                (xlib.XMoveWindow)(display, handle.handle_impl.window, x as _, y as _);
                (xlib.XFlush)(display);
             }
        }
    }
    pub fn change_size(handle: &Handle, width: u32, height: u32) {
         unsafe {
             if let Some(xlib) = XLIB.as_ref() {
                let display = handle.handle_impl.display as *mut Display;
                (xlib.XResizeWindow)(display, handle.handle_impl.window, width as _, height as _);
                (xlib.XFlush)(display);
             }
        }
    }

    #[inline]
    pub fn internal_update(handle: &Handle) -> bool {
        unsafe {
            if let Some(xlib) = XLIB.as_ref() {
                let display = handle.handle_impl.display as *mut Display;
                while (xlib.XPending)(display) > 0 {
                    let mut event = std::mem::zeroed();
                    (xlib.XNextEvent)(display, &mut event);

                    if event.type_ == CLIENT_MESSAGE {
                        let data_ptr = &event.client_message.data as *const _ as *const std::os::raw::c_long;
                        let atom = *data_ptr.offset(0) as u64;

                         if event.client_message.message_type == WM_PROTOCOLS
                            && event.client_message.format == 32
                            && atom == WM_DELETE_WINDOW {
                                if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                                    events_dispatcher.send_event(WindowEvent::Close);
                                }
                                return false;
                            }
                    } else if event.type_ == KEY_PRESS || event.type_ == KEY_RELEASE {
                        // TODO: Implement key handling
                    } else if event.type_ == BUTTON_PRESS || event.type_ == BUTTON_RELEASE {
                        // TODO: Implement mouse button handling
                    } else if event.type_ == MOTION_NOTIFY {
                        // TODO: Implement mouse motion handling
                    }
                }
            }
            true
        }
    }
}
