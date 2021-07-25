use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::*;
use nrg_graphics::*;
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_resources::{ResourceEvent, SharedData, SharedDataRw};
use nrg_serialize::INVALID_UID;

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

        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            job_handler,
            message_channel,
        }
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
    fn init(&mut self) {
    }

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Init && state != RendererState::Submitted {
            return true;
        }

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<ResourceEvent>() {
                let e = msg.as_any().downcast_ref::<ResourceEvent>().unwrap();
                let ResourceEvent::Reload(path) = e;
                if is_shader(path)
                    && SharedData::has_resources_of_type::<PipelineInstance>(&self.shared_data)
                {
                    let pipelines = SharedData::get_resources_of_type::<PipelineInstance>(
                        &self.shared_data,
                    );
                    for p in pipelines.iter() {
                        p.resource().get_mut()
                            .check_shaders_to_reload(path.to_str().unwrap().to_string());
                    }
                } else if is_texture(path)
                    && SharedData::has_resources_of_type::<TextureInstance>(&self.shared_data)
                {
                    let textures =
                        SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);
                    for t in textures.iter() {
                        if t.resource().get().path() == path.as_path() {
                            t.resource().get_mut().invalidate();
                        }
                    }
                }
                
            }
        });

        {
            let mut renderer = self.renderer.write().unwrap();
            let mut render_passes =
                SharedData::get_resources_of_type::<RenderPassInstance>(&self.shared_data);
            let mut pipelines =
                SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
            let mut materials =
                SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);

            let mut textures =
                SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);
            let fonts = SharedData::get_resources_of_type::<FontInstance>(&self.shared_data);

            renderer.prepare_frame(
                &mut render_passes,
                &mut pipelines,
                &mut materials,
                &mut textures,
                &fonts,
            );
        }

        let wait_count = Arc::new(AtomicUsize::new(0));

        if SharedData::has_resources_of_type::<MaterialInstance>(&self.shared_data) {
            let materials =
                SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);

            materials
                .iter()
                .enumerate()
                .for_each(|(material_index, material_instance)| {
                    if material_instance.resource().get().has_meshes() {
                        let mut diffuse_texture_id = INVALID_UID;
                        let diffuse_color = material_instance.resource().get().diffuse_color();
                        let outline_color = material_instance.resource().get().outline_color();
                        let pipeline = material_instance.resource().get().pipeline();
                        
                        let (diffuse_texture_index, diffuse_layer_index) =
                        if !material_instance.resource().get().has_diffuse_texture() {
                            (INVALID_INDEX, INVALID_INDEX)
                        } else {
                                let diffuse_texture = material_instance.resource().get().diffuse_texture();
                                diffuse_texture_id = diffuse_texture.id();
                                let (ti, li) = (
                                    diffuse_texture.resource().get().texture_index() as _,
                                    diffuse_texture.resource().get().layer_index() as _,
                                );
                                (ti, li)
                            };

                        material_instance
                            .resource().get()
                            .meshes()
                            .iter()
                            .enumerate()
                            .for_each(|(mesh_index, mesh_instance)| {
                                if mesh_instance.resource().get().is_visible() {
                                    let mesh_id = mesh_instance.id();
                                    let shared_data = self.shared_data.clone();
                                    let r = self.renderer.clone();
                                    let pipeline = pipeline.clone();
                                    let wait_count = wait_count.clone();

                                    wait_count.fetch_add(1, Ordering::SeqCst);

                                    let job_name = format!(
                                        "PrepareMaterial [{}] with mesh [{}]",
                                        material_index, mesh_index
                                    );

                                    self.job_handler.write().unwrap().add_job(
                                        job_name.as_str(),
                                        move || {
                                            let mesh_instance = SharedData::get_resource::<
                                                MeshInstance,
                                            >(
                                                &shared_data, mesh_id
                                            );

                                            if !diffuse_texture_id.is_nil() && diffuse_texture_index >= 0 && diffuse_layer_index >= 0 {
                                                let renderer = r.read().unwrap();
                                                let diffuse_texture = renderer
                                                    .get_texture_handler()
                                                    .get_texture(diffuse_texture_id);
                                                mesh_instance.resource()
                                                    .get_mut()
                                                    .process_uv_for_texture(Some(diffuse_texture));
                                            } else {
                                                mesh_instance.resource()
                                                    .get_mut()
                                                    .process_uv_for_texture(None);
                                            }
                                            let mut renderer = r.write().unwrap();
                                            if let Some(pipeline) = renderer
                                                .get_pipelines()
                                                .iter_mut()
                                                .find(|p| p.id() == pipeline.id())
                                            {
                                                pipeline.add_mesh_instance(
                                                    &mesh_instance.resource().get(),
                                                    diffuse_color,
                                                    diffuse_texture_index,
                                                    diffuse_layer_index,
                                                    outline_color,
                                                );
                                            } else {
                                                eprintln!("Tyring to render with an unregistered pipeline {}", pipeline.resource().get().data().name);
                                            }

                                            wait_count.fetch_sub(1, Ordering::SeqCst);
                                        },
                                    );
                                }
                            });
                    }
                });
        }

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
                r.end_preparation();
            });

        true
    }
    fn uninit(&mut self) {}
}
