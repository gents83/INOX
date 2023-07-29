use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use inox_core::{
    App, ContextRc, PfnCreatePlugin, PfnDestroyPlugin, PfnLoadConfigPlugin, PfnPreparePlugin,
    PfnUnpreparePlugin,
};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

#[derive(Default)]
pub struct Launcher {
    app: Arc<RwLock<App>>,
}

impl Launcher {
    pub fn context(&self) -> ContextRc {
        self.app.read().unwrap().context().clone()
    }
    pub fn shared_data(&self) -> SharedDataRc {
        self.app.read().unwrap().context().shared_data().clone()
    }
    pub fn message_hub(&self) -> MessageHubRc {
        self.app.read().unwrap().context().message_hub().clone()
    }

    pub fn start(&self) {
        let app = &mut self.app.write().unwrap();

        debug_log!("Starting app");

        app.start();
    }

    pub fn add_dynamic_plugin(&self, name: &str, path: &Path) {
        let app = &mut self.app.write().unwrap();
        app.add_dynamic_plugin(path);
        app.load_config_on_plugin_systems(name);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_static_plugin(
        &self,
        name: &str,
        context: &ContextRc,
        fn_c: PfnCreatePlugin,
        fn_l: PfnLoadConfigPlugin,
        fn_p: PfnPreparePlugin,
        fn_u: PfnUnpreparePlugin,
        fn_d: PfnDestroyPlugin,
    ) {
        let app = &mut self.app.write().unwrap();

        if let Some(fn_c) = fn_c {
            let mut plugin_holder = unsafe { fn_c(context) };
            plugin_holder.load_config_fn = fn_l;
            plugin_holder.prepare_fn = fn_p;
            plugin_holder.unprepare_fn = fn_u;
            plugin_holder.destroy_fn = fn_d;
            app.add_static_plugin(plugin_holder);
            app.load_config_on_plugin_systems(name);
        }
    }

    pub fn update(&self) -> bool {
        self.app.write().unwrap().run(false)
    }
}
