use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    platform::has_wireframe_support, rendering_system::RenderingSystem,
    update_system::UpdateSystem, BlitPass, ComputeFinalizePass, ComputePathTracingDirectPass,
    ComputePathTracingIndirectPass, ComputeRuntimeVerticesPass, CullingPass, DebugPass, Pass,
    RenderPass, Renderer, RendererRw, TextureFormat, TextureUsage, VisibilityBufferPass,
    WireframePass, DEFAULT_HEIGHT, DEFAULT_WIDTH, WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::read_from_file;
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_CULLING_PASS: bool = true;
const ADD_UI_PASS: bool = true;

enum RenderTargetType {
    Visibility = 0,
    Depth = 1,
    GBuffer = 2,
    Radiance = 3,
    DebugData = 4,
    Finalize = 5,
}

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
            Self::create_render_targets(renderer, DEFAULT_WIDTH, DEFAULT_HEIGHT);
            Self::create_render_passes(&context_rc, renderer);
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

        let viewer_system = ViewerSystem::new(context, &self.renderer, false);
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
    fn create_render_targets(renderer: &mut Renderer, width: u32, height: u32) {
        let half_dims = (width / 4, height / 4);
        let single_sample = 1;
        let multi_sample = 4;
        let usage = TextureUsage::TextureBinding
            | TextureUsage::CopySrc
            | TextureUsage::CopyDst
            | TextureUsage::RenderTarget;

        //Visibility = 0,
        let _visibility = renderer.add_render_target(
            half_dims.0,
            half_dims.1,
            TextureFormat::Rgba8Unorm,
            usage,
            single_sample,
        );
        debug_assert!(_visibility == RenderTargetType::Visibility as usize);
        //Depth = 1,
        let _depth = renderer.add_render_target(
            half_dims.0,
            half_dims.1,
            TextureFormat::Depth32Float,
            usage,
            single_sample,
        );
        debug_assert!(_depth == RenderTargetType::Depth as usize);
        //GBuffer = 2,
        let _gbuffer = renderer.add_render_target(
            half_dims.0,
            half_dims.1,
            TextureFormat::Rgba32Float,
            usage | TextureUsage::StorageBinding,
            single_sample,
        );
        debug_assert!(_gbuffer == RenderTargetType::GBuffer as usize);
        //Radiance = 3,
        let _radiance = renderer.add_render_target(
            half_dims.0,
            half_dims.1,
            TextureFormat::Rgba32Float,
            usage | TextureUsage::StorageBinding,
            single_sample,
        );
        debug_assert!(_radiance == RenderTargetType::Radiance as usize);
        //Debug = 4,
        let _debug_data = renderer.add_render_target(
            half_dims.0,
            half_dims.1,
            TextureFormat::R32Float,
            usage | TextureUsage::StorageBinding,
            single_sample,
        );
        debug_assert!(_debug_data == RenderTargetType::DebugData as usize);
        //Finalize = 5,
        let _finalize = renderer.add_render_target(
            width,
            height,
            TextureFormat::Rgba8Unorm,
            usage | TextureUsage::StorageBinding,
            multi_sample,
        );
        debug_assert!(_finalize == RenderTargetType::Finalize as usize);
    }
    fn create_render_passes(context: &ContextRc, renderer: &mut Renderer) {
        Self::create_compute_runtime_vertices_pass(context, renderer, true);
        Self::create_culling_pass(context, renderer, ADD_CULLING_PASS);

        Self::create_visibility_pass(context, renderer);
        Self::create_compute_pathtracing_direct_pass(context, renderer);
        Self::create_compute_pathtracing_indirect_pass(context, renderer);
        Self::create_compute_finalize_pass(context, renderer);
        Self::create_blit_pass(context, renderer);

        Self::create_debug_pass(context, renderer);
        Self::create_wireframe_pass(context, renderer, has_wireframe_support());
        Self::create_ui_pass(context, renderer, ADD_UI_PASS);
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
    fn create_visibility_pass(context: &ContextRc, renderer: &mut Renderer) {
        let visibility_pass = VisibilityBufferPass::create(context, &renderer.render_context());
        visibility_pass
            .add_render_target(renderer.render_target(RenderTargetType::Visibility as usize))
            .add_depth_target(renderer.render_target(RenderTargetType::Depth as usize));
        renderer.add_pass(visibility_pass, true);
    }
    fn create_compute_pathtracing_direct_pass(context: &ContextRc, renderer: &mut Renderer) {
        let mut compute_pathtracing_direct_pass =
            ComputePathTracingDirectPass::create(context, &renderer.render_context());
        let gbuffer_texture = renderer.render_target(RenderTargetType::GBuffer as usize);
        compute_pathtracing_direct_pass
            .set_gbuffer_texture(gbuffer_texture.id(), gbuffer_texture.get().dimensions())
            .set_visibility_texture(
                renderer.render_target_id(RenderTargetType::Visibility as usize),
            )
            .set_depth_texture(renderer.render_target_id(RenderTargetType::Depth as usize));
        renderer.add_pass(compute_pathtracing_direct_pass, true);
    }
    fn create_compute_pathtracing_indirect_pass(context: &ContextRc, renderer: &mut Renderer) {
        let mut compute_pathtracing_indirect_pass =
            ComputePathTracingIndirectPass::create(context, &renderer.render_context());
        let radiance_texture = renderer.render_target(RenderTargetType::Radiance as usize);
        compute_pathtracing_indirect_pass
            .set_radiance_texture(radiance_texture.id(), radiance_texture.get().dimensions())
            .set_visibility_texture(
                renderer.render_target_id(RenderTargetType::Visibility as usize),
            )
            .set_depth_texture(renderer.render_target_id(RenderTargetType::Depth as usize))
            .set_debug_data_texture(
                renderer.render_target_id(RenderTargetType::DebugData as usize),
            );
        renderer.add_pass(compute_pathtracing_indirect_pass, true);
    }
    fn create_compute_finalize_pass(context: &ContextRc, renderer: &mut Renderer) {
        let mut compute_finalize_pass =
            ComputeFinalizePass::create(context, &renderer.render_context());
        let finalize_texture = renderer.render_target(RenderTargetType::Finalize as usize);
        compute_finalize_pass
            .set_finalize_texture(finalize_texture.id(), finalize_texture.get().dimensions())
            .set_visibility_texture(
                renderer.render_target_id(RenderTargetType::Visibility as usize),
            )
            .set_gbuffer_texture(renderer.render_target_id(RenderTargetType::GBuffer as usize))
            .set_radiance_texture(renderer.render_target_id(RenderTargetType::Radiance as usize))
            .set_depth_texture(renderer.render_target_id(RenderTargetType::Depth as usize));
        renderer.add_pass(compute_finalize_pass, true);
    }
    fn create_blit_pass(context: &ContextRc, renderer: &mut Renderer) {
        let mut blit_pass = BlitPass::create(context, &renderer.render_context());
        blit_pass.set_source(renderer.render_target_id(RenderTargetType::Finalize as usize));
        renderer.add_pass(blit_pass, true);
    }
    fn create_debug_pass(context: &ContextRc, renderer: &mut Renderer) {
        let mut debug_pass = DebugPass::create(context, &renderer.render_context());
        debug_pass
            .set_finalize_texture(renderer.render_target_id(RenderTargetType::Finalize as usize))
            .set_visibility_texture(
                renderer.render_target_id(RenderTargetType::Visibility as usize),
            )
            .set_radiance_texture(renderer.render_target_id(RenderTargetType::Radiance as usize))
            .set_gbuffer_texture(renderer.render_target_id(RenderTargetType::GBuffer as usize))
            .set_depth_texture(renderer.render_target_id(RenderTargetType::Depth as usize))
            .set_debug_data_texture(
                renderer.render_target_id(RenderTargetType::DebugData as usize),
            );
        renderer.add_pass(debug_pass, true);
    }
    fn create_wireframe_pass(context: &ContextRc, renderer: &mut Renderer, is_enabled: bool) {
        if !is_enabled {
            return;
        }
        let wireframe_pass = WireframePass::create(context, &renderer.render_context());
        renderer.add_pass(wireframe_pass, is_enabled);
    }
    fn create_ui_pass(context: &ContextRc, renderer: &mut Renderer, is_enabled: bool) {
        if !is_enabled {
            return;
        }
        let ui_pass = UIPass::create(context, &renderer.render_context());
        /*
        if USE_3DVIEW {
            if let Some(blit_pass) = renderer.pass::<BlitPass>() {
                blit_pass.add_render_target(RenderTarget::Texture {
                    width,
                    height,
                    format: TextureFormat::Rgba8Unorm,
                });
            }
        }*/
        renderer.add_pass(ui_pass, is_enabled);
    }
}
