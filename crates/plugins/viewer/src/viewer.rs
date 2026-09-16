use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    wavefront_bounce_stage, BlitPass, CommandsPass, ComputeDiffuseLightingPass, ComputeGBufferPass,
    ComputeIndirectBouncePass, ComputeIndirectSetupPass, ComputeInstancesPass, ComputeShadowAOPass,
    ComputeSpecularLightingPass, ComputeWavefrontAoResolvePass, ComputeWavefrontCompositePass,
    ComputeWavefrontDenoisePass, ComputeWavefrontDirectCandidatesPass,
    ComputeWavefrontDispatchPass, ComputeWavefrontNextBouncePass, ComputeWavefrontPrimaryPass,
    ComputeWavefrontShadePass, ComputeWavefrontSurfacePass, ComputeWavefrontTransparencyPass,
    ComputeWavefrontTransparencyResolvePass, ComputeWavefrontTraversalPass,
    ComputeWavefrontUpscalePass, ComputeWavefrontVisibilityPass, CullingPass, DebugPass,
    DepthFirstPass, DepthPyramidPass, FinalizePass, VisibilityBufferPass,
    WavefrontBounceBindingIds, WavefrontQueueBindingIds, WireframePass,
    WAVEFRONT_HIT_DISPATCH_ARGS_ID, WAVEFRONT_HIT_HEADER_ID, WAVEFRONT_HIT_QUEUE_ID,
    WAVEFRONT_NEXT2_DISPATCH_ARGS_ID, WAVEFRONT_NEXT2_HEADER_ID, WAVEFRONT_NEXT2_QUEUE_ID,
    WAVEFRONT_NEXT3_DISPATCH_ARGS_ID, WAVEFRONT_NEXT3_HEADER_ID, WAVEFRONT_NEXT3_QUEUE_ID,
    WAVEFRONT_NEXT_DISPATCH_ARGS_ID, WAVEFRONT_NEXT_HEADER_ID, WAVEFRONT_NEXT_QUEUE_ID,
    WAVEFRONT_PRIMARY_QUEUE_ID, WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_render::{
    platform::has_wireframe_support, rendering_system::RenderingSystem,
    update_system::UpdateSystem, GPULight, GPUMaterial, GPUTexture, Pass, RenderContext,
    RenderContextRc, RenderPass, Renderer, RendererRw, TextureFormat, TextureUsage, DEFAULT_HEIGHT,
    DEFAULT_WIDTH,
};
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::{read_from_file, SerializationType};
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_UI_PASS: bool = true;

const MAX_NUM_LIGHTS: usize = 512;
const MAX_NUM_TEXTURES: usize = 2048;
const MAX_NUM_MATERIALS: usize = 256;

pub const PRODUCTION_PASS_ORDER: &[&str] = &[
    "DepthFirstPass",
    "DepthPyramidPass",
    "ComputeInstancesPass",
    "CullingPass",
    "CommandsPass",
    "VisibilityBufferPass",
    "ComputeWavefrontSurfacePass",
    "ComputeWavefrontTransparencyPass",
    "ComputeWavefrontAoResolvePass",
    "ComputeWavefrontDirectCandidatesPass",
    "ComputeWavefrontVisibilityPass",
    "ComputeWavefrontPrimaryPass",
    "ComputeWavefrontDispatchPass",
    "ComputeWavefrontTraversalPass",
    "ComputeWavefrontHitDispatchPass",
    "ComputeWavefrontShadePass",
    "ComputeWavefrontBounce1",
    "ComputeWavefrontBounce1Dispatch",
    "ComputeWavefrontBounce1Traversal",
    "ComputeWavefrontBounce1HitDispatch",
    "ComputeWavefrontBounce1Shade",
    "ComputeWavefrontBounce2",
    "ComputeWavefrontBounce2Dispatch",
    "ComputeWavefrontBounce2Traversal",
    "ComputeWavefrontBounce2HitDispatch",
    "ComputeWavefrontBounce2Shade",
    "ComputeWavefrontBounce3",
    "ComputeWavefrontBounce3Dispatch",
    "ComputeWavefrontBounce3Traversal",
    "ComputeWavefrontBounce3HitDispatch",
    "ComputeWavefrontBounce3Shade",
    "ComputeWavefrontBounce4",
    "ComputeWavefrontBounce4Dispatch",
    "ComputeWavefrontBounce4Traversal",
    "ComputeWavefrontBounce4HitDispatch",
    "ComputeWavefrontBounce4Shade",
    "ComputeWavefrontBounce5",
    "ComputeWavefrontBounce5Dispatch",
    "ComputeWavefrontBounce5Traversal",
    "ComputeWavefrontBounce5HitDispatch",
    "ComputeWavefrontBounce5Shade",
    "ComputeWavefrontBounce6",
    "ComputeWavefrontBounce6Dispatch",
    "ComputeWavefrontBounce6Traversal",
    "ComputeWavefrontBounce6HitDispatch",
    "ComputeWavefrontBounce6Shade",
    "ComputeWavefrontBounce7",
    "ComputeWavefrontBounce7Dispatch",
    "ComputeWavefrontBounce7Traversal",
    "ComputeWavefrontBounce7HitDispatch",
    "ComputeWavefrontBounce7Shade",
    "ComputeWavefrontBounce8",
    "ComputeWavefrontBounce8Dispatch",
    "ComputeWavefrontBounce8Traversal",
    "ComputeWavefrontBounce8HitDispatch",
    "ComputeWavefrontBounce8Shade",
    "ComputeWavefrontCompositePass",
    "ComputeWavefrontTransparencyResolvePass",
    "ComputeWavefrontDenoiseTemporalPass",
    "ComputeWavefrontDenoiseBilateralPass",
    "ComputeWavefrontUpscalePass",
    "FinalizePass",
    "BlitPass",
];

pub fn active_registered_pass_names(render_context: &RenderContext) -> Vec<String> {
    render_context
        .passes()
        .iter()
        .filter(|(_, enabled)| *enabled)
        .map(|(pass, _)| pass.name().to_owned())
        .collect()
}

enum RenderTargetType {
    Visibility = 0,
    Depth = 1,
    HiZ = 2,
    Frame0 = 3,
    Frame1 = 4,
    Normal = 5,
    Motion = 6,
    Albedo = 7,
    Variance = 8,
    Denoised = 9,
    Upscaled = 10,
}

pub struct Viewer {
    _renderer: RendererRw,
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
        let _renderer = Renderer::new(window.handle(), context, move |render_context| {
            Self::create_data_buffers(render_context, DEFAULT_WIDTH, DEFAULT_HEIGHT);
            Self::create_systems(&context_rc, render_context);
            Self::create_render_targets(render_context, DEFAULT_WIDTH, DEFAULT_HEIGHT);
            Self::create_render_passes(&context_rc, render_context);
        });

        let window_system = WindowSystem::new(window, context);
        context.add_system(inox_core::Phases::PlatformUpdate, window_system, None);

        Viewer { _renderer }
    }

    fn name(&self) -> &str {
        "inox_viewer"
    }

    fn prepare(&mut self, _context: &ContextRc) {}

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
            SerializationType::Json,
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

#[allow(dead_code)]
impl Viewer {
    fn create_systems(context: &ContextRc, render_context: &RenderContextRc) {
        let render_update_system = UpdateSystem::new(render_context, context);
        let rendering_draw_system = RenderingSystem::new(render_context, context);
        let mut ui_system = if ADD_UI_PASS {
            Some(UISystem::new(context))
        } else {
            None
        };

        let viewer_system = ViewerSystem::new(context, render_context, false);
        let object_system = ObjectSystem::new(context);
        let script_system = ScriptSystem::new(context);

        context.add_system(
            inox_core::Phases::Render,
            render_update_system,
            Some(&[RenderingSystem::system_id()]),
        );
        context.add_system(
            inox_core::Phases::EndFrame,
            rendering_draw_system,
            Some(&[UpdateSystem::system_id()]),
        );

        context.add_system(
            inox_core::Phases::Update,
            object_system,
            Some(&[RenderingSystem::system_id()]),
        );
        context.add_system(
            inox_core::Phases::Update,
            script_system,
            Some(&[RenderingSystem::system_id()]),
        );

        if let Some(ui_system) = ui_system.take() {
            context.add_system(
                inox_core::Phases::Update,
                ui_system,
                Some(&[RenderingSystem::system_id()]),
            );
        }
        context.add_system(
            inox_core::Phases::Update,
            viewer_system,
            Some(&[RenderingSystem::system_id()]),
        );
    }
    fn create_render_targets(render_context: &RenderContextRc, width: u32, height: u32) {
        let _half_dims = (width / 2, height / 2);
        let single_sample = 1;
        let usage = TextureUsage::TextureBinding
            | TextureUsage::CopySrc
            | TextureUsage::CopyDst
            | TextureUsage::RenderTarget;

        //Visibility,
        let _visibility = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::R32Uint,
            usage,
        );
        debug_assert!(_visibility == RenderTargetType::Visibility as usize);
        //Depth,
        let _depth = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Depth32Float,
            usage,
        );
        debug_assert!(_depth == RenderTargetType::Depth as usize);
        //HiZ,
        let _hzb = render_context.create_render_target(
            (width, height, single_sample, 1, 11),
            TextureFormat::R32Float,
            TextureUsage::TextureBinding
                | TextureUsage::CopySrc
                | TextureUsage::CopyDst
                | TextureUsage::StorageBinding,
        );
        debug_assert!(_hzb == RenderTargetType::HiZ as usize);
        //Frame0,
        let _frame0 = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba32Float,
            usage,
        );
        debug_assert!(_frame0 == RenderTargetType::Frame0 as usize);
        //Frame1,
        let _frame1 = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba32Float,
            usage,
        );
        debug_assert!(_frame1 == RenderTargetType::Frame1 as usize);
        // Auxiliary history targets used by temporal denoising and reconstruction.
        let _normal = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_normal == RenderTargetType::Normal as usize);
        let _motion = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rg16Float,
            usage,
        );
        debug_assert!(_motion == RenderTargetType::Motion as usize);
        let _albedo = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba8Unorm,
            usage,
        );
        debug_assert!(_albedo == RenderTargetType::Albedo as usize);
        let _variance = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rg16Float,
            usage,
        );
        debug_assert!(_variance == RenderTargetType::Variance as usize);
        let _denoised = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_denoised == RenderTargetType::Denoised as usize);
        let _upscaled = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_upscaled == RenderTargetType::Upscaled as usize);
    }
    fn create_data_buffers(render_context: &RenderContextRc, _width: u32, _height: u32) {
        render_context
            .global_buffers()
            .buffer::<GPUTexture>()
            .write()
            .unwrap()
            .prealloc::<MAX_NUM_TEXTURES>();
        render_context
            .global_buffers()
            .buffer::<GPULight>()
            .write()
            .unwrap()
            .prealloc::<MAX_NUM_LIGHTS>();
        render_context
            .global_buffers()
            .buffer::<GPUMaterial>()
            .write()
            .unwrap()
            .prealloc::<MAX_NUM_MATERIALS>();
    }
    fn create_render_passes(context: &ContextRc, render_context: &RenderContextRc) {
        Self::create_depth_pyramid_pass(context, render_context);
        Self::create_instances_pass(context, render_context);
        Self::create_culling_pass(context, render_context);

        Self::create_visibility_pass(context, render_context);

        // Wavefront pathtracer passes
        Self::create_gbuffer_pass(context, render_context);
        Self::create_wavefront_surface_pass(context, render_context);
        Self::create_wavefront_transparency_pass(context, render_context);
        Self::create_shadow_ao_pass(context, render_context);
        Self::create_wavefront_ao_resolve_pass(context, render_context);
        Self::create_wavefront_direct_candidates_pass(context, render_context);
        Self::create_wavefront_visibility_pass(context, render_context);
        Self::create_diffuse_lighting_pass(context, render_context);
        Self::create_specular_lighting_pass(context, render_context);
        Self::create_indirect_setup_pass(context, render_context);
        Self::create_indirect_bounce_pass(context, render_context);

        // Queue-driven wavefront stages. These consume only active rays and
        // keep scene data in the packed portable binding layout.
        Self::create_wavefront_primary_pass(context, render_context);
        Self::create_wavefront_dispatch_pass(context, render_context);
        Self::create_wavefront_traversal_pass(context, render_context);
        Self::create_wavefront_hit_dispatch_pass(context, render_context);
        Self::create_wavefront_shade_pass(context, render_context);
        Self::create_wavefront_bounce_chain(context, render_context);
        Self::create_wavefront_composite_pass(context, render_context);
        Self::create_wavefront_transparency_resolve_pass(context, render_context);
        Self::create_wavefront_denoiser_passes(context, render_context);
        Self::create_wavefront_upscale_pass(context, render_context);

        Self::create_finalize_pass(context, render_context);
        Self::create_blit_pass(context, render_context);

        let registered_passes = active_registered_pass_names(render_context);
        assert_eq!(
            registered_passes,
            PRODUCTION_PASS_ORDER
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            "active production pass registration diverged from PRODUCTION_PASS_ORDER"
        );

        //Self::create_debug_pass(context, render_context);
        Self::create_wireframe_pass(context, render_context, has_wireframe_support());
        Self::create_ui_pass(context, render_context, ADD_UI_PASS);
    }
    fn create_instances_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let instances_pass = ComputeInstancesPass::create(context, render_context);
        render_context.add_pass(instances_pass, true);
    }
    fn create_culling_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut culling_pass = CullingPass::create(context, render_context);
        culling_pass.set_hzb_texture(&render_context.render_target(RenderTargetType::HiZ as usize));
        render_context.add_pass(culling_pass, true);
        let commands_pass = CommandsPass::create(context, render_context);
        render_context.add_pass(commands_pass, true);
    }
    fn create_visibility_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_pass = VisibilityBufferPass::create(context, render_context);
        visibility_pass
            .add_render_target(&render_context.render_target(RenderTargetType::Visibility as usize))
            .add_depth_target(&render_context.render_target(RenderTargetType::Depth as usize));
        render_context.add_pass(visibility_pass, true);
    }
    fn create_gbuffer_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut gbuffer_pass = ComputeGBufferPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        gbuffer_pass
            .set_visibility_texture(
                visibility_texture.id(),
                visibility_texture.get().dimensions(),
            )
            .set_depth_texture(&render_context.render_target_id(RenderTargetType::Depth as usize));
        // Replaced by the packed-scene surface pass below.
        render_context.add_pass(gbuffer_pass, false);
    }
    fn create_shadow_ao_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut shadow_ao_pass = ComputeShadowAOPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        shadow_ao_pass.set_dimensions(visibility_texture.get().dimensions());
        // Replaced by the packed AO/direct queues.
        render_context.add_pass(shadow_ao_pass, false);
    }
    fn create_diffuse_lighting_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut diffuse_pass = ComputeDiffuseLightingPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        diffuse_pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(diffuse_pass, false);
    }
    fn create_specular_lighting_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut specular_pass = ComputeSpecularLightingPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        specular_pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(specular_pass, false);
    }
    fn create_indirect_setup_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut indirect_setup_pass = ComputeIndirectSetupPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        indirect_setup_pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(indirect_setup_pass, false);
    }
    fn create_indirect_bounce_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut indirect_bounce_pass = ComputeIndirectBouncePass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        indirect_bounce_pass.set_dimensions(visibility_texture.get().dimensions());
        // Replaced by compact packed-scene wavefront traversal.
        render_context.add_pass(indirect_bounce_pass, false);
    }
    fn create_wavefront_primary_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontPrimaryPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_dispatch_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let pass = ComputeWavefrontDispatchPass::create(context, render_context);
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_traversal_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontTraversalPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_hit_dispatch_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_HIT_HEADER_ID,
            WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            "ComputeWavefrontHitDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_transparency_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontTransparencyPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_surface_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontSurfacePass::create(context, render_context);
        let visibility = render_context.render_target(RenderTargetType::Visibility as usize);
        let depth = render_context.render_target(RenderTargetType::Depth as usize);
        pass.set_visibility_texture(visibility.id(), visibility.get().dimensions())
            .set_depth_texture(depth.id());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_ao_resolve_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontAoResolvePass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_direct_candidates_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let mut pass = ComputeWavefrontDirectCandidatesPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_visibility_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontVisibilityPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_shade_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let mut pass = ComputeWavefrontShadePass::create(context, render_context);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_bounce_chain(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let mut source_queue = WAVEFRONT_PRIMARY_QUEUE_ID;
        let mut source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
        for bounce in 1..=8u32 {
            let stage = wavefront_bounce_stage(bounce, source_queue, source_dispatch);
            let mut next = ComputeWavefrontNextBouncePass::create_for_queues(
                context,
                render_context,
                stage.bindings,
                stage.name.as_str(),
            );
            next.set_dimensions(visibility_texture.get().dimensions());
            render_context.add_pass(next, true);
            let dispatch = ComputeWavefrontDispatchPass::create_for_header(
                context,
                render_context,
                stage.bindings.destination_ray_header,
                stage.destination_dispatch_args,
                stage.dispatch_name.as_str(),
            );
            render_context.add_pass(dispatch, true);
            let mut traversal = ComputeWavefrontTraversalPass::create_for_queues(
                context,
                render_context,
                stage.traversal_bindings,
                stage.traversal_name.as_str(),
            );
            traversal.set_dimensions(visibility_texture.get().dimensions());
            render_context.add_pass(traversal, true);
            let hit_dispatch = ComputeWavefrontDispatchPass::create_for_header(
                context,
                render_context,
                stage.bindings.source_hit_header,
                WAVEFRONT_HIT_DISPATCH_ARGS_ID,
                stage.hit_dispatch_name.as_str(),
            );
            render_context.add_pass(hit_dispatch, true);
            let mut shade = ComputeWavefrontShadePass::create_for_queues(
                context,
                render_context,
                stage.shade_bindings,
                stage.shade_name.as_str(),
            );
            shade.set_dimensions(visibility_texture.get().dimensions());
            render_context.add_pass(shade, true);
            source_queue = stage.bindings.destination_ray_queue;
            source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
        }
    }
    fn create_wavefront_next_bounce_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontNextBouncePass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_next_dispatch_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_NEXT_HEADER_ID,
            WAVEFRONT_NEXT_DISPATCH_ARGS_ID,
            "ComputeWavefrontNextDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_next_traversal_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontTraversalPass::create_for_queues(
            context,
            render_context,
            WavefrontQueueBindingIds {
                ray_queue: WAVEFRONT_NEXT_QUEUE_ID,
                ray_header: WAVEFRONT_NEXT_HEADER_ID,
                hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                hit_header: WAVEFRONT_HIT_HEADER_ID,
                dispatch_args: WAVEFRONT_NEXT_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontNextTraversalPass",
        );
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_next_hit_dispatch_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_HIT_HEADER_ID,
            WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            "ComputeWavefrontNextHitDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_next_shade_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let mut pass = ComputeWavefrontShadePass::create_for_queues(
            context,
            render_context,
            WavefrontQueueBindingIds {
                ray_queue: WAVEFRONT_NEXT_QUEUE_ID,
                ray_header: WAVEFRONT_NEXT_HEADER_ID,
                hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                hit_header: WAVEFRONT_HIT_HEADER_ID,
                dispatch_args: WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontNextShadePass",
        );
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_second_bounce_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontNextBouncePass::create_for_queues(
            context,
            render_context,
            WavefrontBounceBindingIds {
                source_ray_queue: WAVEFRONT_NEXT_QUEUE_ID,
                source_hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                source_hit_header: WAVEFRONT_HIT_HEADER_ID,
                destination_ray_queue: WAVEFRONT_NEXT2_QUEUE_ID,
                destination_ray_header: WAVEFRONT_NEXT2_HEADER_ID,
                dispatch_args: WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontSecondBouncePass",
        );
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_second_dispatch_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_NEXT2_HEADER_ID,
            WAVEFRONT_NEXT2_DISPATCH_ARGS_ID,
            "ComputeWavefrontSecondDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_second_traversal_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let mut pass = ComputeWavefrontTraversalPass::create_for_queues(
            context,
            render_context,
            WavefrontQueueBindingIds {
                ray_queue: WAVEFRONT_NEXT2_QUEUE_ID,
                ray_header: WAVEFRONT_NEXT2_HEADER_ID,
                hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                hit_header: WAVEFRONT_HIT_HEADER_ID,
                dispatch_args: WAVEFRONT_NEXT2_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontSecondTraversalPass",
        );
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_second_hit_dispatch_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_HIT_HEADER_ID,
            WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            "ComputeWavefrontSecondHitDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_second_shade_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let mut pass = ComputeWavefrontShadePass::create_for_queues(
            context,
            render_context,
            WavefrontQueueBindingIds {
                ray_queue: WAVEFRONT_NEXT2_QUEUE_ID,
                ray_header: WAVEFRONT_NEXT2_HEADER_ID,
                hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                hit_header: WAVEFRONT_HIT_HEADER_ID,
                dispatch_args: WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontSecondShadePass",
        );
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_third_bounce_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontNextBouncePass::create_for_queues(
            context,
            render_context,
            WavefrontBounceBindingIds {
                source_ray_queue: WAVEFRONT_NEXT2_QUEUE_ID,
                source_hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                source_hit_header: WAVEFRONT_HIT_HEADER_ID,
                destination_ray_queue: WAVEFRONT_NEXT3_QUEUE_ID,
                destination_ray_header: WAVEFRONT_NEXT3_HEADER_ID,
                dispatch_args: WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontThirdBouncePass",
        );
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_third_dispatch_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_NEXT3_HEADER_ID,
            WAVEFRONT_NEXT3_DISPATCH_ARGS_ID,
            "ComputeWavefrontThirdDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_third_traversal_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let mut pass = ComputeWavefrontTraversalPass::create_for_queues(
            context,
            render_context,
            WavefrontQueueBindingIds {
                ray_queue: WAVEFRONT_NEXT3_QUEUE_ID,
                ray_header: WAVEFRONT_NEXT3_HEADER_ID,
                hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                hit_header: WAVEFRONT_HIT_HEADER_ID,
                dispatch_args: WAVEFRONT_NEXT3_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontThirdTraversalPass",
        );
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_third_hit_dispatch_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let pass = ComputeWavefrontDispatchPass::create_for_header(
            context,
            render_context,
            WAVEFRONT_HIT_HEADER_ID,
            WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            "ComputeWavefrontThirdHitDispatchPass",
        );
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_third_shade_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let mut pass = ComputeWavefrontShadePass::create_for_queues(
            context,
            render_context,
            WavefrontQueueBindingIds {
                ray_queue: WAVEFRONT_NEXT3_QUEUE_ID,
                ray_header: WAVEFRONT_NEXT3_HEADER_ID,
                hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                hit_header: WAVEFRONT_HIT_HEADER_ID,
                dispatch_args: WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            },
            "ComputeWavefrontThirdShadePass",
        );
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_composite_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontCompositePass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_transparency_resolve_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let mut pass = ComputeWavefrontTransparencyResolvePass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_wavefront_denoiser_passes(context: &ContextRc, render_context: &RenderContextRc) {
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let mut temporal = ComputeWavefrontDenoisePass::create(context, render_context);
        temporal.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(temporal, true);
        let mut bilateral = ComputeWavefrontDenoisePass::create_bilateral(context, render_context);
        bilateral.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(bilateral, true);
    }
    fn create_wavefront_upscale_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeWavefrontUpscalePass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(pass, true);
    }
    fn create_finalize_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut finalize_pass = FinalizePass::create(context, render_context);
        finalize_pass.set_frame_textures([
            &render_context.render_target(RenderTargetType::Frame0 as usize),
            &render_context.render_target(RenderTargetType::Frame1 as usize),
        ]);
        render_context.add_pass(finalize_pass, true);
    }
    fn create_blit_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut blit_pass = BlitPass::create(context, render_context);
        blit_pass.set_sources([
            &render_context.render_target_id(RenderTargetType::Frame0 as usize),
            &render_context.render_target_id(RenderTargetType::Frame1 as usize),
        ]);
        render_context.add_pass(blit_pass, true);
    }
    fn create_depth_pyramid_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut depth_first_pass = DepthFirstPass::create(context, render_context);
        depth_first_pass
            .set_depth_texture(render_context.render_target(RenderTargetType::Depth as usize))
            .set_hzb_texture(render_context.render_target(RenderTargetType::HiZ as usize));
        render_context.add_pass(depth_first_pass, true);

        let mut depth_pyramid_pass = DepthPyramidPass::create(context, render_context);
        depth_pyramid_pass
            .set_depth_texture(render_context.render_target(RenderTargetType::Depth as usize))
            .set_hzb_texture(render_context.render_target(RenderTargetType::HiZ as usize));
        render_context.add_pass(depth_pyramid_pass, true);
    }
    #[allow(dead_code)]
    fn create_debug_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut debug_pass = DebugPass::create(context, render_context);
        debug_pass
            .set_visibility_texture(
                &render_context.render_target_id(RenderTargetType::Visibility as usize),
            )
            .set_depth_texture(&render_context.render_target_id(RenderTargetType::Depth as usize));
        render_context.add_pass(debug_pass, true);
    }
    fn create_wireframe_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
        is_enabled: bool,
    ) {
        if !is_enabled {
            return;
        }
        let wireframe_pass = WireframePass::create(context, render_context);
        render_context.add_pass(wireframe_pass, is_enabled);
    }
    fn create_ui_pass(context: &ContextRc, render_context: &RenderContextRc, is_enabled: bool) {
        if !is_enabled {
            return;
        }
        let ui_pass = UIPass::create(context, render_context);
        render_context.add_pass(ui_pass, is_enabled);
    }
}
