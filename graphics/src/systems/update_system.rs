use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_resources::{Resource, ResourceData, ResourceEvent, SharedData, SharedDataRw};
use nrg_serialize::INVALID_UID;

use crate::{
    is_shader, is_texture, Mesh, Pipeline, RenderPass, RendererRw, RendererState, Texture,
    INVALID_INDEX,
};

pub struct UpdateSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    message_channel: MessageChannel,
}

impl UpdateSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        let message_channel = MessageChannel::default();
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<ResourceEvent>(message_channel.get_messagebox());

        crate::register_resource_types(shared_data);
        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            job_handler,
            message_channel,
        }
    }

    fn handle_events(&self) {
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<ResourceEvent>() {
                let e = msg.as_any().downcast_ref::<ResourceEvent>().unwrap();
                let ResourceEvent::Reload(path) = e;
                if is_shader(path)
                    && SharedData::has_resources_of_type::<Pipeline>(&self.shared_data)
                {
                    SharedData::for_each_resource(&self.shared_data, |p: &Resource<Pipeline>| {
                        p.get_mut()
                            .check_shaders_to_reload(path.to_str().unwrap().to_string());
                    });
                } else if is_texture(path)
                    && SharedData::has_resources_of_type::<Texture>(&self.shared_data)
                {
                    SharedData::for_each_resource(&self.shared_data, |t: &Resource<Texture>| {
                        if t.get().path() == path.as_path() {
                            t.get_mut().invalidate();
                        }
                    });
                }
            }
        });
    }

    fn create_render_mesh_job(
        renderer: RendererRw,
        mesh: Resource<Mesh>,
        render_pass_pipeline: Option<Resource<Pipeline>>,
    ) {
        let mut texture_id = INVALID_UID;
        if let Some(material) = mesh.get().material() {
            let material = material.get();
            if material.has_diffuse_texture() {
                texture_id = material.diffuse_texture().id();
            }
        }
        if !texture_id.is_nil() {
            let renderer = renderer.read().unwrap();
            let texture_info = renderer.get_texture_handler().get_texture_info(texture_id);
            mesh.get_mut().process_uv_for_texture(texture_info);
        }
        if let Some(material) = mesh.get().material() {
            let material = material.get();
            let diffuse_color = material.diffuse_color();

            let (diffuse_texture_index, diffuse_layer_index) = if material.has_diffuse_texture() {
                nrg_profiler::scoped_profile!("Obtaining texture info");
                let (diffuse_texture_index, diffuse_layer_index) = (
                    material.diffuse_texture().get().texture_index() as _,
                    material.diffuse_texture().get().layer_index() as _,
                );
                (diffuse_texture_index, diffuse_layer_index)
            } else {
                (INVALID_INDEX, INVALID_INDEX)
            };
            if let Some(pipeline) = render_pass_pipeline {
                let renderer = renderer.read().unwrap();
                let device = renderer.device();
                let physical_device = renderer.instance().get_physical_device();
                pipeline.get_mut().add_mesh_instance(
                    device,
                    physical_device,
                    &mesh.get(),
                    diffuse_color,
                    diffuse_texture_index,
                    diffuse_layer_index,
                );
            } else if let Some(pipeline) = material.pipeline() {
                let renderer = renderer.read().unwrap();
                let device = renderer.device();
                let physical_device = renderer.instance().get_physical_device();
                pipeline.get_mut().add_mesh_instance(
                    device,
                    physical_device,
                    &mesh.get(),
                    diffuse_color,
                    diffuse_texture_index,
                    diffuse_layer_index,
                );
            }
        }
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
    fn id(&self) -> SystemId {
        self.id
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Submitted {
            return true;
        }

        self.handle_events();

        let wait_count = Arc::new(AtomicUsize::new(0));

        {
            let mut renderer = self.renderer.write().unwrap();
            if !renderer.device_mut().acquire_image() {
                renderer.recreate();
                return true;
            }
            renderer.change_state(RendererState::Preparing);
            renderer.prepare_frame();
        }

        SharedData::for_each_resource(&self.shared_data, |texture: &Resource<Texture>| {
            if texture.get().update_from_gpu() {
                let job_name = format!("Readback texture {:?}", texture.id());
                let renderer = self.renderer.clone();
                let texture = texture.clone();
                let wait_count = wait_count.clone();
                wait_count.fetch_add(1, Ordering::SeqCst);
                self.job_handler
                    .write()
                    .unwrap()
                    .add_job(job_name.as_str(), move || {
                        let renderer = renderer.read().unwrap();
                        let device = renderer.device();
                        let physical_device = renderer.instance().get_physical_device();
                        let texture_handler = renderer.get_texture_handler();
                        texture
                            .get_mut()
                            .capture_image(texture_handler, device, physical_device);
                        wait_count.fetch_sub(1, Ordering::SeqCst);
                    });
            }
        });

        SharedData::for_each_resource(&self.shared_data, |render_pass: &Resource<RenderPass>| {
            if render_pass.get().is_initialized() {
                let pipeline = render_pass.get().pipeline().clone();
                let mesh_category_to_draw = render_pass.get().mesh_category_to_draw().to_vec();
                SharedData::for_each_resource(&self.shared_data, |mesh: &Resource<Mesh>| {
                    let should_render = mesh_category_to_draw
                        .iter()
                        .any(|id| mesh.get().category_identifier() == *id);

                    if !should_render || !mesh.get().is_visible() {
                        return;
                    }
                    let renderer = self.renderer.clone();
                    let wait_count = wait_count.clone();
                    let mesh = mesh.clone();
                    let pipeline = pipeline.clone();

                    let job_name = format!(
                        "Processing mesh {:?} for RenderPass [{:?}",
                        mesh.id(),
                        render_pass.get().data().name
                    );
                    wait_count.fetch_add(1, Ordering::SeqCst);
                    self.job_handler
                        .write()
                        .unwrap()
                        .add_job(job_name.as_str(), move || {
                            nrg_profiler::scoped_profile!(format!(
                                "create_render_mesh_job[{}]",
                                mesh.id()
                            )
                            .as_str());
                            Self::create_render_mesh_job(renderer, mesh, pipeline);
                            wait_count.fetch_sub(1, Ordering::SeqCst);
                        });
                });
            }
        });

        let renderer = self.renderer.clone();
        let job_name = "EndPreparation";
        self.job_handler
            .write()
            .unwrap()
            .add_job(job_name, move || {
                while wait_count.load(Ordering::SeqCst) > 0 {
                    thread::yield_now();
                }

                let mut r = renderer.write().unwrap();
                r.change_state(RendererState::Prepared);
            });

        true
    }
    fn uninit(&mut self) {}
}
