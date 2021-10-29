use crate::{
    Device, Instance, Light, LightData, Material, Pipeline, RenderPass, ShaderMaterialData,
    ShaderTextureData, Texture, TextureHandler,
};
use nrg_resources::DataTypeResource;

use nrg_platform::Handle;
use nrg_resources::{SharedData, SharedDataRc};

use std::sync::{Arc, RwLock};

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
    light_data: Vec<LightData>,
    texture_data: Vec<ShaderTextureData>,
    material_data: Vec<ShaderMaterialData>,
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
            light_data: Vec::new(),
            texture_data: Vec::new(),
            material_data: Vec::new(),
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

    pub fn light_data(&self) -> &[LightData] {
        self.light_data.as_slice()
    }
    pub fn texture_data(&self) -> &[ShaderTextureData] {
        self.texture_data.as_slice()
    }
    pub fn material_data(&self) -> &[ShaderMaterialData] {
        self.material_data.as_slice()
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
        self.init_materials();
        self.init_lights();

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
                if let Some(texture_data) = texture_handler.get_texture_data(texture_handle.id()) {
                    let uniform_index = self.texture_data.len();
                    texture.set_texture_data(
                        uniform_index,
                        texture_data.get_width(),
                        texture_data.get_height(),
                    );
                    self.texture_data.push(texture_data);
                } else {
                    let width = texture.width();
                    let height = texture.height();
                    if let Some(image_data) = texture.image_data() {
                        let texture_data = texture_handler.add_image(
                            device,
                            physical_device,
                            texture_handle.id(),
                            width,
                            height,
                            image_data,
                        );
                        let uniform_index = self.texture_data.len();
                        texture.set_texture_data(
                            uniform_index,
                            texture_data.get_width(),
                            texture_data.get_height(),
                        );
                        self.texture_data.push(texture_data);
                    }
                }
            }
        });
    }

    fn init_lights(&mut self) {
        nrg_profiler::scoped_profile!("renderer::init_lights");
        self.light_data.clear();
        self.shared_data.for_each_resource(|_id, light: &Light| {
            if light.is_active() {
                self.light_data.push(*light.data());
            }
        });
    }
    fn init_materials(&mut self) {
        nrg_profiler::scoped_profile!("renderer::init_materials");
        self.shared_data
            .for_each_resource_mut(|_id, material: &mut Material| {
                if !material.is_initialized() {
                    let uniform_index = self.material_data.len() as i32;
                    self.material_data
                        .push(material.create_uniform_material_data());
                    material.set_uniform_index(uniform_index);
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
