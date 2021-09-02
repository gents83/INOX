use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_resources::{DataTypeResource, ResourceRef, SharedData, SharedDataRw};

use crate::{Pipeline, PipelineId, RenderPass, RendererRw, RendererState, View, ViewRc};

pub struct RenderingSystem {
    id: SystemId,
    view_index: usize,
    view: ViewRc,
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
            view_index: 0,
            view: ResourceRef::default(),
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
    fn init(&mut self) {
        if !SharedData::has_resources_of_type::<View>(&self.shared_data) {
            self.view = View::create_from_data(&self.shared_data, self.view_index as _);
        }
    }

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        let mut renderer = self.renderer.write().unwrap();

        let success = renderer.begin_frame();
        if !success {
            renderer.recreate();
            return true;
        }

        let wait_count = Arc::new(AtomicUsize::new(0));

        let view = self.view.resource().get().view();
        let proj = self.view.resource().get().proj();

        let render_passes = SharedData::get_resources_of_type::<RenderPass>(&self.shared_data);
        let mut render_pass_specific_pipeline: Vec<PipelineId> = Vec::new();
        render_passes.iter().for_each(|render_pass| {
            if let Some(pipeline) = render_pass.resource().get().pipeline() {
                render_pass_specific_pipeline.push(pipeline.id());
            }
        });
        render_passes.iter().for_each(|render_pass| {
            let job_name = format!(
                "Draw RenderPass [{:?}",
                render_pass.resource().get().data().name
            );
            let renderer = self.renderer.clone();
            let shared_data = self.shared_data.clone();
            let render_pass = render_pass.clone();
            let wait_count = wait_count.clone();
            self.job_handler
                .write()
                .unwrap()
                .add_job(job_name.as_str(), move || {
                    wait_count.fetch_add(1, Ordering::SeqCst);
                    nrg_profiler::scoped_profile!(format!(
                        "renderer::render_pass[{}]",
                        render_pass.resource().get().data().name
                    )
                    .as_str());

                    let mut renderer = renderer.write().unwrap();

                    render_pass.resource().get().begin(renderer.device_mut());

                    let width = render_pass.resource().get().get_framebuffer_width();
                    let height = render_pass.resource().get().get_framebuffer_height();

                    let pipelines = SharedData::get_resources_of_type::<Pipeline>(&shared_data);
                    pipelines.iter().for_each(|pipeline| {
                        if pipeline
                            .resource()
                            .get()
                            .should_draw(render_pass.resource().get().mesh_category_to_draw())
                        {
                            let texture_atlas = renderer.get_texture_handler().get_textures_atlas();
                            pipeline
                                .resource()
                                .get_mut()
                                .update_bindings(width, height, &view, &proj, texture_atlas)
                                .draw(renderer.device_mut());
                        }
                    });

                    render_pass.resource().get().end(renderer.device_mut());

                    wait_count.fetch_sub(1, Ordering::SeqCst);
                });
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
