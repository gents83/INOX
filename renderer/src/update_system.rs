use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use crate::config::*;

use nrg_core::*;
use nrg_events::EventsRw;
use nrg_graphics::*;
use nrg_platform::WindowEvent;
use nrg_resources::SharedDataRw;

pub struct UpdateSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
    config: Config,
}

impl UpdateSystem {
    pub fn new(renderer: RendererRw, shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            config: config.clone(),
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
        for pipeline_data in self.config.pipelines.iter() {
            PipelineInstance::create(&self.shared_data, pipeline_data);
        }
    }

    fn run(&mut self) -> (bool, Vec<Job>) {
        let state = self.renderer.read().unwrap().get_state();
        if state != RendererState::Init && state != RendererState::Submitted {
            return (true, Vec::new());
        }

        let should_recreate_swap_chain = {
            let read_data = self.shared_data.read().unwrap();
            let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
            let events = events_rw.read().unwrap();
            let mut size_changed = false;
            if let Some(window_events) = events.read_all_events::<WindowEvent>() {
                for event in window_events {
                    if let WindowEvent::SizeChanged(_width, _height) = event {
                        size_changed = true;
                    }
                }
            }
            size_changed
        };

        {
            let mut renderer = self.renderer.write().unwrap();
            let data = self.shared_data.read().unwrap();
            let mut pipelines = data.get_resources_of_type_mut::<PipelineInstance>();
            let mut materials = data.get_resources_of_type_mut::<MaterialInstance>();
            let mut textures = data.get_resources_of_type_mut::<TextureInstance>();

            if should_recreate_swap_chain {
                renderer.recreate();
            }

            renderer.prepare_frame(&mut pipelines, &mut materials, &mut textures);
        }

        let mut jobs = Vec::new();
        let wait_count = Arc::new(AtomicUsize::new(0));

        let data = self.shared_data.read().unwrap();
        let materials = data.get_resources_of_type_mut::<MaterialInstance>();

        materials
            .iter()
            .enumerate()
            .for_each(|(material_index, material_instance)| {
                if material_instance.has_meshes() {
                    let diffuse_texture_id = material_instance.get_diffuse_texture();
                    let diffuse_color = material_instance.get_diffuse_color();
                    let pipeline_id = material_instance.get_pipeline_id();

                    let (diffuse_texture_handler_index, diffuse_texture_index,  diffuse_layer_index )= if diffuse_texture_id.is_nil() {
                        (INVALID_INDEX, INVALID_INDEX, INVALID_INDEX)
                    } else {
                        let texture_instance =
                        data.get_resource::<TextureInstance>(diffuse_texture_id);
                        (texture_instance.get_texture_handler_index() as _,
                        texture_instance.get_texture_index() as _,
                        texture_instance.get_layer_index() as _)
                    };

                    material_instance.get_meshes().iter().enumerate().for_each(
                        |(mesh_index, mesh_id)| {
                            let mesh_instance = data.get_resource::<MeshInstance>(*mesh_id);
                            if mesh_instance.is_visible() {
                                let mesh_id = *mesh_id;
                                let shared_data = self.shared_data.clone();
                                let r = self.renderer.clone();
                                let wait_count = wait_count.clone();

                                wait_count.fetch_add(1, Ordering::SeqCst);

                                jobs.push(Job::new(
                                    format!(
                                        "PrepareMaterial [{}] with mesh [{}]",
                                        material_index, mesh_index
                                    ),
                                    move || {
                                        let data = shared_data.read().unwrap();
                                        let mut mesh_instance =
                                            data.get_resource_mut::<MeshInstance>(mesh_id);

                                        let mut renderer = r.write().unwrap();
                                        let diffuse_texture = if diffuse_texture_handler_index >= 0 {
                                            Some(
                                                renderer
                                                    .get_texture_handler()
                                                    .get_texture(diffuse_texture_handler_index as _),
                                            )
                                        } else {
                                            None
                                        };
                                        mesh_instance.process_uv_for_texture(diffuse_texture);
                                        let pipeline = renderer
                                            .get_pipelines()
                                            .iter_mut()
                                            .find(|p| p.id() == pipeline_id)
                                            .unwrap();
                                        pipeline.add_mesh_instance(
                                            &mesh_instance,
                                            diffuse_color,
                                            diffuse_texture_index,
                                            diffuse_layer_index,
                                        );

                                        wait_count.fetch_sub(1, Ordering::SeqCst);
                                    },
                                ));
                            }
                        },
                    );
                }
            });

        let renderer = self.renderer.clone();
        jobs.push(Job::new("EndPreparation".to_string(), move || {
            while wait_count.load(Ordering::SeqCst) > 0 {
                thread::yield_now();
            }
            let mut r = renderer.write().unwrap();
            r.end_preparation();
        }));

        (true, jobs)
    }
    fn uninit(&mut self) {}
}
