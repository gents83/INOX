use inox_core::{implement_unique_system_uid, ContextRc, System};

use inox_math::Vector2;
use inox_messenger::{Listener, MessageHubRc};
use inox_platform::WindowEvent;
use inox_resources::{
    ConfigBase, ConfigEvent, DataTypeResource, DataTypeResourceEvent, ReloadEvent, Resource,
    ResourceEvent, SerializableResourceEvent, SharedData, SharedDataRc,
};
use inox_serialize::read_from_file;
use inox_uid::generate_random_uid;

use crate::{
    is_shader, CommandBuffer, ComputePipeline, Light, Material, Mesh, RenderPipeline, RendererRw,
    RendererState, Texture, View, DEFAULT_HEIGHT, DEFAULT_WIDTH,
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
    resolution_changed: bool,
}

impl UpdateSystem {
    pub fn new(renderer: RendererRw, context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());

        Self {
            view: View::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &0,
                None,
            ),
            config: Config::default(),
            renderer,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            listener,
            resolution_changed: false,
            scale_factor: 1.0,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }

    fn handle_events(&mut self, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("update_system::handle_events");
        //REMINDER: message processing order is important - RenderPass must be processed before Texture
        self.listener
            .process_messages(|e: &WindowEvent| match e {
                WindowEvent::SizeChanged(width, height) => {
                    self.width = *width;
                    self.height = *height;
                    self.resolution_changed = true;
                }
                WindowEvent::ScaleFactorChanged(v) => {
                    self.scale_factor = *v;
                    self.resolution_changed = true;
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
            .process_messages(|e: &ResourceEvent<Texture>| match e {
                ResourceEvent::Changed(id) => {
                    self.renderer
                        .write()
                        .unwrap()
                        .on_texture_changed(id, &mut command_buffer.encoder);
                }
                ResourceEvent::Created(t) => {
                    self.renderer
                        .write()
                        .unwrap()
                        .on_texture_changed(t.id(), &mut command_buffer.encoder);
                }
                ResourceEvent::Destroyed(id) => {
                    let renderer = self.renderer.read().unwrap();
                    let render_context = renderer.render_context();
                    render_context.render_buffers.remove_texture(id);
                }
            })
            .process_messages(|e: &DataTypeResourceEvent<Light>| {
                let DataTypeResourceEvent::Loaded(id, light_data) = e;
                let renderer = self.renderer.read().unwrap();
                let render_context = renderer.render_context();
                render_context.render_buffers.update_light(id, light_data);
            })
            .process_messages(|e: &ResourceEvent<Light>| match e {
                ResourceEvent::Created(l) => {
                    let renderer = self.renderer.read().unwrap();
                    let render_context = renderer.render_context();
                    render_context
                        .render_buffers
                        .add_light(l.id(), &mut l.get_mut());
                }
                ResourceEvent::Changed(id) => {
                    if let Some(light) = self.shared_data.get_resource::<Light>(id) {
                        let renderer = self.renderer.read().unwrap();

                        let render_context = renderer.render_context();
                        render_context
                            .render_buffers
                            .update_light(id, light.get().data());
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    let renderer = self.renderer.read().unwrap();
                    let render_context = renderer.render_context();
                    render_context.render_buffers.remove_light(id);
                }
            })
            .process_messages(|e: &ResourceEvent<Material>| match e {
                ResourceEvent::Created(m) => {
                    let renderer = self.renderer.read().unwrap();
                    let render_context = renderer.render_context();
                    render_context
                        .render_buffers
                        .add_material(m.id(), &mut m.get_mut());
                }
                ResourceEvent::Changed(id) => {
                    if let Some(m) = self.shared_data.get_resource::<Material>(id) {
                        let renderer = self.renderer.read().unwrap();
                        let render_context = renderer.render_context();
                        render_context
                            .render_buffers
                            .add_material(m.id(), &mut m.get_mut());
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    let renderer = self.renderer.read().unwrap();
                    let render_context = renderer.render_context();
                    render_context.render_buffers.remove_material(id);
                }
            })
            .process_messages(|e: &DataTypeResourceEvent<Material>| {
                let DataTypeResourceEvent::Loaded(id, material_data) = e;
                let renderer = self.renderer.read().unwrap();
                let render_context = renderer.render_context();
                render_context
                    .render_buffers
                    .update_material(id, material_data);
            })
            .process_messages(|e: &DataTypeResourceEvent<Mesh>| {
                let DataTypeResourceEvent::Loaded(id, mesh_data) = e;
                let renderer = self.renderer.read().unwrap();
                let render_context = renderer.render_context();
                render_context.render_buffers.add_mesh(id, mesh_data);
            })
            .process_messages(|e: &ResourceEvent<Mesh>| match e {
                ResourceEvent::Changed(id) => {
                    if let Some(mesh) = self.shared_data.get_resource::<Mesh>(id) {
                        let renderer = self.renderer.read().unwrap();
                        let render_context = renderer.render_context();
                        render_context
                            .render_buffers
                            .change_mesh(id, &mut mesh.get_mut());
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    let renderer = self.renderer.read().unwrap();
                    let render_context = renderer.render_context();
                    render_context.render_buffers.remove_mesh(id, true);
                }
                _ => {}
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
            .register::<DataTypeResourceEvent<Light>>()
            .register::<DataTypeResourceEvent<Material>>()
            .register::<DataTypeResourceEvent<Mesh>>()
            .register::<SerializableResourceEvent<RenderPipeline>>()
            .register::<SerializableResourceEvent<ComputePipeline>>()
            .register::<SerializableResourceEvent<Texture>>()
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

        {
            let mut renderer = self.renderer.write().unwrap();
            if self.resolution_changed || !renderer.obtain_surface_texture() {
                renderer.set_surface_size(self.width as f32 as _, self.height as f32 as _);

                self.resolution_changed = false;
                return true;
            }
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Preparing);
        }

        let mut command_buffer = {
            let renderer = self.renderer.read().unwrap();
            let render_context = renderer.render_context();
            render_context.core.new_command_buffer()
        };

        self.handle_events(&mut command_buffer);

        {
            let mut renderer = self.renderer.write().unwrap();
            {
                let screen_size = Vector2::new(self.width as _, self.height as _);

                let render_context = renderer.render_context();
                render_context.update_constant_data(
                    self.view.get().view(),
                    self.view.get().proj(),
                    screen_size,
                    self.view.get().fov_in_degrees(),
                );
            }

            renderer.prepare();
            renderer.update_passes(command_buffer);
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
            .unregister::<DataTypeResourceEvent<Light>>()
            .unregister::<DataTypeResourceEvent<Material>>()
            .unregister::<DataTypeResourceEvent<Mesh>>()
            .unregister::<SerializableResourceEvent<RenderPipeline>>()
            .unregister::<SerializableResourceEvent<ComputePipeline>>()
            .unregister::<SerializableResourceEvent<Texture>>()
            .unregister::<ConfigEvent<Config>>()
            .unregister::<ResourceEvent<Light>>()
            .unregister::<ResourceEvent<Texture>>()
            .unregister::<ResourceEvent<Material>>()
            .unregister::<ResourceEvent<Mesh>>();
    }
}
