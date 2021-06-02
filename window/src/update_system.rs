use std::{any::TypeId, sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    }, thread};

use crate::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::{ResourceEvent, SharedData, SharedDataRw};

pub struct UpdateSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    config: Config,
    message_channel: MessageChannel,
}

impl UpdateSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        job_handler: JobHandlerRw,
        config: &Config,
    ) -> Self {
        let message_channel = MessageChannel::default();
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WindowEvent>(message_channel.get_messagebox());
            global_messenger
                .write()
                .unwrap()
                .register_messagebox::<ResourceEvent>(message_channel.get_messagebox());

        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            job_handler,
            config: config.clone(),
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
    fn init(&mut self) {
        for pipeline_data in self.config.get_pipelines().iter() {
            PipelineInstance::create(&self.shared_data, pipeline_data);
        }
    }

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Init && state != RendererState::Submitted {
            return true;
        }

        let mut should_recreate_swap_chain = false;
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<WindowEvent>() {
                let e = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                if let WindowEvent::SizeChanged(_width, _height) = e {
                    should_recreate_swap_chain = true;
                }
            } else if msg.type_id() == TypeId::of::<ResourceEvent>() {
                let e = msg.as_any().downcast_ref::<ResourceEvent>().unwrap();
                let ResourceEvent::Reload(path) = e;
                if path.extension().unwrap() == SHADER_EXTENSION {
                    let pipelines =
                    SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
                    for p in pipelines.iter() {
                            p.get_mut().check_shaders_to_reload(path.to_str().unwrap().to_string());                        
                    }
                }
            }
        });

        {
            let mut renderer = self.renderer.write().unwrap();
            let mut pipelines =
                SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
            let mut materials =
                SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);
            let mut textures =
                SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);

            if should_recreate_swap_chain {
                renderer.recreate();
            }

            renderer.prepare_frame(&mut pipelines, &mut materials, &mut textures);
        }
        
        let wait_count = Arc::new(AtomicUsize::new(0));

        let materials = SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);

        materials
            .iter()
            .enumerate()
            .for_each(|(material_index, material_instance)| {
                if material_instance.get().has_meshes() {
                    let diffuse_texture_id = material_instance.get().get_diffuse_texture();
                    let diffuse_color = material_instance.get().get_diffuse_color();
                    let outline_color = material_instance.get().get_outline_color();
                    let pipeline_id = material_instance.get().get_pipeline_id();

                    let (diffuse_texture_handler_index, diffuse_texture_index,  diffuse_layer_index )= if diffuse_texture_id.is_nil() {
                        (INVALID_INDEX, INVALID_INDEX, INVALID_INDEX)
                    } else {
                        let texture_instance =
                        SharedData::get_resource::<TextureInstance>(&self.shared_data, diffuse_texture_id);
                        let (thi, ti, li) =
                        (texture_instance.get().get_texture_handler_index() as _,
                        texture_instance.get().get_texture_index() as _,
                        texture_instance.get().get_layer_index() as _);
                        (thi, ti, li)
                    };

                    material_instance.get().get_meshes().iter().enumerate().for_each(
                        |(mesh_index, mesh_id)| {
                            let mesh_instance = SharedData::get_resource::<MeshInstance>(&self.shared_data, *mesh_id);
                            if mesh_instance.get().is_visible() {
                                let mesh_id = *mesh_id;
                                let shared_data = self.shared_data.clone();
                                let r = self.renderer.clone();
                                let wait_count = wait_count.clone();

                                wait_count.fetch_add(1, Ordering::SeqCst);

                                let job_name = format!(
                                    "PrepareMaterial [{}] with mesh [{}]",
                                    material_index, mesh_index
                                );
                                self.job_handler.write().unwrap().add_job(job_name.as_str(), 
                                    move || {                    
                                        
                                        let mesh_instance =
                                        SharedData::get_resource::<MeshInstance>(&shared_data, mesh_id);

                                        if diffuse_texture_handler_index >= 0 {
                                            let renderer = r.read().unwrap();
                                            let diffuse_texture =
                                                renderer
                                                    .get_texture_handler()
                                                    .get_texture(diffuse_texture_handler_index as _);
                                            mesh_instance.get_mut().process_uv_for_texture(Some(diffuse_texture));
                                        } else {
                                            mesh_instance.get_mut().process_uv_for_texture(None);
                                        }
                                        let mut renderer = r.write().unwrap();
                                        let pipeline = renderer
                                            .get_pipelines()
                                            .iter_mut()
                                            .find(|p| p.id() == pipeline_id)
                                            .unwrap();
                                        pipeline.add_mesh_instance(
                                            &mesh_instance.get(),
                                            diffuse_color,
                                            diffuse_texture_index,
                                            diffuse_layer_index,
                                            outline_color,
                                        );

                                        wait_count.fetch_sub(1, Ordering::SeqCst);
                                    },
                                );
                            }
                        },
                    );
                }
            });

        let renderer = self.renderer.clone();
        let job_name = "EndPreparation";
        self.job_handler.write().unwrap().add_job(job_name, move || {
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
