use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    report_wavefront_queue_diagnostics, wavefront_auxiliary_bindings, wavefront_bounce_stages,
    BlitPass, CommandsPass, ComputeDiffuseLightingPass, ComputeGBufferPass,
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
    WAVEFRONT_PRIMARY_HEADER_ID, WAVEFRONT_PRIMARY_QUEUE_ID, WAVEFRONT_RESERVOIR_ID,
    WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_render::{
    rendering_system::RenderingSystem, update_system::UpdateSystem, GPULight, GPUMaterial,
    GPUTexture, Pass, RenderContext, RenderContextRc, RenderPass, Renderer, RendererRw,
    TextureFormat, TextureUsage, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::{read_from_file, SerializationType};
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};
use inox_uid::Uid;

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_UI_PASS: bool = true;

const MAX_NUM_LIGHTS: usize = 512;
const MAX_NUM_TEXTURES: usize = 2048;
const MAX_NUM_MATERIALS: usize = 256;

pub type ProductionPassFactory = fn(&ContextRc, &RenderContextRc);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionPassMetadata {
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionGraphDependency {
    pub pass: String,
    pub predecessor: String,
    pub resource: Uid,
}

pub struct ProductionPassRegistration {
    pub passes: Vec<ProductionPassMetadata>,
    pub dependencies: Vec<ProductionGraphDependency>,
    factory: ProductionPassFactory,
}

impl ProductionPassRegistration {
    fn new(
        names: impl IntoIterator<Item = String>,
        enabled: bool,
        factory: ProductionPassFactory,
        dependencies: Vec<ProductionGraphDependency>,
    ) -> Self {
        Self {
            passes: names
                .into_iter()
                .map(|name| ProductionPassMetadata { name, enabled })
                .collect(),
            dependencies,
            factory,
        }
    }
}

fn dependency(pass: &str, predecessor: &str, resource: Uid) -> ProductionGraphDependency {
    ProductionGraphDependency {
        pass: pass.to_owned(),
        predecessor: predecessor.to_owned(),
        resource,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionConstructionRecord {
    pub pass: String,
    pub enabled: bool,
    pub resources: Vec<Uid>,
}

/// Construction inputs shared by the production factories and graph tests.
/// The bounce factory consumes `wavefront_bounce_stages()` directly; auxiliary
/// factories consume the same `wavefront_auxiliary_bindings()` values below.
pub fn production_construction_records() -> Vec<ProductionConstructionRecord> {
    let auxiliary = wavefront_auxiliary_bindings();
    let mut records = vec![
        ProductionConstructionRecord {
            pass: "ComputeWavefrontTransparencyPass".to_owned(),
            enabled: true,
            resources: vec![auxiliary.transparent_queue, auxiliary.transparent_header],
        },
        ProductionConstructionRecord {
            pass: "ComputeWavefrontTransparencyResolvePass".to_owned(),
            enabled: true,
            resources: vec![auxiliary.transparent_queue, auxiliary.transparent_header],
        },
        ProductionConstructionRecord {
            pass: "ComputeWavefrontDirectCandidatesPass".to_owned(),
            enabled: true,
            resources: vec![auxiliary.ao_state],
        },
        ProductionConstructionRecord {
            pass: "ComputeWavefrontAoResolvePass".to_owned(),
            enabled: true,
            resources: vec![auxiliary.ao_state],
        },
    ];
    for stage in wavefront_bounce_stages() {
        records.extend([
            ProductionConstructionRecord {
                pass: stage.name,
                enabled: true,
                resources: vec![
                    stage.bindings.source_ray_queue,
                    stage.bindings.source_hit_header,
                    stage.bindings.destination_ray_queue,
                    stage.bindings.destination_ray_header,
                    stage.bindings.dispatch_args,
                ],
            },
            ProductionConstructionRecord {
                pass: stage.dispatch_name,
                enabled: true,
                resources: vec![
                    stage.bindings.destination_ray_header,
                    stage.destination_dispatch_args,
                ],
            },
            ProductionConstructionRecord {
                pass: stage.traversal_name,
                enabled: true,
                resources: vec![
                    stage.traversal_bindings.ray_queue,
                    stage.traversal_bindings.ray_header,
                    stage.traversal_bindings.dispatch_args,
                ],
            },
            ProductionConstructionRecord {
                pass: stage.hit_dispatch_name,
                enabled: true,
                resources: vec![
                    stage.bindings.source_hit_header,
                    WAVEFRONT_HIT_DISPATCH_ARGS_ID,
                ],
            },
            ProductionConstructionRecord {
                pass: stage.shade_name,
                enabled: true,
                resources: vec![
                    stage.shade_bindings.ray_queue,
                    stage.shade_bindings.ray_header,
                    stage.shade_bindings.hit_queue,
                    stage.shade_bindings.hit_header,
                    stage.shade_bindings.dispatch_args,
                ],
            },
        ]);
    }
    records.extend(
        [
            "ComputeGBufferPass",
            "ComputeShadowAOPass",
            "ComputeDiffuseLightingPass",
            "ComputeSpecularLightingPass",
            "ComputeIndirectSetupPass",
            "ComputeIndirectBouncePass",
        ]
        .into_iter()
        .map(|pass| ProductionConstructionRecord {
            pass: pass.to_owned(),
            enabled: false,
            resources: Vec::new(),
        }),
    );
    records
}

fn validate_construction_record(pass: &str, enabled: bool, resources: &[Uid]) {
    let record = production_construction_records()
        .into_iter()
        .find(|record| record.pass == pass)
        .unwrap_or_else(|| panic!("missing construction record for {pass}"));
    assert_eq!(
        record.enabled, enabled,
        "construction flag diverged for {pass}"
    );
    assert_eq!(
        record.resources, resources,
        "construction inputs diverged for {pass}"
    );
}

/// The single production registration table. Its factories construct the
/// actual passes, while the same rows expose observed names, enabled flags,
/// and graph handoffs to integration tests.
pub fn production_pass_registrations() -> Vec<ProductionPassRegistration> {
    let mut registrations = Vec::new();
    let add =
        |registrations: &mut Vec<ProductionPassRegistration>, names: &[&str], enabled, factory| {
            registrations.push(ProductionPassRegistration::new(
                names.iter().map(|name| (*name).to_owned()),
                enabled,
                factory,
                Vec::new(),
            ));
        };

    add(
        &mut registrations,
        &["DepthFirstPass", "DepthPyramidPass"],
        true,
        Viewer::create_depth_pyramid_pass,
    );
    add(
        &mut registrations,
        &["ComputeInstancesPass"],
        true,
        Viewer::create_instances_pass,
    );
    add(
        &mut registrations,
        &["CullingPass", "CommandsPass"],
        true,
        Viewer::create_culling_pass,
    );
    add(
        &mut registrations,
        &["VisibilityBufferPass"],
        true,
        Viewer::create_visibility_pass,
    );
    add(
        &mut registrations,
        &["ComputeGBufferPass"],
        false,
        Viewer::create_gbuffer_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontSurfacePass"],
        true,
        Viewer::create_wavefront_surface_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontTransparencyPass"],
        true,
        Viewer::create_wavefront_transparency_pass,
    );
    add(
        &mut registrations,
        &["ComputeShadowAOPass"],
        false,
        Viewer::create_shadow_ao_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontAoResolvePass"],
        true,
        Viewer::create_wavefront_ao_resolve_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontDirectCandidatesPass"],
        true,
        Viewer::create_wavefront_direct_candidates_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontVisibilityPass"],
        true,
        Viewer::create_wavefront_visibility_pass,
    );
    add(
        &mut registrations,
        &["ComputeDiffuseLightingPass"],
        false,
        Viewer::create_diffuse_lighting_pass,
    );
    add(
        &mut registrations,
        &["ComputeSpecularLightingPass"],
        false,
        Viewer::create_specular_lighting_pass,
    );
    add(
        &mut registrations,
        &["ComputeIndirectSetupPass"],
        false,
        Viewer::create_indirect_setup_pass,
    );
    add(
        &mut registrations,
        &["ComputeIndirectBouncePass"],
        false,
        Viewer::create_indirect_bounce_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontPrimaryPass"],
        true,
        Viewer::create_wavefront_primary_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontDispatchPass"],
        true,
        Viewer::create_wavefront_dispatch_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontTraversalPass"],
        true,
        Viewer::create_wavefront_traversal_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontHitDispatchPass"],
        true,
        Viewer::create_wavefront_hit_dispatch_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontShadePass"],
        true,
        Viewer::create_wavefront_shade_pass,
    );

    let mut previous_shade = "ComputeWavefrontShadePass".to_owned();
    let mut bounce_names = Vec::new();
    let mut bounce_dependencies = Vec::new();
    for stage in wavefront_bounce_stages() {
        bounce_names.extend([
            stage.name.clone(),
            stage.dispatch_name.clone(),
            stage.traversal_name.clone(),
            stage.hit_dispatch_name.clone(),
            stage.shade_name.clone(),
        ]);
        bounce_dependencies.extend([
            dependency(
                &stage.name,
                &previous_shade,
                stage.bindings.source_ray_queue,
            ),
            dependency(
                &stage.dispatch_name,
                &stage.name,
                stage.bindings.destination_ray_header,
            ),
            dependency(
                &stage.traversal_name,
                &stage.dispatch_name,
                stage.bindings.destination_ray_queue,
            ),
            dependency(
                &stage.hit_dispatch_name,
                &stage.traversal_name,
                stage.bindings.source_hit_header,
            ),
            dependency(
                &stage.shade_name,
                &stage.hit_dispatch_name,
                stage.bindings.source_hit_queue,
            ),
        ]);
        previous_shade = stage.shade_name;
    }
    registrations.push(ProductionPassRegistration::new(
        bounce_names,
        true,
        Viewer::create_wavefront_bounce_chain,
        bounce_dependencies,
    ));
    add(
        &mut registrations,
        &["ComputeWavefrontCompositePass"],
        true,
        Viewer::create_wavefront_composite_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontTransparencyResolvePass"],
        true,
        Viewer::create_wavefront_transparency_resolve_pass,
    );
    add(
        &mut registrations,
        &[
            "ComputeWavefrontDenoiseTemporalPass",
            "ComputeWavefrontDenoiseBilateralPass",
        ],
        true,
        Viewer::create_wavefront_denoiser_passes,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontUpscalePass"],
        true,
        Viewer::create_wavefront_upscale_pass,
    );
    add(
        &mut registrations,
        &["FinalizePass"],
        true,
        Viewer::create_finalize_pass,
    );
    add(
        &mut registrations,
        &["BlitPass"],
        true,
        Viewer::create_blit_pass,
    );

    let mut add_dependency = |pass: &str, predecessor: &str, resource: Uid| {
        registrations
            .iter_mut()
            .find(|registration| registration.passes.iter().any(|entry| entry.name == pass))
            .expect("production dependency references an unregistered pass")
            .dependencies
            .push(dependency(pass, predecessor, resource));
    };
    add_dependency(
        "ComputeWavefrontDispatchPass",
        "ComputeWavefrontPrimaryPass",
        WAVEFRONT_PRIMARY_HEADER_ID,
    );
    add_dependency(
        "ComputeWavefrontTraversalPass",
        "ComputeWavefrontDispatchPass",
        WAVEFRONT_PRIMARY_QUEUE_ID,
    );
    add_dependency(
        "ComputeWavefrontHitDispatchPass",
        "ComputeWavefrontTraversalPass",
        WAVEFRONT_HIT_HEADER_ID,
    );
    add_dependency(
        "ComputeWavefrontShadePass",
        "ComputeWavefrontHitDispatchPass",
        WAVEFRONT_HIT_QUEUE_ID,
    );
    let auxiliary = wavefront_auxiliary_bindings();
    add_dependency(
        "ComputeWavefrontTransparencyResolvePass",
        "ComputeWavefrontTransparencyPass",
        auxiliary.transparent_queue,
    );
    add_dependency(
        "ComputeWavefrontAoResolvePass",
        "ComputeWavefrontSurfacePass",
        auxiliary.ao_state,
    );
    add_dependency(
        "ComputeWavefrontVisibilityPass",
        "ComputeWavefrontDirectCandidatesPass",
        WAVEFRONT_RESERVOIR_ID,
    );

    registrations
}

/// Compatibility view over the registration table for callers interested only
/// in pass names and enabled flags.
pub fn production_pass_metadata() -> Vec<ProductionPassMetadata> {
    production_pass_registrations()
        .into_iter()
        .flat_map(|registration| registration.passes)
        .collect()
}

pub fn active_registered_pass_names(render_context: &RenderContext) -> Vec<String> {
    registered_pass_metadata(render_context)
        .into_iter()
        .filter(|pass| pass.enabled)
        .map(|pass| pass.name)
        .collect()
}

/// Read the pass metadata recorded by the actual render-context registration.
pub fn registered_pass_metadata(render_context: &RenderContext) -> Vec<ProductionPassMetadata> {
    render_context
        .passes()
        .iter()
        .map(|(pass, enabled)| ProductionPassMetadata {
            name: pass.name().to_owned(),
            enabled: *enabled,
        })
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

        // Keep render targets at logical client resolution. The surface is
        // configured in physical pixels, but queue storage must not scale with
        // the monitor DPI or exhaust GPU memory on high-DPI displays.
        let initial_dimensions = (window.width(), window.height());
        let context_rc = context.clone();
        let _renderer = Renderer::new(window.handle(), context, move |render_context| {
            Self::create_data_buffers(render_context, initial_dimensions.0, initial_dimensions.1);
            Self::create_systems(&context_rc, render_context);
            Self::create_render_targets(render_context, initial_dimensions.0, initial_dimensions.1);
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

        #[cfg(not(target_arch = "wasm32"))]
        render_context.add_post_submit_callback(report_wavefront_queue_diagnostics);

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
        let registrations = production_pass_registrations();
        for registration in &registrations {
            (registration.factory)(context, render_context);
        }

        let observed = registered_pass_metadata(render_context);
        let expected = registrations
            .iter()
            .flat_map(|registration| registration.passes.iter().cloned())
            .collect::<Vec<_>>();
        assert_eq!(
            observed, expected,
            "actual production pass registration diverged from the registration table"
        );

        //Self::create_debug_pass(context, render_context);
        Self::create_wireframe_pass(
            context,
            render_context,
            render_context.capabilities().wireframe,
        );
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
        validate_construction_record("ComputeGBufferPass", false, &[]);
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
        validate_construction_record("ComputeShadowAOPass", false, &[]);
        let mut shadow_ao_pass = ComputeShadowAOPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        shadow_ao_pass.set_dimensions(visibility_texture.get().dimensions());
        // Replaced by the packed AO/direct queues.
        render_context.add_pass(shadow_ao_pass, false);
    }
    fn create_diffuse_lighting_pass(context: &ContextRc, render_context: &RenderContextRc) {
        validate_construction_record("ComputeDiffuseLightingPass", false, &[]);
        let mut diffuse_pass = ComputeDiffuseLightingPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        diffuse_pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(diffuse_pass, false);
    }
    fn create_specular_lighting_pass(context: &ContextRc, render_context: &RenderContextRc) {
        validate_construction_record("ComputeSpecularLightingPass", false, &[]);
        let mut specular_pass = ComputeSpecularLightingPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        specular_pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(specular_pass, false);
    }
    fn create_indirect_setup_pass(context: &ContextRc, render_context: &RenderContextRc) {
        validate_construction_record("ComputeIndirectSetupPass", false, &[]);
        let mut indirect_setup_pass = ComputeIndirectSetupPass::create(context, render_context);
        let visibility_texture =
            render_context.render_target(RenderTargetType::Visibility as usize);
        indirect_setup_pass.set_dimensions(visibility_texture.get().dimensions());
        render_context.add_pass(indirect_setup_pass, false);
    }
    fn create_indirect_bounce_pass(context: &ContextRc, render_context: &RenderContextRc) {
        validate_construction_record("ComputeIndirectBouncePass", false, &[]);
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
        let auxiliary = wavefront_auxiliary_bindings();
        validate_construction_record(
            "ComputeWavefrontTransparencyPass",
            true,
            &[auxiliary.transparent_queue, auxiliary.transparent_header],
        );
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
        let auxiliary = wavefront_auxiliary_bindings();
        validate_construction_record("ComputeWavefrontAoResolvePass", true, &[auxiliary.ao_state]);
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
        let auxiliary = wavefront_auxiliary_bindings();
        validate_construction_record(
            "ComputeWavefrontDirectCandidatesPass",
            true,
            &[auxiliary.ao_state],
        );
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
        for stage in wavefront_bounce_stages() {
            validate_construction_record(
                &stage.name,
                true,
                &[
                    stage.bindings.source_ray_queue,
                    stage.bindings.source_hit_header,
                    stage.bindings.destination_ray_queue,
                    stage.bindings.destination_ray_header,
                    stage.bindings.dispatch_args,
                ],
            );
            validate_construction_record(
                &stage.dispatch_name,
                true,
                &[
                    stage.bindings.destination_ray_header,
                    stage.destination_dispatch_args,
                ],
            );
            validate_construction_record(
                &stage.traversal_name,
                true,
                &[
                    stage.traversal_bindings.ray_queue,
                    stage.traversal_bindings.ray_header,
                    stage.traversal_bindings.dispatch_args,
                ],
            );
            validate_construction_record(
                &stage.hit_dispatch_name,
                true,
                &[
                    stage.bindings.source_hit_header,
                    WAVEFRONT_HIT_DISPATCH_ARGS_ID,
                ],
            );
            validate_construction_record(
                &stage.shade_name,
                true,
                &[
                    stage.shade_bindings.ray_queue,
                    stage.shade_bindings.ray_header,
                    stage.shade_bindings.hit_queue,
                    stage.shade_bindings.hit_header,
                    stage.shade_bindings.dispatch_args,
                ],
            );
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
        let auxiliary = wavefront_auxiliary_bindings();
        validate_construction_record(
            "ComputeWavefrontTransparencyResolvePass",
            true,
            &[auxiliary.transparent_queue, auxiliary.transparent_header],
        );
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
