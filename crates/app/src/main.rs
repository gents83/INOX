#![allow(dead_code)]
#![warn(clippy::all)]
#![allow(unexpected_cfgs)]

pub mod launcher;
pub mod platform;

pub fn main() {
    use crate::{launcher::Launcher, platform::*};
    use std::sync::Arc;

    setup_env();

    let launcher = Arc::new(Launcher::default());

    load_plugins(&launcher);

    launcher.start();

    main_update(launcher);
}
