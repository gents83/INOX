use inox_core::{
    implement_unique_system_uid, ContextRc, JobHandlerRw, JobHandlerTrait, JobPriority, System,
    INDEPENDENT_JOB_ID,
};
use inox_messenger::Listener;
use inox_resources::{
    ConfigBase, ConfigEvent, Handle, OnCreateData, SerializableResource, SharedDataRc,
};
use inox_serialize::read_from_file;

use crate::{
    RendererRw, RendererState, Texture, ENV_MAP_UID, LUT_PBR_CHARLIE_UID, LUT_PBR_GGX_UID,
};

use super::config::Config;

pub const RENDERING_PHASE: &str = "RENDERING_PHASE";

pub struct RenderingSystem {
    config: Config,
    listener: Listener,
    shared_data: SharedDataRc,
    renderer: RendererRw,
    job_handler: JobHandlerRw,
    pbr_lut_charlie: Handle<Texture>,
    pbr_lut_ggx: Handle<Texture>,
    env_map: Handle<Texture>,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw, context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());
        Self {
            renderer,
            config: Config::default(),
            shared_data: context.shared_data().clone(),
            listener,
            job_handler: context.job_handler().clone(),
            pbr_lut_charlie: None,
            pbr_lut_ggx: None,
            env_map: None,
        }
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

implement_unique_system_uid!(RenderingSystem);

impl System for RenderingSystem {
    fn read_config(&mut self, plugin_name: &str) {
        self.listener.register::<ConfigEvent<Config>>();
        let message_hub = self.listener.message_hub().clone();
        let filename = self.config.get_filename().to_string();
        read_from_file(
            self.config.get_filepath(plugin_name).as_path(),
            self.shared_data.serializable_registry(),
            Box::new(move |data: Config| {
                message_hub.send_event(ConfigEvent::Loaded(filename.clone(), data));
            }),
        );
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.listener.register::<ConfigEvent<Config>>();
    }

    fn run(&mut self) -> bool {
        self.listener
            .process_messages(|e: &ConfigEvent<Config>| match e {
                ConfigEvent::Loaded(filename, config) => {
                    inox_profiler::scoped_profile!("Processing ConfigEvent");
                    if filename == self.config.get_filename() {
                        self.config = config.clone();

                        self.pbr_lut_charlie = Some(Texture::request_load(
                            &self.shared_data,
                            self.listener.message_hub(),
                            &config.lut_pbr_charlie,
                            OnCreateData::create(|t: &mut Texture| {
                                t.mark_as_LUT(&LUT_PBR_CHARLIE_UID);
                            }),
                        ));
                        self.pbr_lut_ggx = Some(Texture::request_load(
                            &self.shared_data,
                            self.listener.message_hub(),
                            &config.lut_pbr_ggx,
                            OnCreateData::create(|t: &mut Texture| {
                                t.mark_as_LUT(&LUT_PBR_GGX_UID);
                            }),
                        ));
                        self.env_map = Some(Texture::request_load(
                            &self.shared_data,
                            self.listener.message_hub(),
                            &config.env_map,
                            OnCreateData::create(|t: &mut Texture| {
                                t.mark_as_LUT(&ENV_MAP_UID);
                            }),
                        ));
                    }
                }
            });

        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Drawing);
        }

        let renderer = self.renderer.clone();
        self.job_handler.add_job(
            &INDEPENDENT_JOB_ID,
            "Render Draw",
            JobPriority::High,
            move || {
                let mut renderer = renderer.write().unwrap();
                renderer.submit_command_buffer();
                renderer.present();
                renderer.change_state(RendererState::Submitted);
            },
        );
        true
    }

    fn uninit(&mut self) {
        self.listener.unregister::<ConfigEvent<Config>>();
    }
}
