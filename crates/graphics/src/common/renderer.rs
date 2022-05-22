use crate::{
    ComputePipeline, GetRenderContext, Light, LightId, Material, MaterialId, Mesh, MeshId, Pass,
    RenderContext, RenderContextRw, RenderPass, RenderPassId, RenderPipeline, Texture, TextureId,
};
use inox_core::ContextRc;
use inox_math::Matrix4;
use inox_messenger::MessageHubRc;
use inox_resources::{DataTypeResource, Resource};

use inox_platform::Handle;
use inox_resources::{SharedData, SharedDataRc};

use std::sync::{Arc, RwLock};

pub const DEFAULT_WIDTH: u32 = 3840;
pub const DEFAULT_HEIGHT: u32 = 2160;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Init,
    Preparing,
    Prepared,
    Drawing,
    Submitted,
}

pub struct Renderer {
    context: RenderContextRw,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    state: RendererState,
    passes: Vec<Box<dyn Pass>>,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Drop for Renderer {
    fn drop(&mut self) {
        crate::unregister_resource_types(&self.shared_data, &self.message_hub);
    }
}

impl Renderer {
    pub fn new(handle: &Handle, context: &ContextRc, _enable_debug: bool) -> Self {
        crate::register_resource_types(context.shared_data(), context.message_hub());

        let render_context = Arc::new(RwLock::new(None));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(RenderContext::create_render_context(
            handle.clone(),
            context.clone(),
            render_context.clone(),
        ));

        #[cfg(all(not(target_arch = "wasm32")))]
        futures::executor::block_on(RenderContext::create_render_context(
            handle.clone(),
            context.clone(),
            render_context.clone(),
        ));

        Renderer {
            state: RendererState::Init,
            context: render_context,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            passes: Vec::new(),
        }
    }

    pub fn render_context(&self) -> &RenderContextRw {
        &self.context
    }
    pub fn passes(&self) -> &[Box<dyn Pass>] {
        self.passes.as_slice()
    }
    pub fn pass<T>(&self, index: usize) -> Option<&T>
    where
        T: Pass,
    {
        if index >= self.passes.len() {
            return None;
        }
        self.passes[index].downcast_ref::<T>()
    }
    pub fn pass_mut<T>(&mut self, index: usize) -> Option<&mut T>
    where
        T: Pass,
    {
        if index >= self.passes.len() {
            return None;
        }
        self.passes[index].downcast_mut::<T>()
    }

    pub fn add_pass(&mut self, pass: impl Pass) -> &mut Self {
        self.passes.push(Box::new(pass));
        self
    }

    pub fn check_initialization(&mut self) {
        if self.context.read().unwrap().is_none() {
            self.state = RendererState::Init;
        } else {
            self.state = RendererState::Submitted;
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        (
            self.context.get().as_ref().unwrap().config.width,
            self.context.get().as_ref().unwrap().config.height,
        )
    }

    pub fn state(&self) -> RendererState {
        self.state
    }
    pub fn change_state(&mut self, render_state: RendererState) -> &mut Self {
        self.state = render_state;
        self
    }

    pub fn need_redraw(&self) -> bool {
        self.state != RendererState::Submitted
    }

    pub fn recreate(&self) {
        inox_profiler::scoped_profile!("renderer::recreate");

        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, pipeline: &mut RenderPipeline| {
                pipeline.invalidate();
            },
        );
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, pipeline: &mut ComputePipeline| {
                pipeline.invalidate();
            },
        );
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, render_pass: &mut RenderPass| {
                render_pass.invalidate();
            },
        );
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        let mut context = self.context.get_mut();
        let context = context.as_mut().unwrap();
        context.config.width = width;
        context.config.height = height;
        context.surface.configure(&context.device, &context.config);
        inox_log::debug_log!("Surface size: {}x{}", width, height);
        self.recreate();
    }

    pub fn on_texture_changed(
        &mut self,
        texture_id: &TextureId,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        inox_profiler::scoped_profile!("renderer::on_texture_changed");
        let mut render_context = self.context.get_mut();
        let render_context = render_context.as_mut().unwrap();
        let texture_handler = &mut render_context.texture_handler;

        if let Some(texture) = self.shared_data.get_resource::<Texture>(texture_id) {
            if !texture.get().is_initialized() {
                if texture_handler.texture_index(texture_id) == None {
                    let width = texture.get().width();
                    let height = texture.get().height();
                    if let Some(image_data) = texture.get().image_data() {
                        texture_handler.add_image(
                            &render_context.device,
                            encoder,
                            texture_id,
                            (width, height),
                            image_data,
                        );
                    }
                }
                if let Some(texture_data) = texture_handler.get_texture_data(texture_id) {
                    let uniform_index = render_context.add_texture_data(texture_id, texture_data);
                    texture.get_mut().set_texture_data(
                        uniform_index,
                        texture_data.width(),
                        texture_data.height(),
                    );
                    //Need to update all materials that use this texture
                    self.shared_data
                        .for_each_resource_mut(|_, m: &mut Material| {
                            if m.has_texture_id(texture_id) {
                                m.mark_as_dirty();
                            }
                        });
                }
            }
        }
    }

    pub fn on_light_changed(&mut self, light_id: &LightId) {
        inox_profiler::scoped_profile!("renderer::on_light_changed");
        if let Some(light) = self.shared_data.get_resource::<Light>(light_id) {
            let mut render_context = self.context.get_mut();
            let render_context = render_context.as_mut().unwrap();
            let uniform_index = render_context.add_light_data(light_id, *light.get().data());
            light.get_mut().update_uniform(uniform_index as _);
        }
    }

    pub fn on_render_pipeline_changed(&mut self, pipeline_id: &MaterialId) {
        inox_profiler::scoped_profile!("renderer::on_pipeline_changed");
        if let Some(pipeline) = self.shared_data.get_resource::<RenderPipeline>(pipeline_id) {
            let vertex_format = pipeline.get().vertex_format();
            let render_context = self.context.get();
            let render_context = render_context.as_ref().unwrap();
            render_context
                .graphics_data
                .get_mut()
                .set_pipeline_vertex_format(pipeline.id(), vertex_format);
        }
    }

    pub fn on_material_changed(&mut self, material_id: &MaterialId) {
        inox_profiler::scoped_profile!("renderer::on_material_changed");
        if let Some(material) = self.shared_data.get_resource::<Material>(material_id) {
            let mut render_context = self.context.get_mut();
            let render_context = render_context.as_mut().unwrap();
            let uniform_index = render_context.add_material_data(material_id, &material.get());
            material.get_mut().update_uniform(uniform_index as _);
            //Need to update all meshes that use this material
            self.shared_data.for_each_resource_mut(|_, m: &mut Mesh| {
                if let Some(material) = m.material() {
                    if material.id() == material_id {
                        m.mark_as_dirty();
                    }
                }
            });
        }
    }

    pub fn on_render_pass_changed(&mut self, render_pass_id: &RenderPassId) {
        inox_profiler::scoped_profile!("renderer::on_render_pass_changed");
        let mut render_context = self.context.get_mut();
        let render_context = render_context.as_mut().unwrap();
        if let Some(render_pass) = self.shared_data.get_resource::<RenderPass>(render_pass_id) {
            if !render_pass.get().is_initialized() {
                render_pass.get_mut().init(render_context);
            }
        }
    }

    pub fn on_mesh_added(&mut self, mesh: &Resource<Mesh>) {
        inox_profiler::scoped_profile!("renderer::on_mesh_added");
        let render_context = self.context.get();
        let render_context = render_context.as_ref().unwrap();
        render_context
            .graphics_data
            .get_mut()
            .update_mesh(mesh.id(), &mesh.get());
    }
    pub fn on_mesh_changed(&mut self, mesh_id: &MeshId) {
        inox_profiler::scoped_profile!("renderer::on_mesh_changed");
        if let Some(mesh) = self.shared_data.get_resource::<Mesh>(mesh_id) {
            let render_context = self.context.get();
            let render_context = render_context.as_ref().unwrap();
            render_context
                .graphics_data
                .get_mut()
                .update_mesh(mesh.id(), &mesh.get());
        }
    }
    pub fn on_mesh_removed(&mut self, mesh_id: &MeshId) {
        inox_profiler::scoped_profile!("renderer::on_mesh_removed");
        let render_context = self.context.get();
        let render_context = render_context.as_ref().unwrap();
        render_context.graphics_data.get_mut().remove_mesh(mesh_id);
    }

    pub fn obtain_surface_texture(&mut self) {
        let surface_texture = {
            inox_profiler::scoped_profile!("wgpu::get_current_texture");

            self.context
                .get()
                .as_ref()
                .unwrap()
                .surface
                .get_current_texture()
        };
        if let Ok(output) = surface_texture {
            let screen_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.context.get_mut().as_mut().unwrap().surface_view = Some(screen_view);
            self.context.get_mut().as_mut().unwrap().surface_texture = Some(output);
        }
    }

    pub fn send_to_gpu(&mut self, encoder: wgpu::CommandEncoder) {
        inox_profiler::scoped_profile!("renderer::send_to_gpu");

        let mut render_context = self.context.get_mut();
        let render_context = render_context.as_mut().unwrap();

        if render_context.graphics_data.get().total_vertex_count() == 0 {
            return;
        }

        self.passes.iter_mut().for_each(|pass| {
            pass.init(render_context);
        });

        let graphics_data = &mut render_context.graphics_data.get_mut();
        graphics_data.send_to_gpu(render_context);

        render_context.submit(encoder);
    }

    pub fn update_passes(&mut self) {
        inox_profiler::scoped_profile!("renderer::execute_passes");

        let render_context = self.context.get();
        let render_context = render_context.as_ref().unwrap();
        if render_context.graphics_data.get().total_vertex_count() == 0 {
            return;
        }
        self.passes.iter_mut().for_each(|pass| {
            pass.update(render_context);
        });
    }

    pub fn present(&self) {
        inox_profiler::scoped_profile!("renderer::present");

        let surface_texture = self
            .context
            .get_mut()
            .as_mut()
            .unwrap()
            .surface_texture
            .take();
        if let Some(surface_texture) = surface_texture {
            surface_texture.present();
        }
    }
}
