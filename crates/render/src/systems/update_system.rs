use inox_core::{implement_unique_system_uid, ContextRc, System};

use inox_math::{VecBase, Vector2};
use inox_messenger::Listener;
use inox_platform::{MouseEvent, MouseState, WindowEvent};
use inox_resources::{
    DataTypeResource, DataTypeResourceEvent, ReloadEvent, Resource, ResourceEvent,
    SerializableResourceEvent, SharedData, SharedDataRc,
};
use inox_uid::generate_random_uid;

use crate::{
    is_shader, CommandBuffer, ComputePipeline, Light, Material, Mesh, RenderContextRc,
    RenderPipeline, RendererState, Texture, View, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

pub const RENDERING_UPDATE: &str = "RENDERING_UPDATE";

pub struct UpdateSystem {
    render_context: RenderContextRc,
    shared_data: SharedDataRc,
    listener: Listener,
    view: Resource<View>,
    mouse_coords: Vector2,
    width: u32,
    height: u32,
    resolution_changed: bool,
}

impl UpdateSystem {
    pub fn new(render_context: &RenderContextRc, context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());

        Self {
            view: View::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &0,
                None,
            ),
            render_context: render_context.clone(),
            shared_data: context.shared_data().clone(),
            listener,
            resolution_changed: false,
            mouse_coords: Vector2::default_zero(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }

    fn handle_events(&mut self, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("update_system::handle_events");
        //REMINDER: message processing order is important - RenderPass must be processed before Texture
        self.listener
            .process_messages(|e: &WindowEvent| {
                if let WindowEvent::SizeChanged(width, height) = e {
                    self.width = *width;
                    self.height = *height;
                    self.resolution_changed = true;
                }
            })
            .process_messages(|e: &MouseEvent| {
                if e.state == MouseState::Move {
                    self.mouse_coords = [e.x as f32, e.y as f32].into();
                }
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
                    self.render_context
                        .on_texture_changed(id, &mut command_buffer.encoder);
                }
                ResourceEvent::Created(t) => {
                    self.render_context
                        .on_texture_changed(t.id(), &mut command_buffer.encoder);
                }
                ResourceEvent::Destroyed(id) => {
                    self.render_context
                        .global_buffers()
                        .remove_texture(&self.render_context, id);
                }
            })
            .process_messages(|e: &DataTypeResourceEvent<Light>| {
                let DataTypeResourceEvent::Loaded(id, light_data) = e;
                self.render_context.global_buffers().update_light(
                    &self.render_context,
                    id,
                    light_data,
                );
            })
            .process_messages(|e: &ResourceEvent<Light>| match e {
                ResourceEvent::Created(l) => {
                    self.render_context.global_buffers().add_light(
                        &self.render_context,
                        l.id(),
                        &mut l.get_mut(),
                    );
                }
                ResourceEvent::Changed(id) => {
                    if let Some(light) = self.shared_data.get_resource::<Light>(id) {
                        self.render_context.global_buffers().update_light(
                            &self.render_context,
                            id,
                            light.get().data(),
                        );
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    self.render_context
                        .global_buffers()
                        .remove_light(&self.render_context, id);
                }
            })
            .process_messages(|e: &ResourceEvent<Material>| match e {
                ResourceEvent::Created(m) => {
                    self.render_context.global_buffers().add_material(
                        &self.render_context,
                        m.id(),
                        &mut m.get_mut(),
                    );
                }
                ResourceEvent::Changed(id) => {
                    if let Some(m) = self.shared_data.get_resource::<Material>(id) {
                        self.render_context.global_buffers().add_material(
                            &self.render_context,
                            m.id(),
                            &mut m.get_mut(),
                        );
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    self.render_context.global_buffers().remove_material(id);
                }
            })
            .process_messages(|e: &DataTypeResourceEvent<Material>| {
                let DataTypeResourceEvent::Loaded(id, material_data) = e;
                self.render_context.global_buffers().update_material(
                    &self.render_context,
                    id,
                    material_data,
                );
            })
            .process_messages(|e: &DataTypeResourceEvent<Mesh>| {
                let DataTypeResourceEvent::Loaded(id, mesh_data) = e;
                let mesh_index = self.render_context.global_buffers().add_mesh(id, mesh_data);
                if let Some(mesh) = self.shared_data.get_resource::<Mesh>(id) {
                    mesh.get_mut().set_mesh_index(mesh_index);
                }
            })
            .process_messages(|e: &ResourceEvent<Mesh>| match e {
                ResourceEvent::Changed(id) => {
                    if let Some(mesh) = self.shared_data.get_resource::<Mesh>(id) {
                        self.render_context.global_buffers().change_mesh(
                            &self.render_context,
                            id,
                            &mut mesh.get_mut(),
                        );
                    }
                }
                ResourceEvent::Destroyed(id) => {
                    self.render_context.global_buffers().remove_mesh(id, true);
                }
                _ => {}
            });
    }
}

unsafe impl Send for UpdateSystem {}
unsafe impl Sync for UpdateSystem {}

implement_unique_system_uid!(UpdateSystem);

impl System for UpdateSystem {
    fn read_config(&mut self, _plugin_name: &str) {}

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.listener
            .register::<WindowEvent>()
            .register::<MouseEvent>()
            .register::<ReloadEvent>()
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
        let state = self.render_context.state();
        if state != RendererState::Submitted {
            return true;
        }

        if self.resolution_changed {
            self.render_context
                .set_surface_size(self.width as f32 as _, self.height as f32 as _);

            self.resolution_changed = false;
            return true;
        }
        if !self.render_context.obtain_surface_texture() {
            return true;
        }

        self.render_context.change_state(RendererState::Preparing);

        let mut command_buffer = self.render_context.new_command_buffer();

        self.handle_events(&mut command_buffer);

        {
            let screen_size = Vector2::new(self.width as _, self.height as _);

            self.render_context.global_buffers().update_constant_data(
                &self.render_context,
                (
                    self.view.get().view(),
                    self.view.get().proj(),
                    self.view.get().near(),
                    self.view.get().far(),
                ),
                screen_size,
                self.mouse_coords,
            );

            self.render_context.update_passes(command_buffer);
        }

        self.render_context.change_state(RendererState::Prepared);

        true
    }
    fn uninit(&mut self) {
        self.listener
            .unregister::<WindowEvent>()
            .unregister::<MouseEvent>()
            .unregister::<ReloadEvent>()
            .unregister::<DataTypeResourceEvent<Light>>()
            .unregister::<DataTypeResourceEvent<Material>>()
            .unregister::<DataTypeResourceEvent<Mesh>>()
            .unregister::<SerializableResourceEvent<RenderPipeline>>()
            .unregister::<SerializableResourceEvent<ComputePipeline>>()
            .unregister::<SerializableResourceEvent<Texture>>()
            .unregister::<ResourceEvent<Light>>()
            .unregister::<ResourceEvent<Texture>>()
            .unregister::<ResourceEvent<Material>>()
            .unregister::<ResourceEvent<Mesh>>();
    }
}
