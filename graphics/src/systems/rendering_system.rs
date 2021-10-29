use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::{JobHandlerRw, System};
use nrg_math::Matrix4;
use nrg_messenger::MessengerRw;
use nrg_resources::{DataTypeResource, Resource, SharedData, SharedDataRc};
use nrg_serialize::generate_random_uid;

use crate::{Pipeline, PipelineId, RenderPass, RendererRw, RendererState, View};

pub const RENDERING_PHASE: &str = "RENDERING_PHASE";

pub struct RenderingSystem {
    view: Resource<View>,
    renderer: RendererRw,
    job_handler: JobHandlerRw,
    shared_data: SharedDataRc,
}

impl RenderingSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        job_handler: &JobHandlerRw,
    ) -> Self {
        Self {
            view: View::create_from_data(shared_data, global_messenger, generate_random_uid(), 0),
            renderer,
            job_handler: job_handler.clone(),
            shared_data: shared_data.clone(),
        }
    }

    fn draw_pipeline(
        renderer: &RendererRw,
        render_pass: &RenderPass,
        pipeline: &mut Pipeline,
        view: &Matrix4,
        proj: &Matrix4,
    ) {
        let renderer = renderer.read().unwrap();
        let device = renderer.device();
        let physical_device = renderer.instance().get_physical_device();
        let texture_handler = renderer.get_texture_handler();
        let width = render_pass.get_framebuffer_width();
        let height = render_pass.get_framebuffer_height();

        pipeline.bind(render_pass.get_command_buffer());

        let textures = texture_handler.get_textures_atlas();
        let material_data = renderer.material_data();
        debug_assert!(textures.is_empty() == false);
        let used_textures = pipeline.find_used_textures(textures, material_data);

        pipeline
            .update_bindings(
                device,
                render_pass.get_command_buffer(),
                width,
                height,
                &view,
                &proj,
                textures,
                used_textures.as_slice(),
                renderer.light_data(),
                renderer.texture_data(),
                renderer.material_data(),
            )
            .fill_command_buffer(device, physical_device, render_pass.get_command_buffer());
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

impl System for RenderingSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        {
            self.renderer
                .write()
                .unwrap()
                .change_state(RendererState::Drawing);
        }

        let wait_count = Arc::new(AtomicUsize::new(0));

        let view = self.view.get(|v| v.view());
        let proj = self.view.get(|v| v.proj());

        let mut render_pass_specific_pipeline: Vec<PipelineId> = Vec::new();
        SharedData::for_each_resource(&self.shared_data, |_, render_pass: &RenderPass| {
            if let Some(pipeline) = render_pass.pipeline() {
                render_pass_specific_pipeline.push(*pipeline.id());
            }
        });
        SharedData::for_each_resource(
            &self.shared_data,
            |render_pass_handle, render_pass: &RenderPass| {
                if render_pass.is_initialized() {
                    let job_name = format!("Draw RenderPass {:?}", render_pass.data().name);
                    let renderer = self.renderer.clone();
                    let shared_data = self.shared_data.clone();
                    let render_pass = render_pass_handle.clone();
                    let render_pass_specific_pipeline = render_pass_specific_pipeline.clone();
                    let wait_count = wait_count.clone();
                    wait_count.fetch_add(1, Ordering::SeqCst);

                    self.job_handler.write().unwrap().add_job(
                        &RenderingSystem::id(),
                        job_name.as_str(),
                        move || {
                            render_pass.get_mut(|render_pass: &mut RenderPass| {
                                nrg_profiler::scoped_profile!(format!(
                                    "fill_command_buffer_for_render_pass[{}]",
                                    render_pass.data().name
                                )
                                .as_str());

                                {
                                    let mut renderer = renderer.write().unwrap();

                                    render_pass.acquire_command_buffer(renderer.device_mut());
                                }

                                {
                                    let renderer = renderer.read().unwrap();
                                    render_pass.begin_command_buffer(renderer.device());
                                }

                                if let Some(pipeline) = render_pass.pipeline() {
                                    pipeline.get_mut(|pipeline| {
                                        if pipeline.is_initialized() {
                                            Self::draw_pipeline(
                                                &renderer,
                                                &render_pass,
                                                pipeline,
                                                &view,
                                                &proj,
                                            );
                                        }
                                    });
                                } else {
                                    shared_data.for_each_resource_mut(
                                        |pipeline_handle, pipeline: &mut Pipeline| {
                                            let should_render = {
                                                pipeline.is_initialized()
                                                    && !render_pass_specific_pipeline
                                                        .iter()
                                                        .any(|id| id == pipeline_handle.id())
                                            };
                                            if should_render {
                                                Self::draw_pipeline(
                                                    &renderer,
                                                    &render_pass,
                                                    pipeline,
                                                    &view,
                                                    &proj,
                                                );
                                            }
                                        },
                                    );
                                }

                                {
                                    let renderer = renderer.read().unwrap();
                                    render_pass.end_command_buffer(renderer.device());
                                }
                            });

                            wait_count.fetch_sub(1, Ordering::SeqCst);
                        },
                    );
                }
            },
        );

        let renderer = self.renderer.clone();
        let shared_data = self.shared_data.clone();
        let job_name = "EndDraw";

        self.job_handler
            .write()
            .unwrap()
            .add_job(&RenderingSystem::id(), job_name, move || {
                while wait_count.load(Ordering::SeqCst) > 0 {
                    thread::yield_now();
                }

                {
                    let mut renderer = renderer.write().unwrap();
                    renderer.begin_frame();
                }

                SharedData::for_each_resource_mut(
                    &shared_data,
                    |_, render_pass: &mut RenderPass| {
                        if render_pass.is_initialized() {
                            nrg_profiler::scoped_profile!(format!(
                                "draw_render_pass[{}]",
                                render_pass.data().name
                            )
                            .as_str());
                            let renderer = renderer.read().unwrap();
                            render_pass.draw(renderer.device());
                        }
                    },
                );

                {
                    let mut renderer = renderer.write().unwrap();
                    renderer.end_frame();

                    let success = renderer.present();
                    if !success {
                        renderer.recreate();
                    }

                    renderer.change_state(RendererState::Submitted);
                }
            });

        true
    }
    fn uninit(&mut self) {}
}
