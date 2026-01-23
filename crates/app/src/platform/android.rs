use std::sync::Arc;
use android_activity::AndroidApp;
use crate::launcher::Launcher;

#[no_mangle]
fn android_main(app: AndroidApp) {
    inox_platform::platform_impl::platform::create_android_app(app);
    crate::main();
}

pub fn setup_env() {
    // Android setup
}

pub fn load_plugins(launcher: &Arc<Launcher>) {
    // Same as PC/generic
    let context = launcher.context();
    launcher.add_dynamic_plugin("inox_viewer", std::path::Path::new(""));
}

pub fn main_update(launcher: Arc<Launcher>) {
    loop {
        let can_continue = launcher.update();
        if !can_continue {
            break;
        }
    }
}
