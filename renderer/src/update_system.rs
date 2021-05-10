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
use nrg_resources::{SharedData, SharedDataRw};

pub struct UpdateSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
    events_rw: EventsRw,
    config: Config,
}

impl UpdateSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRw,
        events_rw: &EventsRw,
        config: &Config,
    ) -> Self {
        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            events_rw: events_rw.clone(),
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
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Init && state != RendererState::Submitted {
            return (true, Vec::new());
        }

        let should_recreate_swap_chain = {
            let events = self.events_rw.read().unwrap();
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

        let mut jobs = Vec::new();
        let wait_count = Arc::new(AtomicUsize::new(0));

        let materials = SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);

        materials
            .iter()
            .enumerate()
            .for_each(|(material_index, material_instance)| {
                if material_instance.get().has_meshes() {
                    let diffuse_texture_id = material_instance.get().get_diffuse_texture();
                    let diffuse_color = material_instance.get().get_diffuse_color();
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

                                jobs.push(Job::new(
                                    format!(
                                        "PrepareMaterial [{}] with mesh [{}]",
                                        material_index, mesh_index
                                    ),
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
