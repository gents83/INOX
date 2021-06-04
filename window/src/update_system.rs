use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use crate::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::{ResourceEvent, ResourceRc, ResourceTrait, SharedData, SharedDataRw};

pub struct UpdateSystem {
    id: SystemId,
    is_enabled: bool,
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
            is_enabled: false,
            shared_data: shared_data.clone(),
            job_handler,
            config: config.clone(),
            message_channel,
        }
    }

    fn load_fonts(&self, fonts: &[ResourceRc<FontInstance>]) {
        nrg_profiler::scoped_profile!("update_system::load_fonts");

        fonts.iter().for_each(|font_instance| {
            let material_id = font_instance.get().material();

            if !MaterialInstance::has_textures(&self.shared_data, material_id) {
                let font_path = font_instance.get().path();
                let texture_id = TextureInstance::find_id(&self.shared_data, font_path.as_path());
                if texture_id.is_nil() {
                    TextureInstance::create_from_path(&self.shared_data, font_path.as_path());
                }
                MaterialInstance::add_texture(&self.shared_data, material_id, texture_id);
            }
        });
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
                match e {
                    WindowEvent::SizeChanged(_width, _height) => {
                        should_recreate_swap_chain = true;
                    }
                    WindowEvent::Show => {
                        self.is_enabled = true;
                    }
                    WindowEvent::Hide => {
                        self.is_enabled = false;
                    }
                    _ => {}
                }
            } else if msg.type_id() == TypeId::of::<ResourceEvent>() {
                let e = msg.as_any().downcast_ref::<ResourceEvent>().unwrap();
                let ResourceEvent::Reload(path) = e;
                if is_shader(path)
                    && SharedData::has_resources_of_type::<PipelineInstance>(&self.shared_data)
                {
                    let pipelines =
                        SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
                    for p in pipelines.iter() {
                        p.get_mut()
                            .check_shaders_to_reload(path.to_str().unwrap().to_string());
                    }
                } else if is_texture(path)
                    && SharedData::has_resources_of_type::<TextureInstance>(&self.shared_data)
                {
                    let textures =
                        SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);
                    for t in textures.iter() {
                        if t.get().get_path() == path.as_path() {
                            t.get_mut().invalidate();
                        }
                    }
                }
            }
        });

        if self.is_enabled && SharedData::has_resources_of_type::<FontInstance>(&self.shared_data) {
            let fonts = SharedData::get_resources_of_type::<FontInstance>(&self.shared_data);
            self.load_fonts(&fonts);
        }

        if self.is_enabled
            && SharedData::has_resources_of_type::<PipelineInstance>(&self.shared_data)
            && SharedData::has_resources_of_type::<MaterialInstance>(&self.shared_data)
            && SharedData::has_resources_of_type::<TextureInstance>(&self.shared_data)
            && SharedData::has_resources_of_type::<FontInstance>(&self.shared_data)
        {
            let mut renderer = self.renderer.write().unwrap();
            let mut pipelines =
                SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
            let mut materials =
                SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);
            let mut textures =
                SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);
            let fonts = SharedData::get_resources_of_type::<FontInstance>(&self.shared_data);

            if should_recreate_swap_chain {
                renderer.recreate();
                for p in pipelines.iter() {
                    p.get_mut().invalidate();
                }
            }

            renderer.prepare_frame(&mut pipelines, &mut materials, &mut textures, &fonts);
        }

        let wait_count = Arc::new(AtomicUsize::new(0));

        if self.is_enabled
            && SharedData::has_resources_of_type::<MaterialInstance>(&self.shared_data)
        {
            let materials =
                SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);

            materials
                .iter()
                .enumerate()
                .for_each(|(material_index, material_instance)| {
                    if material_instance.get().has_meshes() {
                        let diffuse_texture_id = material_instance.get().get_diffuse_texture();
                        let diffuse_color = material_instance.get().get_diffuse_color();
                        let outline_color = material_instance.get().get_outline_color();
                        let pipeline_id = material_instance.get().get_pipeline_id();

                        let (diffuse_texture_index, diffuse_layer_index) =
                            if diffuse_texture_id.is_nil() {
                                (INVALID_INDEX, INVALID_INDEX)
                            } else {
                                let texture_instance = SharedData::get_resource::<TextureInstance>(
                                    &self.shared_data,
                                    diffuse_texture_id,
                                );
                                let (ti, li) = (
                                    texture_instance.get().get_texture_index() as _,
                                    texture_instance.get().get_layer_index() as _,
                                );
                                (ti, li)
                            };

                        material_instance
                            .get()
                            .get_meshes()
                            .iter()
                            .enumerate()
                            .for_each(|(mesh_index, mesh_id)| {
                                let mesh_instance = SharedData::get_resource::<MeshInstance>(
                                    &self.shared_data,
                                    *mesh_id,
                                );
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
                                    self.job_handler.write().unwrap().add_job(
                                        job_name.as_str(),
                                        move || {
                                            let mesh_instance = SharedData::get_resource::<
                                                MeshInstance,
                                            >(
                                                &shared_data, mesh_id
                                            );

                                            if !diffuse_texture_id.is_nil() {
                                                let renderer = r.read().unwrap();
                                                let diffuse_texture = renderer
                                                    .get_texture_handler()
                                                    .get_texture(diffuse_texture_id);
                                                mesh_instance
                                                    .get_mut()
                                                    .process_uv_for_texture(Some(diffuse_texture));
                                            } else {
                                                mesh_instance
                                                    .get_mut()
                                                    .process_uv_for_texture(None);
                                            }
                                            let mut renderer = r.write().unwrap();
                                            if let Some(pipeline) = renderer
                                                .get_pipelines()
                                                .iter_mut()
                                                .find(|p| p.id() == pipeline_id)
                                            {
                                                pipeline.add_mesh_instance(
                                                    &mesh_instance.get(),
                                                    diffuse_color,
                                                    diffuse_texture_index,
                                                    diffuse_layer_index,
                                                    outline_color,
                                                );
                                            } else {
                                                eprintln!("Tyring to render with an unregistered pipeline {}", pipeline_id.to_simple().to_string());
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
