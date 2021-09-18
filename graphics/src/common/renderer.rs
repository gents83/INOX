use crate::{is_texture, Device, Font, Instance, Pipeline, RenderPass, Texture, TextureHandler};

use nrg_platform::Handle;
use nrg_resources::{FileResource, Resource, ResourceData, SharedData, SharedDataRw, DATA_FOLDER};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub const INVALID_INDEX: i32 = -1;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Preparing,
    Prepared,
    Drawing,
    Submitted,
}

pub struct Renderer {
    instance: Instance,
    device: Device,
    shared_data: SharedDataRw,
    texture_handler: TextureHandler,
    state: RendererState,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(handle: &Handle, shared_data: &SharedDataRw, enable_debug: bool) -> Self {
        let instance = Instance::create(handle, enable_debug);
        let device = Device::create(&instance, enable_debug);
        let texture_handler = TextureHandler::create(&device, instance.get_physical_device());
        Renderer {
            shared_data: shared_data.clone(),
            instance,
            device,
            texture_handler,
            state: RendererState::Submitted,
        }
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    pub fn state(&self) -> RendererState {
        self.state
    }
    pub fn change_state(&mut self, render_state: RendererState) -> &mut Self {
        self.state = render_state;
        self
    }

    pub fn prepare_frame(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("renderer::prepare_frame");
        self.init_render_passes();
        self.init_pipelines_for_pass("MainPass");
        self.init_textures();

        self
    }

    pub fn get_texture_handler(&self) -> &TextureHandler {
        &self.texture_handler
    }
    pub fn get_texture_handler_mut(&mut self) -> &mut TextureHandler {
        &mut self.texture_handler
    }

    pub fn need_redraw(&self) -> bool {
        self.state != RendererState::Submitted
    }

    pub fn begin_frame(&mut self) {
        nrg_profiler::scoped_profile!("renderer::begin_frame");

        self.device.begin_frame();
    }

    pub fn end_frame(&self) {
        nrg_profiler::scoped_profile!("renderer::end_frame");

        self.device.end_frame();
        self.device.submit();
    }
    pub fn present(&mut self) -> bool {
        nrg_profiler::scoped_profile!("renderer::present");
        self.device.present()
    }

    pub fn recreate(&mut self) {
        nrg_profiler::scoped_profile!("renderer::recreate");

        self.device.recreate_swap_chain(&mut self.instance);

        SharedData::for_each_resource(&self.shared_data, |pipeline: &Resource<Pipeline>| {
            pipeline.get_mut().invalidate();
        });
        SharedData::for_each_resource(&self.shared_data, |render_pass: &Resource<RenderPass>| {
            render_pass.get_mut().invalidate()
        });
    }
}

impl Renderer {
    fn init_render_passes(&mut self) {
        nrg_profiler::scoped_profile!("renderer::init_render_passes");
        let device = &mut self.device;
        let physical_device = self.instance.get_physical_device();
        let texture_handler = &mut self.texture_handler;
        SharedData::for_each_resource(&self.shared_data, |render_pass: &Resource<RenderPass>| {
            if !render_pass.get().is_initialized() {
                render_pass
                    .get_mut()
                    .init(device, physical_device, texture_handler);
            }
            if let Some(pipeline) = render_pass.get().pipeline() {
                pipeline.get_mut().prepare();
            }
        });
    }
    fn init_pipelines_for_pass(&mut self, render_pass_name: &str) {
        nrg_profiler::scoped_profile!("renderer::init_pipelines");
        let geometry_render_pass =
            SharedData::match_resource(&self.shared_data, |render_pass: &RenderPass| {
                render_pass.data().name == render_pass_name
            });
        if let Some(geometry_render_pass) = geometry_render_pass {
            SharedData::for_each_resource(&self.shared_data, |pipeline: &Resource<Pipeline>| {
                if !pipeline.get().is_initialized() {
                    pipeline
                        .get_mut()
                        .init(&self.device, &*geometry_render_pass.get());
                }
                pipeline.get_mut().prepare();
            });
        }
    }

    fn init_textures(&mut self) {
        nrg_profiler::scoped_profile!("renderer::init_textures");
        let device = &mut self.device;
        let physical_device = &self.instance.get_physical_device();
        let texture_handler = &mut self.texture_handler;
        let shared_data = &self.shared_data;
        SharedData::for_each_resource(shared_data, |texture: &Resource<Texture>| {
            if !texture.get().is_initialized() {
                if texture.get().texture_index() != INVALID_INDEX {
                    //texture needs to be recreated
                    texture_handler.remove(device, texture.id());
                }
                let path = convert_from_local_path(
                    PathBuf::from(DATA_FOLDER).as_path(),
                    texture.get().path(),
                );
                if let Some(texture_info) = texture_handler.get_texture_info(texture.id()) {
                    texture.get_mut().set_texture_info(texture_info);
                } else {
                    let texture_info = if let Some(image_data) = texture.get_mut().image_data() {
                        texture_handler.add(device, physical_device, texture.id(), image_data)
                    } else if is_texture(path.as_path()) {
                        texture_handler.add_from_path(
                            device,
                            physical_device,
                            texture.id(),
                            path.as_path(),
                        )
                    } else {
                        let font =
                            SharedData::match_resource(shared_data, |f: &Font| f.path() == path);
                        if let Some(font) = font {
                            texture_handler.add(
                                device,
                                physical_device,
                                texture.id(),
                                font.get().font_data().get_texture(),
                            )
                        } else {
                            panic!("Unable to load texture with path {:?}", path.as_path());
                        }
                    };
                    texture.get_mut().set_texture_info(&texture_info);
                }
            }
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();
        self.instance.destroy();
    }
}
