use std::any::TypeId;

use nrg_core::{System, SystemId};
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_resources::{ResourceEvent, SharedData, SharedDataRw};

use crate::{
    is_shader, is_texture, FontInstance, MaterialInstance, MeshInstance, PipelineInstance,
    RenderPassInstance, RendererRw, RendererState, TextureInstance, INVALID_INDEX,
};

pub struct UpdateSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
    message_channel: MessageChannel,
}

impl UpdateSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
    ) -> Self {
        let message_channel = MessageChannel::default();
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<ResourceEvent>(message_channel.get_messagebox());

        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
            message_channel,
        }
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

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<ResourceEvent>() {
                let e = msg.as_any().downcast_ref::<ResourceEvent>().unwrap();
                let ResourceEvent::Reload(path) = e;
                if is_shader(path)
                    && SharedData::has_resources_of_type::<PipelineInstance>(&self.shared_data)
                {
                    let pipelines =
                        SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
                    for p in pipelines.iter() {
                        p.resource()
                            .get_mut()
                            .check_shaders_to_reload(path.to_str().unwrap().to_string());
                    }
                } else if is_texture(path)
                    && SharedData::has_resources_of_type::<TextureInstance>(&self.shared_data)
                {
                    let textures =
                        SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);
                    for t in textures.iter() {
                        if t.resource().get().path() == path.as_path() {
                            t.resource().get_mut().invalidate();
                        }
                    }
                }
            }
        });

        {
            let mut renderer = self.renderer.write().unwrap();
            let mut render_passes =
                SharedData::get_resources_of_type::<RenderPassInstance>(&self.shared_data);
            let mut pipelines =
                SharedData::get_resources_of_type::<PipelineInstance>(&self.shared_data);
            let mut materials =
                SharedData::get_resources_of_type::<MaterialInstance>(&self.shared_data);

            let mut textures =
                SharedData::get_resources_of_type::<TextureInstance>(&self.shared_data);
            let fonts = SharedData::get_resources_of_type::<FontInstance>(&self.shared_data);

            renderer.prepare_frame(
                &mut render_passes,
                &mut pipelines,
                &mut materials,
                &mut textures,
                &fonts,
            );
        }

        if SharedData::has_resources_of_type::<MeshInstance>(&self.shared_data) {
            let mut meshes = SharedData::get_resources_of_type::<MeshInstance>(&self.shared_data);
            meshes.iter_mut().for_each(|mesh| {
                if mesh.resource().get().is_visible() {
                    let material = mesh.resource().get().material();
                    if material.id().is_nil() {
                        eprintln!("Tyring to render a mesh with an unregistered material");
                        return;
                    }
                    let pipeline = material.resource().get().pipeline();
                    if pipeline.id().is_nil() {
                        eprintln!(
                            "Tyring to render a mesh with a material with an unregistered pipeline"
                        );
                        return;
                    }

                    let diffuse_color = material.resource().get().diffuse_color();
                    let outline_color = material.resource().get().outline_color();

                    let (diffuse_texture_index, diffuse_layer_index) =
                        if material.resource().get().has_diffuse_texture() {
                            let diffuse_texture = material.resource().get().diffuse_texture();
                            let (diffuse_texture_index, diffuse_layer_index) = (
                                diffuse_texture.resource().get().texture_index() as _,
                                diffuse_texture.resource().get().layer_index() as _,
                            );
                            let r = self.renderer.read().unwrap();
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

                    {
                        for pipeline in self
                            .renderer
                            .write()
                            .unwrap()
                            .get_pipelines_with_id(pipeline.id())
                            .iter_mut()
                        {
                            pipeline.add_mesh_instance(
                                &mesh.resource().get(),
                                diffuse_color,
                                diffuse_texture_index,
                                diffuse_layer_index,
                                outline_color,
                            );
                        }
                    }
                }
            });
        }
        self.renderer.write().unwrap().end_preparation();

        true
    }
    fn uninit(&mut self) {}
}
