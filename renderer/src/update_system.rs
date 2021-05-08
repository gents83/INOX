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
            let mut meshes = data.get_resources_of_type_mut::<MeshInstance>();
            let mut textures = data.get_resources_of_type_mut::<TextureInstance>();

            if should_recreate_swap_chain {
                renderer.recreate();
            }

            renderer.prepare_frame(&mut pipelines, &mut materials, &mut meshes, &mut textures);
        }

        (true, Vec::new())
    }
    fn uninit(&mut self) {}
}
