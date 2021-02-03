#![warn(clippy::all)]

mod main_window;
mod window_system;


#[no_mangle]
pub extern fn create_plugin() -> *mut dyn nrg_core::Plugin {
    let window = main_window::MainWindow::default();
    let boxed = Box::new(window);
    Box::into_raw(boxed)
}

#[no_mangle]
pub extern fn destroy_plugin(ptr: *mut dyn nrg_core::Plugin) {
    let boxed: Box<main_window::MainWindow> = unsafe { Box::from_raw( std::mem::transmute_copy(&ptr) ) };
    let window = *boxed;
    drop(window);
}
