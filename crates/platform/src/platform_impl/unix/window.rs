#![cfg(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd", target_os = "openbsd"))]

use crate::handle::Handle;
use crate::platform_impl::platform::handle::HandleImpl;
use crate::window::{Window, WindowEvent};
use crate::{Key, MouseButton, MouseState, InputState, KeyEvent, KeyTextEvent, MouseEvent};
use inox_messenger::MessageHubRc;
use std::ffi::{CString};
use std::path::Path;
use std::ptr::{self, addr_of_mut};
use std::mem::MaybeUninit;
use std::os::raw::{c_int};
use x11_dl::xlib;

static mut XLIB: Option<xlib::Xlib> = None;
static mut ATOM_WM_DELETE_WINDOW: xlib::Atom = 0;
static mut EVENTS_DISPATCHER: Option<MessageHubRc> = None;

pub fn xlib() -> &'static xlib::Xlib {
    unsafe {
        if XLIB.is_none() {
            XLIB = Some(xlib::Xlib::open().expect("Failed to load Xlib"));
        }
        XLIB.as_ref().unwrap()
    }
}

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
        let xlib = xlib();
        unsafe {
            EVENTS_DISPATCHER = Some(events_dispatcher.clone());

            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                panic!("Failed to open X display");
            }
            let screen = (xlib.XDefaultScreen)(display);
            let root = (xlib.XRootWindow)(display, screen);

            let mut attributes: xlib::XSetWindowAttributes = MaybeUninit::zeroed().assume_init();
            attributes.background_pixel = (xlib.XWhitePixel)(display, screen);
            attributes.event_mask = xlib::ExposureMask | xlib::KeyPressMask | xlib::KeyReleaseMask | xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask | xlib::StructureNotifyMask;

            let window = (xlib.XCreateWindow)(
                display,
                root,
                x as _,
                y as _,
                *width as _,
                *height as _,
                0,
                xlib::CopyFromParent,
                xlib::InputOutput,
                (xlib.XDefaultVisual)(display, screen),
                xlib::CWBackPixel | xlib::CWEventMask,
                &mut attributes,
            );

            let title_c = CString::new(title).unwrap();
            (xlib.XStoreName)(display, window, title_c.as_ptr());

            let wm_delete_window_str = CString::new("WM_DELETE_WINDOW").unwrap();
            ATOM_WM_DELETE_WINDOW = (xlib.XInternAtom)(display, wm_delete_window_str.as_ptr(), xlib::False);
            let mut protocols = [ATOM_WM_DELETE_WINDOW];
            (xlib.XSetWMProtocols)(display, window, protocols.as_mut_ptr(), 1);

            (xlib.XMapWindow)(display, window);
            (xlib.XFlush)(display);

            *scale_factor = 1.0;

            Handle {
                handle_impl: HandleImpl {
                    window,
                    display: display as _,
                },
            }
        }
    }

    pub fn change_title(handle: &Handle, title: &str) {
        let xlib = xlib();
        unsafe {
            let title_c = CString::new(title).unwrap();
            (xlib.XStoreName)(handle.handle_impl.display as _, handle.handle_impl.window, title_c.as_ptr());
            (xlib.XFlush)(handle.handle_impl.display as _);
        }
    }

    pub fn change_visibility(handle: &Handle, is_visible: bool) {
        let xlib = xlib();
        unsafe {
            if is_visible {
                (xlib.XMapWindow)(handle.handle_impl.display as _, handle.handle_impl.window);
                if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                    events_dispatcher.send_event(WindowEvent::Show);
                }
            } else {
                (xlib.XUnmapWindow)(handle.handle_impl.display as _, handle.handle_impl.window);
                if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                    events_dispatcher.send_event(WindowEvent::Hide);
                }
            }
            (xlib.XFlush)(handle.handle_impl.display as _);
        }
    }

    pub fn change_position(handle: &Handle, x: u32, y: u32) {
        let xlib = xlib();
        unsafe {
             (xlib.XMoveWindow)(handle.handle_impl.display as _, handle.handle_impl.window, x as _, y as _);
             (xlib.XFlush)(handle.handle_impl.display as _);
        }
    }

    pub fn change_size(handle: &Handle, width: u32, height: u32) {
        let xlib = xlib();
        unsafe {
             (xlib.XResizeWindow)(handle.handle_impl.display as _, handle.handle_impl.window, width as _, height as _);
             (xlib.XFlush)(handle.handle_impl.display as _);
        }
    }

    pub fn internal_update(handle: &Handle) -> bool {
        let xlib = xlib();
        unsafe {
            let display = handle.handle_impl.display as *mut xlib::Display;
            while (xlib.XPending)(display) > 0 {
                let mut event: xlib::XEvent = MaybeUninit::zeroed().assume_init();
                (xlib.XNextEvent)(display, &mut event);

                match event.type_ {
                    xlib::ClientMessage => {
                         if event.client_message.data.l[0] as xlib::Atom == ATOM_WM_DELETE_WINDOW {
                             if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                                events_dispatcher.send_event(WindowEvent::Close);
                             }
                         }
                    }
                    xlib::ConfigureNotify => {
                        let width = event.configure.width as u32;
                        let height = event.configure.height as u32;
                        let x = event.configure.x as u32;
                        let y = event.configure.y as u32;
                        if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                            events_dispatcher.send_event(WindowEvent::SizeChanged(width, height));
                            events_dispatcher.send_event(WindowEvent::PosChanged(x, y));
                        }
                    }
                    xlib::KeyPress | xlib::KeyRelease => {
                        // Minimal key handling
                        let key_sym = (xlib.XLookupKeysym)(&mut event.key, 0);
                        let state = if event.type_ == xlib::KeyPress { InputState::Pressed } else { InputState::Released };
                        let key = convert_key(key_sym as _);
                        if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                            events_dispatcher.send_event(KeyEvent {
                                code: key,
                                state,
                            });
                        }
                    }
                    xlib::ButtonPress | xlib::ButtonRelease => {
                        let button = match event.button.button {
                            1 => MouseButton::Left,
                            2 => MouseButton::Middle,
                            3 => MouseButton::Right,
                            _ => MouseButton::None,
                        };
                        let state = if event.type_ == xlib::ButtonPress { MouseState::Down } else { MouseState::Up };

                        // We need width/height for normalization
                        let mut attrs: xlib::XWindowAttributes = MaybeUninit::zeroed().assume_init();
                        (xlib.XGetWindowAttributes)(display, handle.handle_impl.window, &mut attrs);
                        let width = attrs.width as f32;
                        let height = attrs.height as f32;

                        let x = event.button.x as f64;
                        let y = event.button.y as f64;

                        if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                            events_dispatcher.send_event(MouseEvent {
                                x,
                                y,
                                normalized_x: x as f32 / width,
                                normalized_y: y as f32 / height,
                                button,
                                state,
                            });
                        }
                    }
                     xlib::MotionNotify => {
                        let mut attrs: xlib::XWindowAttributes = MaybeUninit::zeroed().assume_init();
                        (xlib.XGetWindowAttributes)(display, handle.handle_impl.window, &mut attrs);
                        let width = attrs.width as f32;
                        let height = attrs.height as f32;
                        let x = event.motion.x as f64;
                        let y = event.motion.y as f64;
                        if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                            events_dispatcher.send_event(MouseEvent {
                                x,
                                y,
                                normalized_x: x as f32 / width,
                                normalized_y: y as f32 / height,
                                button: MouseButton::None,
                                state: MouseState::Move,
                            });
                        }
                     }
                    _ => {}
                }
            }
        }
        true
    }
}

pub fn convert_key(keysym: c_int) -> Key {
    // This is a very simplified mapping
    match keysym as u32 {
        0x0061 => Key::A,
        0x0062 => Key::B,
        0x0063 => Key::C,
        0x0064 => Key::D,
        0x0065 => Key::E,
        0x0066 => Key::F,
        0x0067 => Key::G,
        0x0068 => Key::H,
        0x0069 => Key::I,
        0x006a => Key::J,
        0x006b => Key::K,
        0x006c => Key::L,
        0x006d => Key::M,
        0x006e => Key::N,
        0x006f => Key::O,
        0x0070 => Key::P,
        0x0071 => Key::Q,
        0x0072 => Key::R,
        0x0073 => Key::S,
        0x0074 => Key::T,
        0x0075 => Key::U,
        0x0076 => Key::V,
        0x0077 => Key::W,
        0x0078 => Key::X,
        0x0079 => Key::Y,
        0x007a => Key::Z,
        0xFF1B => Key::Escape,
        0xFF0D => Key::Enter,
        0xFF09 => Key::Tab,
        0xFF08 => Key::Backspace,
        0xFF63 => Key::Insert,
        0xFFFF => Key::Delete,
        0xFF51 => Key::ArrowRight,
        0xFF52 => Key::ArrowUp,
        0xFF53 => Key::ArrowLeft,
        0xFF54 => Key::ArrowDown,
        0xFF50 => Key::Home,
        0xFF57 => Key::End,
        0xFF55 => Key::PageUp,
        0xFF56 => Key::PageDown,
        0xFFBE => Key::F1,
        0xFFBF => Key::F2,
        0xFFC0 => Key::F3,
        0xFFC1 => Key::F4,
        0xFFC2 => Key::F5,
        0xFFC3 => Key::F6,
        0xFFC4 => Key::F7,
        0xFFC5 => Key::F8,
        0xFFC6 => Key::F9,
        0xFFC7 => Key::F10,
        0xFFC8 => Key::F11,
        0xFFC9 => Key::F12,
        0xFFE1 => Key::Shift,
        0xFFE2 => Key::Shift,
        0xFFE3 => Key::Control,
        0xFFE4 => Key::Control,
        0xFFE9 => Key::Alt,
        0xFFEA => Key::Alt,
        0x0020 => Key::Space,
        _ => Key::Unidentified,
    }
}
