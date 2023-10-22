#![allow(dead_code)]
#![warn(clippy::all)]

pub mod launcher;
pub mod platform;


pub fn main() {
    use std::sync::Arc;
    use crate::{launcher::Launcher, platform::*};

    setup_env();

    let launcher = Arc::new(Launcher::default());

    load_plugins(&launcher);

    launcher.start();

    main_update(launcher);
}