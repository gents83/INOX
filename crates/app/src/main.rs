use std::sync::Arc;

use inox_launcher::{launcher::Launcher, platform::*};

fn main() {
    setup_env();

    let launcher = Arc::new(Launcher::default());

    load_plugins(&launcher);

    launcher.start();

    main_update(launcher);
}
