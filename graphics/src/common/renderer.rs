use crate::{Device, Instance, Pipeline, RenderPass, Texture, TextureHandler};
use nrg_resources::DataTypeResource;

use nrg_platform::Handle;
use nrg_resources::{SharedData, SharedDataRc};

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
    shared_data: SharedDataRc,
    texture_handler: TextureHandler,
    state: RendererState,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(handle: &Handle, shared_data: &SharedDataRc, enable_debug: bool) -> Self {
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

        SharedData::for_each_resource_mut(&self.shared_data, |_id, pipeline: &mut Pipeline| {
            pipeline.invalidate();
        });
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, render_pass: &mut RenderPass| render_pass.invalidate(),
        );
    }
}

impl Renderer {
    fn init_render_passes(&mut self) {
        nrg_profiler::scoped_profile!("renderer::init_render_passes");
        let device = &mut self.device;
        let physical_device = self.instance.get_physical_device();
        let texture_handler = &mut self.texture_handler;
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, render_pass: &mut RenderPass| {
                if !render_pass.is_initialized() {
                    render_pass.init(device, physical_device, texture_handler);
                }
                if let Some(pipeline) = render_pass.pipeline() {
                    pipeline.get_mut(|p| {
                        if !p.is_initialized() {
                            p.init(device, physical_device, render_pass);
                        }
                        p.prepare();
                    });
                }
            },
        );
    }
    fn init_pipelines_for_pass(&mut self, render_pass_name: &str) {
        nrg_profiler::scoped_profile!("renderer::init_pipelines");
        let geometry_render_pass =
            SharedData::match_resource(&self.shared_data, |render_pass: &RenderPass| {
                render_pass.data().name == render_pass_name
            });
        if let Some(geometry_render_pass) = geometry_render_pass {
            let device = &self.device;
            let physical_device = self.instance.get_physical_device();
            SharedData::for_each_resource_mut(&self.shared_data, |_id, p: &mut Pipeline| {
                if !p.is_initialized() {
                    geometry_render_pass.get(|render_pass| {
                        p.init(device, physical_device, render_pass);
                    });
                }
                p.prepare();
            });
        }
    }

    fn init_textures(&mut self) {
        nrg_profiler::scoped_profile!("renderer::init_textures");
        let device = &mut self.device;
        let physical_device = &self.instance.get_physical_device();
        let texture_handler = &mut self.texture_handler;
        let shared_data = &self.shared_data;
        SharedData::for_each_resource_mut(shared_data, |texture_handle, texture: &mut Texture| {
            if !texture.is_initialized() {
                if texture.texture_index() != INVALID_INDEX {
                    //texture needs to be recreated
                    texture_handler.remove(device, texture_handle.id());
                }
                if let Some(texture_info) = texture_handler.get_texture_info(texture_handle.id()) {
                    texture.set_texture_info(texture_info);
                } else {
                    let width = texture.width();
                    let height = texture.height();
                    if let Some(image_data) = texture.image_data() {
                        let texture_info = texture_handler.add(
                            device,
                            physical_device,
                            texture_handle.id(),
                            width,
                            height,
                            image_data,
                        );
                        texture.set_texture_info(&texture_info);
                    }
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
