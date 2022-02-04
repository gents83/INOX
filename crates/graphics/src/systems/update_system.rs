use sabi_core::{JobHandlerRw, System};

use sabi_messenger::{Listener, MessageHubRc};
use sabi_platform::WindowEvent;
use sabi_resources::{
    ConfigBase, DataTypeResource, ReloadEvent, Resource, ResourceEvent, SerializableResource,
    SharedData, SharedDataRc,
};
use sabi_serialize::{generate_random_uid, read_from_file};

use crate::{
    is_shader, Mesh, Pipeline, RenderPass, RenderPassData, RendererRw, RendererState, Texture,
};

use super::config::Config;
pub const RENDERING_UPDATE: &str = "RENDERING_UPDATE";

pub struct UpdateSystem {
    renderer: RendererRw,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    job_handler: JobHandlerRw,
    listener: Listener,
    render_passes: Vec<Resource<RenderPass>>,
}

impl UpdateSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        job_handler: &JobHandlerRw,
    ) -> Self {
        let listener = Listener::new(message_hub);

        crate::register_resource_types(shared_data, message_hub);
        Self {
            renderer,
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            job_handler: job_handler.clone(),
            listener,
            render_passes: Vec::new(),
        }
    }

    pub fn load_render_passes(&mut self, render_passes: &[RenderPassData]) -> &mut Self {
        for render_pass_data in render_passes.iter() {
            self.render_passes.push(RenderPass::new_resource(
                &self.shared_data,
                &self.message_hub,
                generate_random_uid(),
                render_pass_data.clone(),
            ));
        }
        self
    }

    fn handle_events(&self) {
        self.listener
            .process_messages(|e: &WindowEvent| {
                if let WindowEvent::SizeChanged(width, height) = e {
                    let mut renderer = self.renderer.write().unwrap();
                    renderer.set_surface_size(*width, *height);
                }
            })
            .process_messages(|e: &ReloadEvent| {
                let path = e.path.as_path();
                if is_shader(path) {
                    SharedData::for_each_resource_mut(&self.shared_data, |_, p: &mut Pipeline| {
                        p.check_shaders_to_reload(path.to_str().unwrap().to_string());
                    });
                } else if Texture::is_matching_extension(path) {
                    SharedData::for_each_resource_mut(&self.shared_data, |_, t: &mut Texture| {
                        if t.path() == path {
                            t.invalidate();
                        }
                    });
                }
            })
            .process_messages(|e: &ResourceEvent<Mesh>| match e {
                ResourceEvent::Created(resource) => {
                    self.renderer.write().unwrap().on_mesh_added(resource);
                }
                ResourceEvent::Changed(id) => {
                    self.renderer.write().unwrap().on_mesh_changed(id);
                }
                ResourceEvent::Destroyed(id) => {
                    self.renderer.write().unwrap().on_mesh_removed(id);
                }
                _ => {}
            });
    }
}

impl Drop for UpdateSystem {
    fn drop(&mut self) {
        crate::unregister_resource_types(&self.shared_data);
    }
}

unsafe impl Send for UpdateSystem {}
unsafe impl Sync for UpdateSystem {}

impl System for UpdateSystem {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config = read_from_file(config.get_filepath(plugin_name).as_path());

        self.load_render_passes(&config.render_passes);
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.listener
            .register::<WindowEvent>()
            .register::<ReloadEvent>()
            .register::<ResourceEvent<Mesh>>();
    }

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Submitted {
            return true;
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Preparing);
        }

        self.handle_events();

        //[...]
        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.prepare_frame();
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Prepared);
        }

        true
    }
    fn uninit(&mut self) {
        self.listener
            .unregister::<WindowEvent>()
            .unregister::<ReloadEvent>()
            .unregister::<ResourceEvent<Mesh>>();
    }
}
