use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_resources::{ResourceEvent, SharedData, SharedDataRw};

use crate::{
    is_shader, is_texture, Mesh, MeshCategoryId, MeshRc, Pipeline, RenderPass, RendererRw,
    RendererState, Texture, INVALID_INDEX,
};

pub struct UpdateSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    message_channel: MessageChannel,
}

impl UpdateSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        let message_channel = MessageChannel::default();
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<ResourceEvent>(message_channel.get_messagebox());

        crate::register_resource_types(shared_data);
        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            job_handler,
            message_channel,
        }
    }

    fn handle_events(&self) {
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<ResourceEvent>() {
                let e = msg.as_any().downcast_ref::<ResourceEvent>().unwrap();
                let ResourceEvent::Reload(path) = e;
                if is_shader(path)
                    && SharedData::has_resources_of_type::<Pipeline>(&self.shared_data)
                {
                    let pipelines =
                        SharedData::get_resources_of_type::<Pipeline>(&self.shared_data);
                    for p in pipelines.iter() {
                        p.resource()
                            .get_mut()
                            .check_shaders_to_reload(path.to_str().unwrap().to_string());
                    }
                } else if is_texture(path)
                    && SharedData::has_resources_of_type::<Texture>(&self.shared_data)
                {
                    let textures = SharedData::get_resources_of_type::<Texture>(&self.shared_data);
                    for t in textures.iter() {
                        if t.resource().get().path() == path.as_path() {
                            t.resource().get_mut().invalidate();
                        }
                    }
                }
            }
        });
    }

    fn create_render_mesh_job(
        renderer: RendererRw,
        mesh: MeshRc,
        mesh_category_to_draw: &[MeshCategoryId],
    ) {
        nrg_profiler::scoped_profile!(format!("create_render_mesh_job[{}]", mesh.id()).as_str());

        let should_render = mesh_category_to_draw
            .iter()
            .any(|id| mesh.resource().get().category_identifier() == *id);
        if !should_render {
            return;
        }
        let material = mesh.resource().get().material();
        let material_pipeline = material.resource().get().pipeline().clone();
        if material.id().is_nil() {
            eprintln!("Tyring to render a mesh with no material");
            return;
        }
        if material_pipeline.id().is_nil() {
            eprintln!("Tyring to render a mesh with a material with no pipeline");
            return;
        }
        if !mesh.resource().get().is_visible() {
            return;
        }

        let diffuse_color = material.resource().get().diffuse_color();

        let (diffuse_texture_index, diffuse_layer_index) =
            if material.resource().get().has_diffuse_texture() {
                nrg_profiler::scoped_profile!("Obtaining texture info");
                let diffuse_texture = material.resource().get().diffuse_texture();
                let (diffuse_texture_index, diffuse_layer_index) = (
                    diffuse_texture.resource().get().texture_index() as _,
                    diffuse_texture.resource().get().layer_index() as _,
                );
                let r = renderer.read().unwrap();
                let texture_info = r
                    .get_texture_handler()
                    .get_texture_info(diffuse_texture.id());
                mesh.resource()
                    .get_mut()
                    .process_uv_for_texture(texture_info);
                (diffuse_texture_index, diffuse_layer_index)
            } else {
                (INVALID_INDEX, INVALID_INDEX)
            };

        material_pipeline.resource().get_mut().add_mesh_instance(
            renderer.read().unwrap().device(),
            &mesh.resource().get(),
            diffuse_color,
            diffuse_texture_index,
            diffuse_layer_index,
        );
    }
}

impl Drop for UpdateSystem {
    fn drop(&mut self) {
        crate::unregister_resource_types(&self.shared_data);
    }
}

unsafe impl Send for UpdateSystem {}
unsafe impl Sync for UpdateSystem {}

impl System for UpdateSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Init && state != RendererState::Submitted {
            return true;
        }

        self.handle_events();

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.prepare_frame();
        }

        let wait_count = Arc::new(AtomicUsize::new(0));

        let render_passes = SharedData::get_resources_of_type::<RenderPass>(&self.shared_data);
        render_passes.iter().for_each(|render_pass| {
            let mesh_category_to_draw = render_pass
                .resource()
                .get()
                .mesh_category_to_draw()
                .to_vec();
            let meshes = SharedData::get_resources_of_type::<Mesh>(&self.shared_data);
            meshes.iter().for_each(|mesh| {
                let renderer = self.renderer.clone();
                let wait_count = wait_count.clone();
                let mesh = mesh.clone();
                let mesh_category_to_draw = mesh_category_to_draw.clone();
                let job_name = format!(
                    "Processing mesh {:?} for RenderPass [{:?}",
                    mesh.id(),
                    render_pass.resource().get().data().name
                );
                self.job_handler
                    .write()
                    .unwrap()
                    .add_job(job_name.as_str(), move || {
                        wait_count.fetch_add(1, Ordering::SeqCst);
                        Self::create_render_mesh_job(renderer, mesh, &mesh_category_to_draw);
                        wait_count.fetch_sub(1, Ordering::SeqCst);
                    });
            });
        });

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
