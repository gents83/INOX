use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_math::Matrix4;
use nrg_resources::{DataTypeResource, Resource, ResourceData, SharedData, SharedDataRw};

use crate::{
    api::backend::BackendPhysicalDevice, Device, Pipeline, PipelineId, RenderPass, RendererRw,
    RendererState, TextureHandler, View,
};

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

    fn draw_pipeline(
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        texture_handler: &TextureHandler,
        render_pass: &RenderPass,
        pipeline: &Resource<Pipeline>,
        width: u32,
        height: u32,
        view: &Matrix4,
        proj: &Matrix4,
    ) {
        pipeline.get_mut().bind(render_pass.get_command_buffer());

        let textures = texture_handler.get_textures_atlas();
        debug_assert!(textures.is_empty() == false);
        let used_textures = pipeline.get().find_used_textures(textures);

        pipeline
            .get_mut()
            .update_bindings(
                device,
                render_pass.get_command_buffer(),
                width,
                height,
                &view,
                &proj,
                textures,
                used_textures.as_slice(),
            )
            .fill_command_buffer(device, physical_device, render_pass.get_command_buffer());
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

        self.renderer
            .write()
            .unwrap()
            .change_state(RendererState::Drawing);

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
                let job_name = format!("Draw RenderPass {:?}", render_pass.get().data().name);
                let renderer = self.renderer.clone();
                let render_pass = render_pass.clone();
                let shared_data = self.shared_data.clone();
                let render_pass_specific_pipeline = render_pass_specific_pipeline.clone();
                let wait_count = wait_count.clone();
                wait_count.fetch_add(1, Ordering::SeqCst);
                self.job_handler
                    .write()
                    .unwrap()
                    .add_job(job_name.as_str(), move || {
                        nrg_profiler::scoped_profile!(format!(
                            "fill_command_buffer_for_render_pass[{}]",
                            render_pass.get().data().name
                        )
                        .as_str());

                        {
                            let mut renderer = renderer.write().unwrap();

                            render_pass
                                .get_mut()
                                .acquire_command_buffer(renderer.device_mut());
                        }

                        let renderer = renderer.read().unwrap();
                        let render_pass = render_pass.get();
                        let instance = renderer.instance();
                        let device = renderer.device();
                        let width = render_pass.get_framebuffer_width();
                        let height = render_pass.get_framebuffer_height();
                        let texture_handler = renderer.get_texture_handler();

                        render_pass.begin_command_buffer(device);

                        if let Some(pipeline) = render_pass.pipeline() {
                            Self::draw_pipeline(
                                device,
                                instance.get_physical_device(),
                                &texture_handler,
                                &render_pass,
                                pipeline,
                                width,
                                height,
                                &view,
                                &proj,
                            );
                        } else {
                            SharedData::for_each_resource(
                                &shared_data,
                                |pipeline: &Resource<Pipeline>| {
                                    let should_render = {
                                        let pipeline = pipeline.get();
                                        pipeline.is_initialized()
                                            && !render_pass_specific_pipeline
                                                .iter()
                                                .any(|id| *id == pipeline.id())
                                    };
                                    if should_render {
                                        Self::draw_pipeline(
                                            device,
                                            instance.get_physical_device(),
                                            &texture_handler,
                                            &render_pass,
                                            pipeline,
                                            width,
                                            height,
                                            &view,
                                            &proj,
                                        );
                                    }
                                },
                            );
                        }

                        render_pass.end_command_buffer(device);

                        wait_count.fetch_sub(1, Ordering::SeqCst);
                    });
            }
        });

        let renderer = self.renderer.clone();
        let shared_data = self.shared_data.clone();
        let job_name = "EndDraw";

        self.job_handler
            .write()
            .unwrap()
            .add_job(job_name, move || {
                while wait_count.load(Ordering::SeqCst) > 0 {
                    thread::yield_now();
                }

                {
                    let mut renderer = renderer.write().unwrap();
                    renderer.begin_frame();
                }

                {
                    let renderer = renderer.read().unwrap();
                    let device = renderer.device();

                    SharedData::for_each_resource(
                        &shared_data,
                        |render_pass: &Resource<RenderPass>| {
                            if render_pass.get().is_initialized() {
                                nrg_profiler::scoped_profile!(format!(
                                    "draw_render_pass[{}]",
                                    render_pass.get().data().name
                                )
                                .as_str());
                                render_pass.get_mut().draw(device);
                            }
                        },
                    );
                }

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
