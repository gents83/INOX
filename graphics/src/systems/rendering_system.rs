use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_resources::{DataTypeResource, Resource, ResourceData, SharedData, SharedDataRw};

use crate::{Pipeline, PipelineId, RenderPass, RendererRw, RendererState, View};

pub struct RenderingSystem {
    id: SystemId,
    view: Resource<View>,
    renderer: RendererRw,
    job_handler: JobHandlerRw,
    shared_data: SharedDataRw,
}

impl RenderingSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        Self {
            id: SystemId::new(),
            view: View::create_from_data(shared_data, 0),
            renderer,
            job_handler,
            shared_data: shared_data.clone(),
        }
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        let mut renderer = self.renderer.write().unwrap();

        let success = renderer.begin_frame();
        if !success {
            renderer.recreate();
            renderer.end_draw();
            return true;
        }

        let wait_count = Arc::new(AtomicUsize::new(0));

        let view = self.view.get().view();
        let proj = self.view.get().proj();

        let mut render_pass_specific_pipeline: Vec<PipelineId> = Vec::new();
        SharedData::for_each_resource(&self.shared_data, |render_pass: &Resource<RenderPass>| {
            if let Some(pipeline) = render_pass.get().pipeline() {
                render_pass_specific_pipeline.push(pipeline.id());
            }
        });
        SharedData::for_each_resource(&self.shared_data, |render_pass: &Resource<RenderPass>| {
            if render_pass.get().is_initialized() {
                let job_name = format!("Draw RenderPass [{:?}", render_pass.get().data().name);
                let renderer = self.renderer.clone();
                let render_pass = render_pass.clone();
                let shared_data = self.shared_data.clone();
                let wait_count = wait_count.clone();
                self.job_handler
                    .write()
                    .unwrap()
                    .add_job(job_name.as_str(), move || {
                        wait_count.fetch_add(1, Ordering::SeqCst);
                        let render_pass = render_pass.get();

                        nrg_profiler::scoped_profile!(format!(
                            "renderer::render_pass[{}]",
                            render_pass.data().name
                        )
                        .as_str());

                        {
                            let mut renderer = renderer.write().unwrap();
                            let device = renderer.device_mut();
                            render_pass.acquire_command_buffer(device);
                        }

                        let renderer = renderer.read().unwrap();
                        let device = renderer.device();
                        render_pass.begin(device);

                        let width = render_pass.get_framebuffer_width();
                        let height = render_pass.get_framebuffer_height();

                        SharedData::for_each_resource(
                            &shared_data,
                            |pipeline: &Resource<Pipeline>| {
                                let should_render = {
                                    let pipeline = pipeline.get();
                                    pipeline.is_initialized()
                                        && pipeline.should_draw(render_pass.mesh_category_to_draw())
                                };
                                if should_render {
                                    let texture_atlas =
                                        renderer.get_texture_handler().get_textures_atlas();
                                    pipeline
                                        .get_mut()
                                        .update_bindings(width, height, &view, &proj, texture_atlas)
                                        .draw(device);
                                }
                            },
                        );

                        render_pass.end(device);

                        wait_count.fetch_sub(1, Ordering::SeqCst);
                    });
            }
        });

        let renderer = self.renderer.clone();
        let job_name = "EndDraw";
        self.job_handler
            .write()
            .unwrap()
            .add_job(job_name, move || {
                while wait_count.load(Ordering::SeqCst) > 0 {
                    thread::yield_now();
                }

                let mut r = renderer.write().unwrap();
                r.end_frame();

                let success = r.present();
                if !success {
                    r.recreate();
                }

                r.end_draw();
            });

        true
    }
    fn uninit(&mut self) {}
}
