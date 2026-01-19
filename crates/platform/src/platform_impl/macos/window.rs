#![cfg(target_os = "macos")]

use crate::handle::Handle;
use crate::platform_impl::platform::handle::HandleImpl;
use crate::window::{Window, WindowEvent};
use crate::{Key, MouseButton, MouseState, InputState, KeyEvent, MouseEvent};
use inox_messenger::MessageHubRc;
use std::path::Path;
use std::ptr::{self, addr_of_mut};

use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use cocoa::appkit::{
    NSApp, NSApplication, NSWindow, NSWindowStyleMask, NSBackingStoreType, NSRunningApplication,
    NSApplicationActivationPolicy, NSEvent, NSEventType
};
use objc::{msg_send, sel, sel_impl};

static mut EVENTS_DISPATCHER: Option<MessageHubRc> = None;

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

            let _pool = NSAutoreleasePool::new(nil);
            let app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);

            let rect = NSRect::new(NSPoint::new(x as f64, y as f64), NSSize::new(*width as f64, *height as f64));
            let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                rect,
                NSWindowStyleMask::NSTitledWindowMask | NSWindowStyleMask::NSClosableWindowMask | NSWindowStyleMask::NSResizableWindowMask | NSWindowStyleMask::NSMiniaturizableWindowMask,
                NSBackingStoreType::NSBackingStoreBuffered,
                false
            );
            window.setTitle_(NSString::alloc(nil).init_str(&title));
            window.makeKeyAndOrderFront_(nil);
            let view = window.contentView();

            *scale_factor = window.backingScaleFactor() as f32;

            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(cocoa::appkit::NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps);

            Handle {
                handle_impl: HandleImpl {
                    ns_window: window as _,
                    ns_view: view as _,
                }
            }
        }
    }

    pub fn change_title(handle: &Handle, title: &str) {
        unsafe {
            let window = handle.handle_impl.ns_window as id;
            window.setTitle_(NSString::alloc(nil).init_str(title));
        }
    }

    pub fn change_visibility(handle: &Handle, is_visible: bool) {
        unsafe {
            let window = handle.handle_impl.ns_window as id;
            if is_visible {
                window.setIsVisible_(true);
                if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                    events_dispatcher.send_event(WindowEvent::Show);
                }
            } else {
                window.setIsVisible_(false);
                if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                    events_dispatcher.send_event(WindowEvent::Hide);
                }
            }
        }
    }

    pub fn change_position(handle: &Handle, x: u32, y: u32) {
        unsafe {
            let window = handle.handle_impl.ns_window as id;
            let mut frame = window.frame();
            frame.origin.x = x as f64;
            // MacOS coordinates are bottom-left, typically. But NSWindow setFrameTopLeftPoint might be better or converting.
            // For simplicity assume simple setting. Correct implementation needs screen height awareness for Y flip if needed.
            frame.origin.y = y as f64;
            window.setFrame_display_(frame, true);
        }
    }

    pub fn change_size(handle: &Handle, width: u32, height: u32) {
        unsafe {
            let window = handle.handle_impl.ns_window as id;
            let mut frame = window.frame();
            frame.size.width = width as f64;
            frame.size.height = height as f64;
            window.setFrame_display_(frame, true);
        }
    }

    pub fn internal_update(handle: &Handle) -> bool {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let app = NSApp();

            loop {
                let event = app.nextEventMatchingMask_untilDate_inMode_dequeue_(
                    cocoa::appkit::NSEventMask::NSAnyEventMask.bits(),
                    nil,
                    cocoa::foundation::NSDefaultRunLoopMode,
                    true
                );

                if event == nil {
                    break;
                }

                let event_type = event.eventType();
                match event_type {
                     NSEventType::NSKeyDown | NSEventType::NSKeyUp => {
                        let key_code = event.keyCode();
                        let state = if event_type == NSEventType::NSKeyDown {
                            InputState::Pressed
                        } else {
                            InputState::Released
                        };
                        let key = convert_key(key_code as _);
                        if let Some(events_dispatcher) = &mut *addr_of_mut!(EVENTS_DISPATCHER) {
                            events_dispatcher.send_event(KeyEvent {
                                code: key,
                                state,
                            });
                        }
                     }
                     NSEventType::NSLeftMouseDown | NSEventType::NSRightMouseDown | NSEventType::NSOtherMouseDown | NSEventType::NSLeftMouseUp | NSEventType::NSRightMouseUp | NSEventType::NSOtherMouseUp => {
                        let button = match event_type {
                            NSEventType::NSLeftMouseDown | NSEventType::NSLeftMouseUp => MouseButton::Left,
                            NSEventType::NSRightMouseDown | NSEventType::NSRightMouseUp => MouseButton::Right,
                            NSEventType::NSOtherMouseDown | NSEventType::NSOtherMouseUp => MouseButton::Middle,
                            _ => MouseButton::None,
                        };
                        let state = match event_type {
                            NSEventType::NSLeftMouseDown | NSEventType::NSRightMouseDown | NSEventType::NSOtherMouseDown => MouseState::Down,
                            _ => MouseState::Up,
                        };
                        let location = event.locationInWindow();
                        let window = handle.handle_impl.ns_window as id;
                        let content_rect = window.contentRectForFrameRect_(window.frame());
                        let width = content_rect.size.width as f32;
                        let height = content_rect.size.height as f32;
                        let x = location.x as f64;
                        let y = height as f64 - location.y as f64;

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
                     NSEventType::NSMouseMoved | NSEventType::NSLeftMouseDragged | NSEventType::NSRightMouseDragged | NSEventType::NSOtherMouseDragged => {
                        let location = event.locationInWindow();
                        let window = handle.handle_impl.ns_window as id;
                        let content_rect = window.contentRectForFrameRect_(window.frame());
                        let width = content_rect.size.width as f32;
                        let height = content_rect.size.height as f32;
                        let x = location.x as f64;
                        let y = height as f64 - location.y as f64;

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

                app.sendEvent_(event);
            }

            let _: () = msg_send![pool, release];
        }
        true
    }
}

pub fn convert_key(key_code: u16) -> Key {
    match key_code {
        0x00 => Key::A,
        0x01 => Key::S,
        0x02 => Key::D,
        0x03 => Key::F,
        0x04 => Key::H,
        0x05 => Key::G,
        0x06 => Key::Z,
        0x07 => Key::X,
        0x08 => Key::C,
        0x09 => Key::V,
        0x0B => Key::B,
        0x0C => Key::Q,
        0x0D => Key::W,
        0x0E => Key::E,
        0x0F => Key::R,
        0x10 => Key::Y,
        0x11 => Key::T,
        0x12 => Key::Key1,
        0x13 => Key::Key2,
        0x14 => Key::Key3,
        0x15 => Key::Key4,
        0x16 => Key::Key6,
        0x17 => Key::Key5,
        0x18 => Key::Equal,
        0x19 => Key::Key9,
        0x1A => Key::Key7,
        0x1B => Key::Minus,
        0x1C => Key::Key8,
        0x1D => Key::Key0,
        0x1E => Key::RightBracket,
        0x1F => Key::O,
        0x20 => Key::U,
        0x21 => Key::LeftBracket,
        0x22 => Key::I,
        0x23 => Key::P,
        0x24 => Key::Enter,
        0x25 => Key::L,
        0x26 => Key::J,
        0x27 => Key::Quote,
        0x28 => Key::K,
        0x29 => Key::Semicolon,
        0x2A => Key::Backslash,
        0x2B => Key::Comma,
        0x2C => Key::Slash,
        0x2D => Key::N,
        0x2E => Key::M,
        0x2F => Key::Period,
        0x30 => Key::Tab,
        0x31 => Key::Space,
        0x32 => Key::GraveAccent,
        0x33 => Key::Backspace,
        0x35 => Key::Escape,
        0x37 => Key::Meta, // Command
        0x38 => Key::Shift,
        0x39 => Key::CapsLock,
        0x3A => Key::Alt,
        0x3B => Key::Control,
        0x7B => Key::ArrowLeft,
        0x7C => Key::ArrowRight,
        0x7D => Key::ArrowDown,
        0x7E => Key::ArrowUp,
        _ => Key::Unidentified,
    }
}
