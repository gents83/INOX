use std::sync::{RwLock, OnceLock};
use android_activity::AndroidApp;

pub mod handle;
pub mod window;

pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();
pub static NATIVE_WINDOW: RwLock<*mut core::ffi::c_void> = RwLock::new(core::ptr::null_mut());

pub fn create_android_app(app: AndroidApp) {
    let _ = ANDROID_APP.set(app);
}
