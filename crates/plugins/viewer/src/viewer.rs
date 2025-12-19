use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    BlitPass, CommandsPass, ComputeAOGenerationPass, ComputeAOShadingPass, ComputeAOTraversalPass,
    ComputeDirectLightingPass, ComputeInstancesPass, ComputeShadowGenerationPass,
    ComputeShadowShadingPass, ComputeShadowTraversalPass, CullingPass, DepthFirstPass,
    DepthPyramidPass, FinalizePass, IntersectionPackedData, RayBounceManagerPass, RayPackedData,
    VisibilityBufferPass, WireframePass, AO_INTERSECTIONS_ID, AO_RAYS_ID, BOUNCE_INTERSECTIONS_ID,
    BOUNCE_RAYS_ID, BOUNCE_RAYS_NEXT_ID, SHADOW_INTERSECTIONS_ID, SHADOW_RAYS_ID,
    WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_render::{
    platform::has_wireframe_support, rendering_system::RenderingSystem,
    update_system::UpdateSystem, GPULight, GPUMaterial, GPUTexture, Pass, RenderContextRc,
    RenderPass, Renderer, RendererRw, TextureFormat, TextureUsage, DEFAULT_HEIGHT, DEFAULT_WIDTH,
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

enum RenderTargetType {
    Visibility = 0,
    Depth = 1,
    HiZ = 2,
    Frame0 = 3,
    Frame1 = 4,
    Direct = 5,
    IndirectDiffuse = 6,
    IndirectSpecular = 7,
    Shadow = 8,
    AO = 9,
    Reflection = 10,
    Refraction = 11,
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
            | TextureUsage::RenderTarget
            | TextureUsage::StorageBinding; // Need StorageBinding for Compute Writes

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
            TextureFormat::Rgba8Unorm,
            usage,
        );
        debug_assert!(_frame0 == RenderTargetType::Frame0 as usize);
        //Frame1,
        let _frame1 = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba8Unorm,
            usage,
        );
        debug_assert!(_frame1 == RenderTargetType::Frame1 as usize);

        // Direct
        let _direct = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_direct == RenderTargetType::Direct as usize);

        // IndirectDiffuse
        let _indirect_diffuse = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_indirect_diffuse == RenderTargetType::IndirectDiffuse as usize);

        // IndirectSpecular
        let _indirect_specular = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_indirect_specular == RenderTargetType::IndirectSpecular as usize);

        // Shadow
        let _shadow = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::R16Float,
            usage,
        );
        debug_assert!(_shadow == RenderTargetType::Shadow as usize);

        // AO
        let _ao = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::R16Float,
            usage,
        );
        debug_assert!(_ao == RenderTargetType::AO as usize);

        // Reflection
        let _reflection = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_reflection == RenderTargetType::Reflection as usize);

        // Refraction
        let _refraction = render_context.create_render_target(
            (width, height, single_sample, 1, 1),
            TextureFormat::Rgba16Float,
            usage,
        );
        debug_assert!(_refraction == RenderTargetType::Refraction as usize);
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

        // Pre-allocate ray buffers for path tracing (one ray per pixel)
        let num_pixels = DEFAULT_WIDTH as usize * DEFAULT_HEIGHT as usize;

        render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(BOUNCE_RAYS_ID)
            .write()
            .unwrap()
            .resize(num_pixels, RayPackedData::default());

        render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(BOUNCE_RAYS_NEXT_ID)
            .write()
            .unwrap()
            .resize(num_pixels, RayPackedData::default());

        render_context
            .global_buffers()
            .vector_with_id::<IntersectionPackedData>(BOUNCE_INTERSECTIONS_ID)
            .write()
            .unwrap()
            .resize(num_pixels, IntersectionPackedData::default());

        // Shadow rays (shared across shadow generation/traversal/shading)
        render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(SHADOW_RAYS_ID)
            .write()
            .unwrap()
            .resize(num_pixels, RayPackedData::default());

        render_context
            .global_buffers()
            .vector_with_id::<IntersectionPackedData>(SHADOW_INTERSECTIONS_ID)
            .write()
            .unwrap()
            .resize(num_pixels, IntersectionPackedData::default());

        // AO rays (shared across AO generation/traversal/shading)
        render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(AO_RAYS_ID)
            .write()
            .unwrap()
            .resize(num_pixels, RayPackedData::default());

        render_context
            .global_buffers()
            .vector_with_id::<IntersectionPackedData>(AO_INTERSECTIONS_ID)
            .write()
            .unwrap()
            .resize(num_pixels, IntersectionPackedData::default());
    }

    fn create_render_passes(context: &ContextRc, render_context: &RenderContextRc) {
        Self::create_depth_pyramid_pass(context, render_context);
        Self::create_instances_pass(context, render_context);
        Self::create_culling_pass(context, render_context);

        Self::create_visibility_pass(context, render_context);

        // Wavefront Path Tracer Pipeline with Runtime-Configurable Multi-Bounce
        Self::create_compute_direct_lighting_pass(context, render_context);

        // Create bounce manager that will dispatch traversal + shading N times per frame
        // based on runtime num_bounces value from ConstantData
        Self::create_ray_bounce_manager_pass(context, render_context);

        // Shadow Pipeline (always active, independent of num_bounces)
        Self::create_compute_shadow_generation_pass(context, render_context);
        Self::create_compute_shadow_traversal_pass(context, render_context);
        Self::create_compute_shadow_shading_pass(context, render_context);

        // AO Pipeline (always active, independent of num_bounces)
        Self::create_compute_ao_generation_pass(context, render_context);
        Self::create_compute_ao_traversal_pass(context, render_context);
        Self::create_compute_ao_shading_pass(context, render_context);

        Self::create_finalize_pass(context, render_context);
        Self::create_blit_pass(context, render_context);

        //Self::create_debug_pass(context, render_context);
        Self::create_wireframe_pass(context, render_context, has_wireframe_support());
        Self::create_ui_pass(context, render_context, ADD_UI_PASS);
    }

    // ...

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

    fn create_compute_direct_lighting_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeDirectLightingPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        let _depth_texture = render_context.render_target(RenderTargetType::Depth as usize);
        let _direct_texture = render_context.render_target(RenderTargetType::Direct as usize);
        pass.set_visibility_texture(
            *visibility_texture.id(),
            visibility_texture.get().dimensions(),
        );
        pass.set_depth_texture(&render_context.render_target_id(RenderTargetType::Depth as usize));
        pass.set_direct_lighting_texture(
            &render_context.render_target_id(RenderTargetType::Direct as usize),
        );
        render_context.add_pass(pass, true);
    }

    fn create_compute_shadow_generation_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
    ) {
        let mut pass = ComputeShadowGenerationPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_visibility_texture(
            *visibility_texture.id(),
            visibility_texture.get().dimensions(),
        )
        .set_depth_texture(&render_context.render_target_id(RenderTargetType::Depth as usize));
        render_context.add_pass(pass, true);
    }

    fn create_compute_shadow_traversal_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let pass = ComputeShadowTraversalPass::create(context, render_context);
        render_context.add_pass(pass, true);
    }

    fn create_compute_shadow_shading_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeShadowShadingPass::create(context, render_context);
        let shadow_texture = render_context.render_target(RenderTargetType::Shadow as usize);
        pass.set_shadow_texture(
            &render_context.render_target_id(RenderTargetType::Shadow as usize),
            shadow_texture.get().dimensions(),
        );
        render_context.add_pass(pass, true);
    }

    fn create_compute_ao_generation_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeAOGenerationPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        pass.set_visibility_texture(
            *visibility_texture.id(),
            visibility_texture.get().dimensions(),
        )
        .set_depth_texture(&render_context.render_target_id(RenderTargetType::Depth as usize));
        render_context.add_pass(pass, true);
    }

    fn create_compute_ao_traversal_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let pass = ComputeAOTraversalPass::create(context, render_context);
        render_context.add_pass(pass, true);
    }

    fn create_compute_ao_shading_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut pass = ComputeAOShadingPass::create(context, render_context);
        let ao_texture = render_context.render_target(RenderTargetType::AO as usize);
        pass.set_ao_texture(
            &render_context.render_target_id(RenderTargetType::AO as usize),
            ao_texture.get().dimensions(),
        );
        render_context.add_pass(pass, true);
    }

    fn create_ray_bounce_manager_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut manager = RayBounceManagerPass::create(context, render_context);
        manager
            .set_indirect_diffuse_texture(
                &render_context.render_target_id(RenderTargetType::IndirectDiffuse as usize),
            )
            .set_indirect_specular_texture(
                &render_context.render_target_id(RenderTargetType::IndirectSpecular as usize),
            );
        render_context.add_pass(manager, true);
    }

    // Legacy functions kept below...
    fn create_finalize_pass(context: &ContextRc, render_context: &RenderContextRc) {
        let mut finalize_pass = FinalizePass::create(context, render_context);
        finalize_pass
            .set_frame_textures([
                &render_context.render_target(RenderTargetType::Frame0 as usize),
                &render_context.render_target(RenderTargetType::Frame1 as usize),
            ])
            .set_direct_texture(&render_context.render_target(RenderTargetType::Direct as usize))
            .set_indirect_diffuse_texture(
                &render_context.render_target(RenderTargetType::IndirectDiffuse as usize),
            )
            .set_indirect_specular_texture(
                &render_context.render_target(RenderTargetType::IndirectSpecular as usize),
            )
            .set_shadow_texture(&render_context.render_target(RenderTargetType::Shadow as usize))
            .set_ao_texture(&render_context.render_target(RenderTargetType::AO as usize));
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
