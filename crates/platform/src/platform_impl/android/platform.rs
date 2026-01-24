use std::sync::{RwLock, OnceLock};
use android_activity::AndroidApp;

pub mod handle;
pub mod window;
pub mod watcher;

pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();
pub struct NativeWindowWrapper(*mut core::ffi::c_void);
unsafe impl Send for NativeWindowWrapper {}
unsafe impl Sync for NativeWindowWrapper {}

pub static NATIVE_WINDOW: RwLock<NativeWindowWrapper> = RwLock::new(NativeWindowWrapper(core::ptr::null_mut()));

pub fn create_android_app(app: AndroidApp) {
    let _ = ANDROID_APP.set(app);
}
