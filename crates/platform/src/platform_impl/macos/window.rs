#![cfg(target_os = "macos")]

use crate::handle::{Handle, HandleImpl};
use crate::window::{Window, WindowEvent};
use crate::{Key, MouseButton, MouseState, InputState, KeyEvent, KeyTextEvent, MouseEvent};
use inox_messenger::MessageHubRc;
use std::path::Path;
use std::ptr::{self, addr_of_mut};

use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use cocoa::appkit::{
    NSApp, NSApplication, NSWindow, NSWindowStyleMask, NSBackingStoreType, NSRunningApplication,
    NSApplicationActivationPolicy, NSEvent, NSEventType, NSEventModifierFlags
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
                         // convert key
                         // send KeyEvent
                     }
                     NSEventType::NSLeftMouseDown | NSEventType::NSRightMouseDown | NSEventType::NSOtherMouseDown => {
                         // send MouseEvent
                     }
                     // ...
                     _ => {}
                }

                app.sendEvent_(event);
            }

            let _: () = msg_send![pool, release];
        }
        true
    }
}

pub fn convert_key(key: i32) -> Key {
    Key::Unidentified // TODO: implement
}
