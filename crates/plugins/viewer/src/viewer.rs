use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    platform::has_primitive_index_support, rendering_system::RenderingSystem,
    update_system::UpdateSystem, BlitPass, ComputePbrPass, CullingPass, DebugDrawerSystem,
    GBufferPass, LoadOperation, PBRPass, Pass, RenderPass, RenderTarget, Renderer, RendererRw,
    TextureFormat, VisibilityBufferPass, WireframePass, DEFAULT_HEIGHT, DEFAULT_WIDTH,
    GBUFFER_PASS_NAME, WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::read_from_file;
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_WIREFRAME_PASS: bool = true;
const ADD_UI_PASS: bool = true;
const USE_GBUFFER: bool = false;
const USE_3DVIEW: bool = false;

pub struct Viewer {
    window: Option<Window>,
    renderer: RendererRw,
}
define_plugin!(Viewer);

impl Plugin for Viewer {
    fn create(context: &ContextRc) -> Self {
        let window = {
            Window::create(
                "INOX Engine".to_string(),
                0,
                0,
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
                PathBuf::from("").as_path(),
                context.message_hub(),
            )
        };
        let renderer = Renderer::new(window.handle(), context, false);

        Self::create_render_passes(context, &renderer, DEFAULT_WIDTH, DEFAULT_HEIGHT);

        Viewer {
            window: Some(window),
            renderer,
        }
    }

    fn name(&self) -> &str {
        "inox_viewer"
    }

    fn prepare(&mut self, context: &ContextRc) {
        let window_system = WindowSystem::new(self.window.take().unwrap(), context);
        let render_update_system = UpdateSystem::new(self.renderer.clone(), context);
        let rendering_draw_system = RenderingSystem::new(self.renderer.clone(), context);
        let mut ui_system = if ADD_UI_PASS {
            Some(UISystem::new(context))
        } else {
            None
        };

        let viewer_system = ViewerSystem::new(context, &self.renderer, USE_3DVIEW);
        let object_system = ObjectSystem::new(context);
        let script_system = ScriptSystem::new(context);

        context.add_system(inox_core::Phases::PlatformUpdate, window_system, None);
        context.add_system(
            inox_core::Phases::Render,
            render_update_system,
            Some(&[RenderingSystem::system_id()]),
        );
        context.add_system(
            inox_core::Phases::Render,
            rendering_draw_system,
            Some(&[UpdateSystem::system_id()]),
        );

        context.add_system(inox_core::Phases::Update, object_system, None);
        context.add_system(
            inox_core::Phases::Update,
            script_system,
            Some(&[ObjectSystem::system_id()]),
        );

        if let Some(ui_system) = ui_system.take() {
            context.add_system(inox_core::Phases::Update, ui_system, None);
        }
        if ADD_WIREFRAME_PASS {
            let debug_drawer_system = DebugDrawerSystem::new(context);
            context.add_system(inox_core::Phases::Update, debug_drawer_system, None);
        }
        context.add_system(inox_core::Phases::Update, viewer_system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::Update, &ViewerSystem::system_id());
        if ADD_WIREFRAME_PASS {
            context.remove_system(inox_core::Phases::Update, &DebugDrawerSystem::system_id());
        }
        if ADD_UI_PASS {
            context.remove_system(inox_core::Phases::Update, &UISystem::system_id());
        }

        context.remove_system(inox_core::Phases::Update, &ScriptSystem::system_id());
        context.remove_system(inox_core::Phases::Update, &ObjectSystem::system_id());

        context.remove_system(
            inox_core::Phases::PlatformUpdate,
            &WindowSystem::system_id(),
        );
        context.remove_system(inox_core::Phases::Render, &UpdateSystem::system_id());
        context.remove_system(inox_core::Phases::Render, &RenderingSystem::system_id());
    }

    fn load_config(&mut self, context: &ContextRc) {
        let config = Config::default();
        let shared_data = context.shared_data().clone();

        read_from_file(
            config.get_filepath(self.name()).as_path(),
            context.shared_data().serializable_registry(),
            Box::new(move |data: Config| {
                if let Some(ui_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.name() == UI_PASS_NAME)
                {
                    ui_pass.get_mut().set_pipeline(&data.ui_pass_pipeline);
                }
                if let Some(default_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.name() == GBUFFER_PASS_NAME)
                {
                    default_pass
                        .get_mut()
                        .set_pipeline(&data.opaque_pass_pipeline);
                }
                if let Some(wireframe_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.name() == WIREFRAME_PASS_NAME)
                {
                    wireframe_pass
                        .get_mut()
                        .set_pipeline(&data.wireframe_pass_pipeline);
                }
            }),
        );
    }
}

impl Viewer {
    fn create_render_passes(context: &ContextRc, renderer: &RendererRw, width: u32, height: u32) {
        let use_gbuffer = USE_GBUFFER || !has_primitive_index_support();

        Self::create_gbuffer_pass(context, renderer, width, height, use_gbuffer);
        Self::create_pbr_pass(context, renderer, use_gbuffer);

        Self::create_culling_pass(context, renderer, !use_gbuffer);
        Self::create_visibility_buffer_pass(context, renderer, width, height, !use_gbuffer);
        Self::create_compute_pbr_pass(context, renderer, width, height, !use_gbuffer);
        Self::create_blit_pass(context, renderer, !use_gbuffer);

        Self::create_wireframe_pass(context, renderer, ADD_WIREFRAME_PASS);

        Self::create_ui_pass(context, renderer, width, height, ADD_UI_PASS);
    }
    fn create_gbuffer_pass(
        context: &ContextRc,
        renderer: &RendererRw,
        width: u32,
        height: u32,
        is_enabled: bool,
    ) {
        let gbuffer_pass = GBufferPass::create(context);

        gbuffer_pass
            .render_pass()
            .get_mut()
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_depth_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Depth32Float,
                read_back: false,
            });

        renderer.write().unwrap().add_pass(gbuffer_pass, is_enabled);
    }
    fn create_pbr_pass(context: &ContextRc, renderer: &RendererRw, is_enabled: bool) {
        let mut pbr_pass = PBRPass::create(context);

        if let Some(gbuffer_pass) = renderer.read().unwrap().pass::<GBufferPass>() {
            pbr_pass.set_gbuffers_textures(
                gbuffer_pass
                    .render_pass()
                    .get()
                    .render_textures_id()
                    .as_slice(),
            );
            pbr_pass
                .set_depth_texture(gbuffer_pass.render_pass().get().depth_texture_id().unwrap());
        }
        renderer.write().unwrap().add_pass(pbr_pass, is_enabled);
    }
    fn create_visibility_buffer_pass(
        context: &ContextRc,
        renderer: &RendererRw,
        width: u32,
        height: u32,
        is_enabled: bool,
    ) {
        let visibility_pass = VisibilityBufferPass::create(context);
        visibility_pass
            .render_pass()
            .get_mut()
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
                read_back: false,
            })
            .add_depth_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Depth32Float,
                read_back: false,
            });
        renderer
            .write()
            .unwrap()
            .add_pass(visibility_pass, is_enabled);
    }
    fn create_compute_pbr_pass(
        context: &ContextRc,
        renderer: &RendererRw,
        width: u32,
        height: u32,
        is_enabled: bool,
    ) {
        let mut compute_pbr_pass = ComputePbrPass::create(context);
        compute_pbr_pass.add_render_target_with_resolution(width, height);
        if let Some(visibility_pass) = renderer.read().unwrap().pass::<VisibilityBufferPass>() {
            let pass = visibility_pass.render_pass().get();
            pass.render_textures_id().iter().for_each(|&id| {
                compute_pbr_pass.add_texture(id);
            });
        }
        renderer
            .write()
            .unwrap()
            .add_pass(compute_pbr_pass, is_enabled);
    }
    fn create_blit_pass(context: &ContextRc, renderer: &RendererRw, is_enabled: bool) {
        let mut blit_pass = BlitPass::create(context);
        if let Some(pbr_pass) = renderer.read().unwrap().pass::<ComputePbrPass>() {
            blit_pass.set_source(pbr_pass.render_target_id());
        }
        renderer.write().unwrap().add_pass(blit_pass, is_enabled);
    }
    fn create_wireframe_pass(context: &ContextRc, renderer: &RendererRw, is_enabled: bool) {
        let wireframe_pass = WireframePass::create(context);
        renderer
            .write()
            .unwrap()
            .add_pass(wireframe_pass, is_enabled);
    }
    fn create_ui_pass(
        context: &ContextRc,
        renderer: &RendererRw,
        width: u32,
        height: u32,
        is_enabled: bool,
    ) {
        let ui_pass = UIPass::create(context);
        if USE_3DVIEW {
            if let Some(blit_pass) = renderer.read().unwrap().pass::<BlitPass>() {
                blit_pass
                    .render_pass()
                    .get_mut()
                    .add_render_target(RenderTarget::Texture {
                        width,
                        height,
                        format: TextureFormat::Rgba8Unorm,
                        read_back: false,
                    });
            }
        } else {
            let mut ui_pass = ui_pass.render_pass().get_mut();
            ui_pass.set_load_color_operation(LoadOperation::Load);
        }
        renderer.write().unwrap().add_pass(ui_pass, is_enabled);
    }
    fn create_culling_pass(context: &ContextRc, renderer: &RendererRw, is_enabled: bool) {
        let culling_pass = CullingPass::create(context);
        renderer.write().unwrap().add_pass(culling_pass, is_enabled);
    }
}
