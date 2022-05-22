use inox_core::{implement_unique_system_uid, ContextRc, System};

use inox_math::Vector2;
use inox_messenger::{Listener, MessageHubRc};
use inox_platform::WindowEvent;
use inox_resources::{
    ConfigBase, ConfigEvent, DataTypeResource, ReloadEvent, Resource, ResourceEvent,
    SerializableResourceEvent, SharedData, SharedDataRc,
};
use inox_serialize::read_from_file;
use inox_uid::generate_random_uid;

use crate::{
    is_shader, ComputePipeline, GetRenderContext, Light, Material, Mesh, RenderPass,
    RenderPipeline, RendererRw, RendererState, Texture, View, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

use super::config::Config;
pub const RENDERING_UPDATE: &str = "RENDERING_UPDATE";

pub struct UpdateSystem {
    config: Config,
    renderer: RendererRw,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    listener: Listener,
    view: Resource<View>,
    scale_factor: f32,
    width: u32,
    height: u32,
}

impl UpdateSystem {
    pub fn new(renderer: RendererRw, context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());

        Self {
            view: View::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                0,
            ),
            config: Config::default(),
            renderer,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            listener,
            scale_factor: 1.0,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }

    fn handle_events(&mut self, encoder: &mut wgpu::CommandEncoder) {
        //REMINDER: message processing order is important - RenderPass must be processed before Texture
        self.listener
            .process_messages(|e: &WindowEvent| match e {
                WindowEvent::SizeChanged(width, height) => {
                    self.width = *width;
                    self.height = *height;
                    let mut renderer = self.renderer.write().unwrap();
                    renderer.set_surface_size(
                        (*width as f32 * self.scale_factor) as _,
                        (*height as f32 * self.scale_factor) as _,
                    );
                }
                WindowEvent::ScaleFactorChanged(v) => {
                    self.scale_factor = *v;
                    let mut renderer = self.renderer.write().unwrap();
                    renderer.set_surface_size(
                        (self.width as f32 * self.scale_factor) as _,
                        (self.height as f32 * self.scale_factor) as _,
                    );
                }
                _ => {}
            })
            .process_messages(|e: &ReloadEvent| {
                let ReloadEvent::Reload(path) = e;
                if is_shader(path) {
                    SharedData::for_each_resource_mut(
                        &self.shared_data,
                        |_, p: &mut RenderPipeline| {
                            p.check_shaders_to_reload(path.to_str().unwrap().to_string());
                        },
                    );
                    SharedData::for_each_resource_mut(
                        &self.shared_data,
                        |_, p: &mut ComputePipeline| {
                            p.check_shaders_to_reload(path.to_str().unwrap().to_string());
                        },
                    );
                }
            })
            .process_messages(|e: &ResourceEvent<RenderPass>| match e {
                ResourceEvent::Changed(id) => {
                    self.renderer.write().unwrap().on_render_pass_changed(id);
                }
                ResourceEvent::Created(r) => {
                    self.renderer
                        .write()
                        .unwrap()
                        .on_render_pass_changed(r.id());
                }
                _ => {}
            })
            .process_messages(|e: &ResourceEvent<Texture>| match e {
                ResourceEvent::Changed(id) => {
                    self.renderer
                        .write()
                        .unwrap()
                        .on_texture_changed(id, encoder);
                }
                ResourceEvent::Created(t) => {
                    self.renderer
                        .write()
                        .unwrap()
                        .on_texture_changed(t.id(), encoder);
                }
                _ => {}
            })
            .process_messages(|e: &ResourceEvent<Light>| match e {
                ResourceEvent::Changed(id) => {
                    self.renderer.write().unwrap().on_light_changed(id);
                }
                ResourceEvent::Created(l) => {
                    self.renderer.write().unwrap().on_light_changed(l.id());
                }
                _ => {}
            })
            .process_messages(|e: &ResourceEvent<RenderPipeline>| {
                if let ResourceEvent::Changed(id) = e {
                    self.renderer
                        .write()
                        .unwrap()
                        .on_render_pipeline_changed(id);
                }
            })
            .process_messages(|e: &ResourceEvent<Material>| match e {
                ResourceEvent::Changed(id) => {
                    self.renderer.write().unwrap().on_material_changed(id);
                }
                ResourceEvent::Created(m) => {
                    self.renderer.write().unwrap().on_material_changed(m.id());
                }
                _ => {}
            })
            .process_messages(|e: &ResourceEvent<Mesh>| {
                if let ResourceEvent::Changed(id) = e {
                    self.renderer.write().unwrap().on_mesh_changed(id);
                }
            });
    }
}

unsafe impl Send for UpdateSystem {}
unsafe impl Sync for UpdateSystem {}

implement_unique_system_uid!(UpdateSystem);

impl System for UpdateSystem {
    fn read_config(&mut self, plugin_name: &str) {
        self.listener.register::<ConfigEvent<Config>>();
        let message_hub = self.message_hub.clone();
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
        self.listener
            .register::<WindowEvent>()
            .register::<ReloadEvent>()
            .register::<ConfigEvent<Config>>()
            .register::<SerializableResourceEvent<RenderPipeline>>()
            .register::<SerializableResourceEvent<ComputePipeline>>()
            .register::<SerializableResourceEvent<Texture>>()
            .register::<ResourceEvent<RenderPass>>()
            .register::<ResourceEvent<RenderPipeline>>()
            .register::<ResourceEvent<Material>>()
            .register::<ResourceEvent<Texture>>()
            .register::<ResourceEvent<Light>>()
            .register::<ResourceEvent<Mesh>>();
    }

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Submitted {
            if state == RendererState::Init {
                self.renderer.write().unwrap().check_initialization();
            }
            return true;
        }
        let mut encoder = {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Preparing);
            let render_context = renderer.render_context().get();
            let render_context = render_context.as_ref().unwrap();
            render_context.prepare_bindings();
            render_context.new_encoder()
        };

        self.handle_events(&mut encoder);

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.obtain_surface_texture();

            {
                let resolution = renderer.resolution();
                let screen_size = Vector2::new(resolution.0 as f32, resolution.1 as f32);

                let mut render_context = renderer.render_context().get_mut();
                let render_context = render_context.as_mut().unwrap();
                render_context.update_constant_data(
                    self.view.get().view(),
                    self.view.get().proj(),
                    screen_size,
                );
            }

            renderer.send_to_gpu(encoder);
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
            .unregister::<SerializableResourceEvent<RenderPipeline>>()
            .unregister::<SerializableResourceEvent<ComputePipeline>>()
            .unregister::<SerializableResourceEvent<Texture>>()
            .unregister::<ConfigEvent<Config>>()
            .unregister::<ResourceEvent<Light>>()
            .unregister::<ResourceEvent<Texture>>()
            .unregister::<ResourceEvent<Material>>()
            .unregister::<ResourceEvent<RenderPipeline>>()
            .unregister::<ResourceEvent<RenderPass>>()
            .unregister::<ResourceEvent<Mesh>>();
    }
}
