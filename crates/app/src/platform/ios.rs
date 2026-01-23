#![allow(unexpected_cfgs)]
use std::sync::Arc;
use crate::launcher::Launcher;
use objc::runtime::{Object, Sel};
use objc::{msg_send, sel, sel_impl, class};
use objc::declare::ClassDecl;
use std::sync::OnceLock;

static LAUNCHER: OnceLock<Arc<Launcher>> = OnceLock::new();

#[repr(C)]
struct CGPoint { x: f64, y: f64 }
#[repr(C)]
struct CGSize { width: f64, height: f64 }
#[repr(C)]
struct CGRect { origin: CGPoint, size: CGSize }

pub fn setup_env() {}

pub fn load_plugins(launcher: &Arc<Launcher>) {
    let _ = LAUNCHER.set(launcher.clone());
    // Defer plugin loading to didFinishLaunching
}

#[allow(unexpected_cfgs)]
extern "C" fn did_finish_launching(this: &Object, _cmd: Sel, _app: *mut Object, _options: *mut Object) -> bool {
    unsafe {
        let launcher = LAUNCHER.get().unwrap();

        #[allow(unexpected_cfgs)]
        let screen: *mut Object = msg_send![class!(UIScreen), mainScreen];
        #[allow(unexpected_cfgs)]
        let bounds: CGRect = msg_send![screen, bounds];

        #[allow(unexpected_cfgs)]
        let window: *mut Object = msg_send![class!(UIWindow), alloc];
        #[allow(unexpected_cfgs)]
        let window: *mut Object = msg_send![window, initWithFrame: bounds];

        #[allow(unexpected_cfgs)]
        let vc: *mut Object = msg_send![class!(UIViewController), new];
        #[allow(unexpected_cfgs)]
        let _: () = msg_send![window, setRootViewController: vc];

        #[allow(unexpected_cfgs)]
        let view: *mut Object = msg_send![vc, view];

        inox_platform::platform_impl::platform::set_ui_view(view as _);

        // Load plugins now
        let _context = launcher.context();
        launcher.add_dynamic_plugin("inox_viewer", std::path::Path::new(""));

        #[allow(unexpected_cfgs)]
        let _: () = msg_send![window, makeKeyAndVisible];

        // Setup display link
        #[allow(unexpected_cfgs)]
        let display_link: *mut Object = msg_send![class!(CADisplayLink), displayLinkWithTarget:this selector:sel!(updateLoop:)];
        #[allow(unexpected_cfgs)]
        let loop_mode: *mut Object = msg_send![class!(NSString), stringWithUTF8String:c"kCFRunLoopDefaultMode".as_ptr()];
        #[allow(unexpected_cfgs)]
        let run_loop: *mut Object = msg_send![class!(NSRunLoop), mainRunLoop];
        #[allow(unexpected_cfgs)]
        let _: () = msg_send![display_link, addToRunLoop:run_loop forMode:loop_mode];
    }
    true
}

extern "C" fn update_loop(_this: &Object, _cmd: Sel, _sender: *mut Object) {
    if let Some(launcher) = LAUNCHER.get() {
        launcher.update();
    }
}

pub fn main_update(_launcher: Arc<Launcher>) {
    unsafe {
        let autorelease_pool: *mut Object = msg_send![class!(NSAutoreleasePool), new];
        #[allow(unexpected_cfgs)]
        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];

        #[allow(unexpected_cfgs)]
        let superclass = class!(UIResponder);
        let mut decl = ClassDecl::new("AppDelegate", superclass).unwrap();

        #[allow(unexpected_cfgs)]
        decl.add_method(sel!(application:didFinishLaunchingWithOptions:),
            did_finish_launching as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> bool);
        #[allow(unexpected_cfgs)]
        decl.add_method(sel!(updateLoop:),
            update_loop as extern "C" fn(&Object, Sel, *mut Object));

        decl.register();

        #[allow(unexpected_cfgs)]
        let delegate_class = class!(AppDelegate);
        #[allow(unexpected_cfgs)]
        let delegate_instance: *mut Object = msg_send![delegate_class, new];

        #[allow(unexpected_cfgs)]
        let _: () = msg_send![app, setDelegate: delegate_instance];
        #[allow(unexpected_cfgs)]
        let _: () = msg_send![app, run];

        #[allow(unexpected_cfgs)]
        let _: () = msg_send![autorelease_pool, drain];
    }
}
