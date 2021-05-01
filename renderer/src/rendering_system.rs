use crate::config::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_events::EventsRw;

pub struct RenderingSystem {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
}

impl RenderingSystem {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
        }
    }
}

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        {
            let renderer = {
                let read_data = self.shared_data.read().unwrap();
                let window = &*read_data.get_unique_resource::<Window>();
                let mut renderer = Renderer::new(
                    window.get_handle(),
                    self.config.vk_data.debug_validation_layers,
                );
                let size = Vector2::new(window.get_width() as _, window.get_heigth() as _);
                renderer.set_viewport_size(size);
                renderer
            };
            self.shared_data.write().unwrap().add_resource(renderer);
        }

        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        for pipeline_data in self.config.pipelines.iter() {
            renderer.add_pipeline(pipeline_data);
        }
    }

    fn run(&mut self) -> bool {
        let read_data = self.shared_data.read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        let mut should_recreate_swap_chain = false;
        let events = events_rw.read().unwrap();
        if let Some(window_events) = events.read_all_events::<WindowEvent>() {
            for event in window_events {
                if let WindowEvent::SizeChanged(_width, _height) = event {
                    should_recreate_swap_chain = true;
                }
            }
        }

        if should_recreate_swap_chain {
            renderer.recreate();
        } else if renderer.begin_frame() {
            renderer.draw();
            renderer.end_frame();
        }

        true
    }
    fn uninit(&mut self) {
        self.shared_data
            .write()
            .unwrap()
            .request_remove_resources_of_type::<Renderer>();
    }
}
