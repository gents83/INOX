use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    platform::has_wireframe_support, rendering_system::RenderingSystem,
    update_system::UpdateSystem, BlitPass, ComputeFinalizePass, ComputePathTracingDirectPass,
    ComputePathTracingIndirectPass, ComputeRuntimeVerticesPass, CullingPass, LoadOperation,
    OutputPass, OutputRenderPass, Pass, RenderPass, RenderTarget, Renderer, RendererRw,
    TextureFormat, TextureId, VisibilityBufferPass, WireframePass, DEFAULT_HEIGHT, DEFAULT_WIDTH,
    WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::read_from_file;
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_CULLING_PASS: bool = true;
const ADD_UI_PASS: bool = true;
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
        let context_rc = context.clone();
        let renderer = Renderer::new(window.handle(), context, move |renderer| {
            Self::create_render_passes(&context_rc, renderer, DEFAULT_WIDTH, DEFAULT_HEIGHT);
        });

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
        context.add_system(inox_core::Phases::Update, viewer_system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::Update, &ViewerSystem::system_id());
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
    fn create_render_passes(context: &ContextRc, renderer: &mut Renderer, width: u32, height: u32) {
        let reduced_dimensions = (width / 4, height / 4);

        Self::create_compute_runtime_vertices_pass(context, renderer, true);
        Self::create_culling_pass(context, renderer, ADD_CULLING_PASS);

        let (visibility_texture_id, depth_texture_id) = Self::create_visibility_pass(
            context,
            renderer,
            reduced_dimensions.0,
            reduced_dimensions.1,
        );
        let render_targets_ids = Self::create_compute_pathtracing_direct_pass(
            context,
            renderer,
            reduced_dimensions.0,
            reduced_dimensions.1,
            &visibility_texture_id,
            &depth_texture_id,
        );
        let radiance_texture_id = Self::create_compute_pathtracing_indirect_pass(
            context,
            renderer,
            reduced_dimensions.0,
            reduced_dimensions.1,
            &render_targets_ids[0],
            &render_targets_ids[1],
        );
        let output_texture_id = Self::create_compute_finalize_pass(
            context,
            renderer,
            width,
            height,
            &visibility_texture_id,
            &radiance_texture_id,
            &depth_texture_id,
        );
        Self::create_blit_pass(context, renderer, &output_texture_id);

        Self::create_wireframe_pass(context, renderer, has_wireframe_support());
        Self::create_ui_pass(context, renderer, width, height, ADD_UI_PASS);
    }
    fn create_compute_runtime_vertices_pass(
        context: &ContextRc,
        renderer: &mut Renderer,
        is_enabled: bool,
    ) {
        if !is_enabled {
            return;
        }
        let compute_runtime_vertices_pass =
            ComputeRuntimeVerticesPass::create(context, &renderer.render_context());
        renderer.add_pass(compute_runtime_vertices_pass, is_enabled);
    }
    fn create_culling_pass(context: &ContextRc, renderer: &mut Renderer, is_enabled: bool) {
        if !is_enabled {
            return;
        }
        let culling_pass = CullingPass::create(context, &renderer.render_context());
        renderer.add_pass(culling_pass, is_enabled);
    }
    fn create_compute_pathtracing_direct_pass(
        context: &ContextRc,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
        visibility_texture_id: &TextureId,
        depth_texture_id: &TextureId,
    ) -> Vec<TextureId> {
        let mut compute_pathtracing_direct_pass =
            ComputePathTracingDirectPass::create(context, &renderer.render_context());
        compute_pathtracing_direct_pass
            .add_radiance_render_target_with_resolution(width, height, TextureFormat::Rgba32Float)
            .add_ray_render_target_with_resolution(width, height, TextureFormat::Rgba32Float);
        compute_pathtracing_direct_pass
            .set_visibility_texture(visibility_texture_id)
            .set_depth_texture(depth_texture_id);
        let render_targets_id = compute_pathtracing_direct_pass.render_targets_id().unwrap();
        renderer.add_pass(compute_pathtracing_direct_pass, true);
        render_targets_id
    }
    fn create_compute_pathtracing_indirect_pass(
        context: &ContextRc,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
        radiance_texture_id: &TextureId,
        rays_texture_id: &TextureId,
    ) -> TextureId {
        let mut compute_pathtracing_indirect_pass =
            ComputePathTracingIndirectPass::create(context, &renderer.render_context());
        compute_pathtracing_indirect_pass
            .set_radiance_texture(radiance_texture_id, width, height)
            .set_ray_texture(rays_texture_id);
        renderer.add_pass(compute_pathtracing_indirect_pass, true);
        *radiance_texture_id
    }
    fn create_visibility_pass(
        context: &ContextRc,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
    ) -> (TextureId, TextureId) {
        let visibility_pass = VisibilityBufferPass::create(context, &renderer.render_context());
        visibility_pass
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba8Unorm,
            })
            .add_depth_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Depth32Float,
            });
        let render_target_id = *visibility_pass
            .render_targets_id()
            .unwrap()
            .first()
            .unwrap();
        let depth_target_id = visibility_pass.depth_target_id().unwrap();
        renderer.add_pass(visibility_pass, true);
        (render_target_id, depth_target_id)
    }
    fn create_wireframe_pass(context: &ContextRc, renderer: &mut Renderer, is_enabled: bool) {
        if !is_enabled {
            return;
        }
        let wireframe_pass = WireframePass::create(context, &renderer.render_context());
        renderer.add_pass(wireframe_pass, is_enabled);
    }
    fn create_ui_pass(
        context: &ContextRc,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
        is_enabled: bool,
    ) {
        if !is_enabled {
            return;
        }
        let ui_pass = UIPass::create(context, &renderer.render_context());
        if USE_3DVIEW {
            if let Some(blit_pass) = renderer.pass::<BlitPass>() {
                blit_pass.add_render_target(RenderTarget::Texture {
                    width,
                    height,
                    format: TextureFormat::Rgba8Unorm,
                });
            }
        } else {
            let mut ui_pass = ui_pass.render_pass().get_mut();
            ui_pass.set_load_color_operation(LoadOperation::Load);
        }
        renderer.add_pass(ui_pass, is_enabled);
    }
    fn create_compute_finalize_pass(
        context: &ContextRc,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
        visibility_texture_id: &TextureId,
        radiance_texture_id: &TextureId,
        depth_texture_id: &TextureId,
    ) -> TextureId {
        let mut compute_finalize_pass =
            ComputeFinalizePass::create(context, &renderer.render_context());
        compute_finalize_pass.add_render_target_with_resolution(
            width,
            height,
            TextureFormat::Rgba8Unorm,
        );
        compute_finalize_pass
            .set_visibility_texture(visibility_texture_id)
            .set_radiance_texture(radiance_texture_id)
            .set_depth_texture(depth_texture_id);
        let render_target_id = *compute_finalize_pass
            .render_targets_id()
            .unwrap()
            .first()
            .unwrap();
        renderer.add_pass(compute_finalize_pass, true);
        render_target_id
    }
    fn create_blit_pass(context: &ContextRc, renderer: &mut Renderer, texture: &TextureId) {
        let mut blit_pass = BlitPass::create(context, &renderer.render_context());
        blit_pass.set_source(texture);
        renderer.add_pass(blit_pass, true);
    }
}
