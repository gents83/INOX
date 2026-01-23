use std::sync::Arc;
use crate::launcher::Launcher;
use objc::runtime::{Object, Sel, Class};
use objc::{msg_send, sel, sel_impl, class};
use objc::declare::ClassDecl;

static mut LAUNCHER: Option<Arc<Launcher>> = None;

#[repr(C)]
struct CGPoint { x: f64, y: f64 }
#[repr(C)]
struct CGSize { width: f64, height: f64 }
#[repr(C)]
struct CGRect { origin: CGPoint, size: CGSize }

pub fn setup_env() {}

pub fn load_plugins(launcher: &Arc<Launcher>) {
    unsafe { LAUNCHER = Some(launcher.clone()); }
    // Defer plugin loading to didFinishLaunching
}

extern "C" fn did_finish_launching(this: &Object, _cmd: Sel, _app: *mut Object, _options: *mut Object) -> bool {
    unsafe {
        let launcher = LAUNCHER.as_ref().unwrap();

        let screen: *mut Object = msg_send![class!(UIScreen), mainScreen];
        let bounds: CGRect = msg_send![screen, bounds];

        let window: *mut Object = msg_send![class!(UIWindow), alloc];
        let window: *mut Object = msg_send![window, initWithFrame: bounds];

        let vc: *mut Object = msg_send![class!(UIViewController), new];
        let _: () = msg_send![window, setRootViewController: vc];

        let view: *mut Object = msg_send![vc, view];

        inox_platform::platform_impl::platform::set_ui_view(view as _);

        // Load plugins now
        let context = launcher.context();
        launcher.add_dynamic_plugin("inox_viewer", std::path::Path::new(""));

        let _: () = msg_send![window, makeKeyAndVisible];

        // Setup display link
        let display_link: *mut Object = msg_send![class!(CADisplayLink), displayLinkWithTarget:this selector:sel!(updateLoop:)];
        let loop_mode = msg_send![class!(NSString), stringWithUTF8String:"kCFRunLoopDefaultMode\0".as_ptr()];
        let run_loop = msg_send![class!(NSRunLoop), mainRunLoop];
        let _: () = msg_send![display_link, addToRunLoop:run_loop forMode:loop_mode];
    }
    true
}

extern "C" fn update_loop(_this: &Object, _cmd: Sel, _sender: *mut Object) {
    unsafe {
        if let Some(launcher) = LAUNCHER.as_ref() {
            launcher.update();
        }
    }
}

pub fn main_update(_launcher: Arc<Launcher>) {
    unsafe {
        let autorelease_pool: *mut Object = msg_send![class!(NSAutoreleasePool), new];
        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];

        let superclass = class!(UIResponder);
        let mut decl = ClassDecl::new("AppDelegate", superclass).unwrap();

        decl.add_method(sel!(application:didFinishLaunchingWithOptions:),
            did_finish_launching as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> bool);
        decl.add_method(sel!(updateLoop:),
            update_loop as extern "C" fn(&Object, Sel, *mut Object));

        decl.register();

        let delegate_class = class!(AppDelegate);
        let delegate_instance: *mut Object = msg_send![delegate_class, new];

        let _: () = msg_send![app, setDelegate: delegate_instance];
        let _: () = msg_send![app, run];

        let _: () = msg_send![autorelease_pool, drain];
    }
}
