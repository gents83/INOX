use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use inox_core::{App, PfnCreatePlugin, PfnDestroyPlugin, PfnPreparePlugin, PfnUnpreparePlugin};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

#[derive(Default)]
pub struct Launcher {
    app: Arc<RwLock<App>>,
}

impl Launcher {
    pub fn shared_data(&self) -> SharedDataRc {
        self.app.read().unwrap().get_context().shared_data().clone()
    }
    pub fn message_hub(&self) -> MessageHubRc {
        self.app.read().unwrap().get_context().message_hub().clone()
    }

    pub fn start(&self) {
        let app = &mut self.app.write().unwrap();

        debug_log!("Starting app");

        app.start();
    }

    pub fn add_dynamic_plugin(&self, name: &str, path: &Path) {
        let app = &mut self.app.write().unwrap();
        app.add_dynamic_plugin(path);
        Self::read_config(app, name);
    }

    pub fn add_static_plugin(
        &self,
        name: &str,
        fn_c: PfnCreatePlugin,
        fn_p: PfnPreparePlugin,
        fn_u: PfnUnpreparePlugin,
        fn_d: PfnDestroyPlugin,
    ) {
        let app = &mut self.app.write().unwrap();

        if let Some(fn_c) = fn_c {
            let mut plugin_holder = unsafe { fn_c() };
            plugin_holder.prepare_fn = fn_p;
            plugin_holder.unprepare_fn = fn_u;
            plugin_holder.destroy_fn = fn_d;
            app.add_static_plugin(plugin_holder);
        }
        Self::read_config(app, name);
    }

    fn read_config(app: &mut App, plugin_name: &str) {
        debug_log!("Reading launcher configs");

        app.execute_on_systems(|s| {
            s.read_config(plugin_name);
        });
    }

    pub fn update(&self) -> bool {
        self.app.write().unwrap().run()
    }
}
