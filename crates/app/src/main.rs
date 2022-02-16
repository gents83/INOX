use std::sync::Arc;

use inox_launcher::{launcher::Launcher, platform::*};

fn main() {
    setup_env();

    let launcher = Arc::new(Launcher::default());

    launcher.prepare();

    launcher.start();

    let mut binarizer = binarizer_start(launcher.shared_data(), launcher.message_hub());
    binarizer = binarizer_update(binarizer);

    main_update(launcher);

    binarizer_stop(binarizer);
}
