#[cfg(not(target_arch = "wasm32"))]
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::{collections::HashMap, path::PathBuf};

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};
use inox_graph::{Dependency, Graph, GraphError, GraphValidation, PassNode};

#[cfg(not(target_arch = "wasm32"))]
use inox_graphics::{report_cornell_image_diagnostics, report_wavefront_queue_diagnostics};
use inox_graphics::{
    wavefront_auxiliary_bindings, wavefront_bounce_stages, BlitPass, CommandsPass,
    ComputeDiffuseLightingPass, ComputeGBufferPass, ComputeIndirectBouncePass,
    ComputeIndirectSetupPass, ComputeInstancesPass, ComputeShadowAOPass,
    ComputeSpecularLightingPass, ComputeWavefrontAoResolvePass, ComputeWavefrontCompositePass,
    ComputeWavefrontDenoisePass, ComputeWavefrontDirectCandidatesPass,
    ComputeWavefrontDispatchPass, ComputeWavefrontNextBouncePass, ComputeWavefrontPrimaryPass,
    ComputeWavefrontShadePass, ComputeWavefrontSurfacePass, ComputeWavefrontTransparencyPass,
    ComputeWavefrontTransparencyResolvePass, ComputeWavefrontTraversalPass,
    ComputeWavefrontUpscalePass, ComputeWavefrontVisibilityPass, CullingPass, DebugOptions,
    DebugPass, DepthFirstPass, DepthPyramidPass, DrawPreparationResults, FinalizePass,
    RadiancePackedData, RayPackedData, ThroughputPackedData, VisibilityBufferPass,
    WavefrontBounceBindingIds, WavefrontQueueBindingIds, WireframePass, ACTIVE_INSTANCE_DATA_ID,
    DRAW_PREPARATION_RESULT_BUFFER_ID, WAVEFRONT_AO_ID, WAVEFRONT_DISPATCH_ARGS_ID,
    WAVEFRONT_FINAL_ACCUMULATOR_ID, WAVEFRONT_HIT_DISPATCH_ARGS_ID, WAVEFRONT_HIT_HEADER_ID,
    WAVEFRONT_HIT_QUEUE_ID, WAVEFRONT_INDIRECT_ACCUMULATOR_ID, WAVEFRONT_NEXT2_DISPATCH_ARGS_ID,
    WAVEFRONT_NEXT2_HEADER_ID, WAVEFRONT_NEXT2_QUEUE_ID, WAVEFRONT_NEXT3_DISPATCH_ARGS_ID,
    WAVEFRONT_NEXT3_HEADER_ID, WAVEFRONT_NEXT3_QUEUE_ID, WAVEFRONT_NEXT_DISPATCH_ARGS_ID,
    WAVEFRONT_NEXT_HEADER_ID, WAVEFRONT_NEXT_QUEUE_ID, WAVEFRONT_PRIMARY_HEADER_ID,
    WAVEFRONT_PRIMARY_QUEUE_ID, WAVEFRONT_RESERVOIR_ID, WAVEFRONT_SCENE_ID,
    WAVEFRONT_UPSCALED_ACCUMULATOR_ID, WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
#[cfg(target_arch = "wasm32")]
use inox_render::signal_browser_smoke_marker;
use inox_render::{
    rendering_system::RenderingSystem, update_system::UpdateSystem, AsBinding, DrawIndexedCommand,
    FrameDimensions, GPUInstance, GPULight, GPUMaterial, GPUTexture, Pass, RenderContext,
    RenderContextRc, RenderPass, Renderer, RendererRw, SceneTableKind, TextureFormat, TextureUsage,
    DEFAULT_HEIGHT, DEFAULT_WIDTH,
};
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::{read_from_file, SerializationType};
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};
use inox_uid::{generate_uid_from_type, Uid};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_UI_PASS: bool = true;
const DEBUG_OPTIONS: DebugOptions = DebugOptions::disabled();

const MAX_NUM_LIGHTS: usize = 512;
const MAX_NUM_TEXTURES: usize = 2048;
const MAX_NUM_MATERIALS: usize = 256;

pub type ProductionPassFactory = fn(&ContextRc, &RenderContextRc);

/// Generic graph metadata exposed under the legacy viewer names for source
/// compatibility. Factories remain viewer-specific; nodes and edges do not.
pub type ProductionPassMetadata = PassNode<Uid>;
pub type ProductionGraphDependency = Dependency<Uid>;

pub struct ProductionPassRegistration {
    pub passes: Vec<ProductionPassMetadata>,
    pub dependencies: Vec<ProductionGraphDependency>,
    factory: ProductionPassFactory,
    /// Disabled legacy passes still register for compatibility, but a
    /// capability-gated wavefront registration must not construct pipelines.
    construct: bool,
}

impl ProductionPassRegistration {
    pub fn constructs_pipelines(&self) -> bool {
        self.construct
    }

    fn new_with_construction(
        names: impl IntoIterator<Item = String>,
        enabled: bool,
        factory: ProductionPassFactory,
        dependencies: Vec<ProductionGraphDependency>,
        construct: bool,
    ) -> Self {
        Self {
            passes: names
                .into_iter()
                .map(|name| production_pass_node(name, enabled))
                .collect(),
            dependencies,
            factory,
            construct,
        }
    }
}

fn dependency(pass: &str, predecessor: &str, resource: Uid) -> ProductionGraphDependency {
    Dependency::new(predecessor, pass, resource)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionConstructionRecord {
    pub pass: String,
    pub enabled: bool,
    pub resources: Vec<Uid>,
}

fn wavefront_gbuffer_resource_ids() -> [Uid; 3] {
    // These are the type-derived IDs used by the concrete surface, primary,
    // transparency, and direct-candidate bindings. Keeping the graph on the
    // same IDs makes the packed G-buffer handoff explicit and verifiable.
    [
        generate_uid_from_type::<RayPackedData>(),
        generate_uid_from_type::<RadiancePackedData>(),
        generate_uid_from_type::<ThroughputPackedData>(),
    ]
}

fn production_pass_node(name: String, enabled: bool) -> ProductionPassMetadata {
    let auxiliary = wavefront_auxiliary_bindings();
    let [gbuffer_world, gbuffer_material, gbuffer_auxiliary] = wavefront_gbuffer_resource_ids();
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    match name.as_str() {
        "ComputeWavefrontSurfacePass" => {
            writes.extend([gbuffer_world, gbuffer_material, gbuffer_auxiliary]);
        }
        "ComputeWavefrontTransparencyPass" => {
            reads.push(gbuffer_material);
            writes.extend([auxiliary.transparent_queue, auxiliary.transparent_header]);
        }
        "ComputeWavefrontTransparencyResolvePass" => {
            reads.extend([auxiliary.transparent_queue, auxiliary.transparent_header]);
            writes.push(WAVEFRONT_FINAL_ACCUMULATOR_ID);
        }
        "ComputeWavefrontDirectCandidatesPass" => {
            reads.extend([gbuffer_world, gbuffer_material]);
            writes.extend([
                WAVEFRONT_RESERVOIR_ID,
                WAVEFRONT_AO_ID,
                inox_graphics::WAVEFRONT_DIRECT_ACCUMULATOR_ID,
            ]);
        }
        "ComputeWavefrontAoResolvePass" => {
            reads.push(WAVEFRONT_AO_ID);
            writes.push(WAVEFRONT_AO_ID);
        }
        "ComputeWavefrontVisibilityPass" => {
            reads.extend([gbuffer_material, WAVEFRONT_RESERVOIR_ID, WAVEFRONT_AO_ID]);
            writes.push(inox_graphics::WAVEFRONT_DIRECT_ACCUMULATOR_ID);
        }
        "ComputeWavefrontCompositePass" => {
            reads.extend([
                WAVEFRONT_FINAL_ACCUMULATOR_ID,
                inox_graphics::WAVEFRONT_DIRECT_ACCUMULATOR_ID,
                WAVEFRONT_INDIRECT_ACCUMULATOR_ID,
                WAVEFRONT_AO_ID,
            ]);
            writes.push(WAVEFRONT_FINAL_ACCUMULATOR_ID);
        }
        "ComputeWavefrontDenoiseTemporalPass" | "ComputeWavefrontDenoiseBilateralPass" => {
            reads.push(WAVEFRONT_FINAL_ACCUMULATOR_ID);
            writes.push(WAVEFRONT_FINAL_ACCUMULATOR_ID);
        }
        "ComputeWavefrontUpscalePass" => {
            reads.push(WAVEFRONT_FINAL_ACCUMULATOR_ID);
            writes.push(WAVEFRONT_UPSCALED_ACCUMULATOR_ID);
        }
        "FinalizePass" => {
            reads.push(WAVEFRONT_UPSCALED_ACCUMULATOR_ID);
        }
        "ComputeWavefrontPrimaryPass" => {
            reads.extend([gbuffer_world, gbuffer_material, gbuffer_auxiliary]);
            writes.extend([WAVEFRONT_PRIMARY_QUEUE_ID, WAVEFRONT_PRIMARY_HEADER_ID]);
        }
        "ComputeWavefrontDispatchPass" => {
            reads.push(WAVEFRONT_PRIMARY_HEADER_ID);
            writes.push(WAVEFRONT_DISPATCH_ARGS_ID);
        }
        "ComputeWavefrontTraversalPass" => {
            reads.extend([
                WAVEFRONT_PRIMARY_QUEUE_ID,
                WAVEFRONT_PRIMARY_HEADER_ID,
                WAVEFRONT_DISPATCH_ARGS_ID,
            ]);
            writes.extend([WAVEFRONT_HIT_QUEUE_ID, WAVEFRONT_HIT_HEADER_ID]);
        }
        "ComputeWavefrontHitDispatchPass" => {
            reads.push(WAVEFRONT_HIT_HEADER_ID);
            writes.push(WAVEFRONT_HIT_DISPATCH_ARGS_ID);
        }
        "ComputeWavefrontShadePass" => {
            reads.extend([
                WAVEFRONT_PRIMARY_QUEUE_ID,
                WAVEFRONT_PRIMARY_HEADER_ID,
                WAVEFRONT_HIT_QUEUE_ID,
                WAVEFRONT_HIT_HEADER_ID,
                WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            ]);
            writes.push(WAVEFRONT_INDIRECT_ACCUMULATOR_ID);
        }
        _ => {
            if let Some(stage) = wavefront_bounce_stages().into_iter().find(|stage| {
                name == stage.name
                    || name == stage.dispatch_name
                    || name == stage.traversal_name
                    || name == stage.hit_dispatch_name
                    || name == stage.shade_name
            }) {
                if name == stage.name {
                    reads.extend([
                        stage.bindings.source_ray_queue,
                        stage.bindings.source_hit_queue,
                        stage.bindings.source_hit_header,
                        stage.bindings.dispatch_args,
                    ]);
                    writes.extend([
                        stage.bindings.destination_ray_queue,
                        stage.bindings.destination_ray_header,
                    ]);
                } else if name == stage.dispatch_name {
                    reads.push(stage.bindings.destination_ray_header);
                    writes.push(stage.destination_dispatch_args);
                } else if name == stage.traversal_name {
                    reads.extend([
                        stage.traversal_bindings.ray_queue,
                        stage.traversal_bindings.ray_header,
                        stage.traversal_bindings.dispatch_args,
                    ]);
                    writes.extend([WAVEFRONT_HIT_QUEUE_ID, WAVEFRONT_HIT_HEADER_ID]);
                } else if name == stage.hit_dispatch_name {
                    reads.push(WAVEFRONT_HIT_HEADER_ID);
                    writes.push(WAVEFRONT_HIT_DISPATCH_ARGS_ID);
                } else {
                    reads.extend([
                        stage.shade_bindings.ray_queue,
                        stage.shade_bindings.ray_header,
                        stage.shade_bindings.hit_queue,
                        stage.shade_bindings.hit_header,
                        stage.shade_bindings.dispatch_args,
                    ]);
                    writes.push(WAVEFRONT_INDIRECT_ACCUMULATOR_ID);
                }
            }
        }
    }
    PassNode::new(name, enabled, reads, writes)
}

/// Construction inputs shared by the production factories and graph tests.
/// The bounce factory consumes `wavefront_bounce_stages()` directly; auxiliary
/// factories consume the same `wavefront_auxiliary_bindings()` values below.
pub fn production_construction_records() -> Vec<ProductionConstructionRecord> {
    production_construction_records_with_wavefront(true)
}

pub fn production_construction_records_with_wavefront(
    wavefront_layout_supported: bool,
) -> Vec<ProductionConstructionRecord> {
    production_graph_with_wavefront(wavefront_layout_supported)
        .nodes()
        .iter()
        .map(|node| {
            let mut resources = node.reads.clone();
            resources.extend(node.writes.iter().copied());
            resources.sort_unstable();
            resources.dedup();
            ProductionConstructionRecord {
                pass: node.name.clone(),
                enabled: node.enabled,
                resources,
            }
        })
        .collect()
}

fn validate_construction_record(pass: &str, enabled: bool, resources: &[Uid]) {
    let graph = production_graph();
    let node = graph
        .nodes()
        .iter()
        .find(|node| node.name == pass)
        .unwrap_or_else(|| panic!("missing graph node for {pass}"));
    assert_eq!(node.enabled, enabled, "graph flag diverged for {pass}");
    for resource in resources {
        assert!(
            node.reads
                .iter()
                .chain(node.writes.iter())
                .any(|entry| entry == resource),
            "graph metadata is missing construction resource for {pass}"
        );
    }
}

/// The single production registration table. Its factories construct the
/// actual passes, while the same rows expose observed names, enabled flags,
/// and graph handoffs to integration tests.
pub fn production_pass_registrations() -> Vec<ProductionPassRegistration> {
    production_pass_registrations_with_wavefront(true)
}

/// Build production metadata for the adapter's negotiated layout capability.
/// When the full wavefront binding layout is unavailable, wavefront nodes are
/// disabled and their factories are not invoked, so no incompatible pipelines
/// or bind groups are created.
pub fn production_pass_registrations_with_wavefront(
    wavefront_layout_supported: bool,
) -> Vec<ProductionPassRegistration> {
    let mut registrations = Vec::new();
    let add =
        |registrations: &mut Vec<ProductionPassRegistration>, names: &[&str], enabled, factory| {
            let is_wavefront = names
                .iter()
                .any(|name| name.starts_with("ComputeWavefront"));
            registrations.push(ProductionPassRegistration::new_with_construction(
                names.iter().map(|name| (*name).to_owned()),
                enabled && (!is_wavefront || wavefront_layout_supported),
                factory,
                Vec::new(),
                !is_wavefront || wavefront_layout_supported,
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
        &["ComputeWavefrontDirectCandidatesPass"],
        true,
        Viewer::create_wavefront_direct_candidates_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontAoResolvePass"],
        true,
        Viewer::create_wavefront_ao_resolve_pass,
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

    let mut ray_queue_producers = HashMap::from([(
        WAVEFRONT_PRIMARY_QUEUE_ID,
        "ComputeWavefrontPrimaryPass".to_owned(),
    )]);
    let mut previous_traversal = "ComputeWavefrontTraversalPass".to_owned();
    let mut previous_hit_dispatch = "ComputeWavefrontHitDispatchPass".to_owned();
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
        // The next-bounce pass has three independent inputs. Keep each
        // producer/resource handoff explicit. Ray queues ping-pong, so the
        // producer is the most recent stage that wrote this queue rather than
        // always the immediately preceding stage.
        let source_ray_producer = ray_queue_producers
            .get(&stage.bindings.source_ray_queue)
            .cloned()
            .expect("every generated bounce source queue has a producer");
        bounce_dependencies.extend([
            dependency(
                &stage.name,
                &source_ray_producer,
                stage.bindings.source_ray_queue,
            ),
            dependency(
                &stage.name,
                &previous_traversal,
                stage.bindings.source_hit_queue,
            ),
            dependency(
                &stage.name,
                &previous_traversal,
                stage.bindings.source_hit_header,
            ),
            dependency(
                &stage.name,
                &previous_hit_dispatch,
                stage.bindings.dispatch_args,
            ),
            dependency(
                &stage.dispatch_name,
                &stage.name,
                stage.bindings.destination_ray_header,
            ),
            dependency(
                &stage.traversal_name,
                &stage.dispatch_name,
                stage.destination_dispatch_args,
            ),
            dependency(
                &stage.hit_dispatch_name,
                &stage.traversal_name,
                WAVEFRONT_HIT_HEADER_ID,
            ),
            dependency(
                &stage.shade_name,
                &stage.hit_dispatch_name,
                WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            ),
        ]);
        ray_queue_producers.insert(stage.bindings.destination_ray_queue, stage.name.clone());
        previous_traversal = stage.traversal_name;
        previous_hit_dispatch = stage.hit_dispatch_name;
    }
    registrations.push(ProductionPassRegistration::new_with_construction(
        bounce_names,
        wavefront_layout_supported,
        Viewer::create_wavefront_bounce_chain,
        bounce_dependencies,
        wavefront_layout_supported,
    ));
    add(
        &mut registrations,
        &["ComputeWavefrontTransparencyResolvePass"],
        true,
        Viewer::create_wavefront_transparency_resolve_pass,
    );
    add(
        &mut registrations,
        &["ComputeWavefrontCompositePass"],
        true,
        Viewer::create_wavefront_composite_pass,
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
    let [gbuffer_world, gbuffer_material, gbuffer_auxiliary] = wavefront_gbuffer_resource_ids();
    for resource in [gbuffer_world, gbuffer_material, gbuffer_auxiliary] {
        add_dependency(
            "ComputeWavefrontPrimaryPass",
            "ComputeWavefrontSurfacePass",
            resource,
        );
    }
    add_dependency(
        "ComputeWavefrontDirectCandidatesPass",
        "ComputeWavefrontSurfacePass",
        gbuffer_material,
    );
    add_dependency(
        "ComputeWavefrontDispatchPass",
        "ComputeWavefrontPrimaryPass",
        WAVEFRONT_PRIMARY_HEADER_ID,
    );
    add_dependency(
        "ComputeWavefrontTraversalPass",
        "ComputeWavefrontDispatchPass",
        WAVEFRONT_DISPATCH_ARGS_ID,
    );
    add_dependency(
        "ComputeWavefrontHitDispatchPass",
        "ComputeWavefrontTraversalPass",
        WAVEFRONT_HIT_HEADER_ID,
    );
    add_dependency(
        "ComputeWavefrontShadePass",
        "ComputeWavefrontHitDispatchPass",
        WAVEFRONT_HIT_DISPATCH_ARGS_ID,
    );
    let auxiliary = wavefront_auxiliary_bindings();
    add_dependency(
        "ComputeWavefrontTransparencyResolvePass",
        "ComputeWavefrontTransparencyPass",
        auxiliary.transparent_queue,
    );
    add_dependency(
        "ComputeWavefrontAoResolvePass",
        "ComputeWavefrontDirectCandidatesPass",
        auxiliary.ao_state,
    );
    add_dependency(
        "ComputeWavefrontVisibilityPass",
        "ComputeWavefrontDirectCandidatesPass",
        WAVEFRONT_RESERVOIR_ID,
    );
    add_dependency(
        "ComputeWavefrontVisibilityPass",
        "ComputeWavefrontAoResolvePass",
        WAVEFRONT_AO_ID,
    );
    add_dependency(
        "ComputeWavefrontCompositePass",
        "ComputeWavefrontTransparencyResolvePass",
        WAVEFRONT_FINAL_ACCUMULATOR_ID,
    );
    add_dependency(
        "ComputeWavefrontCompositePass",
        "ComputeWavefrontVisibilityPass",
        inox_graphics::WAVEFRONT_DIRECT_ACCUMULATOR_ID,
    );
    let final_bounce_shade = wavefront_bounce_stages()
        .last()
        .expect("production graph requires a final wavefront bounce")
        .shade_name
        .clone();
    add_dependency(
        "ComputeWavefrontCompositePass",
        &final_bounce_shade,
        WAVEFRONT_INDIRECT_ACCUMULATOR_ID,
    );
    add_dependency(
        "ComputeWavefrontDenoiseTemporalPass",
        "ComputeWavefrontCompositePass",
        WAVEFRONT_FINAL_ACCUMULATOR_ID,
    );
    add_dependency(
        "ComputeWavefrontDenoiseBilateralPass",
        "ComputeWavefrontDenoiseTemporalPass",
        WAVEFRONT_FINAL_ACCUMULATOR_ID,
    );
    add_dependency(
        "ComputeWavefrontUpscalePass",
        "ComputeWavefrontDenoiseBilateralPass",
        WAVEFRONT_FINAL_ACCUMULATOR_ID,
    );
    if wavefront_layout_supported {
        add_dependency(
            "FinalizePass",
            "ComputeWavefrontUpscalePass",
            WAVEFRONT_UPSCALED_ACCUMULATOR_ID,
        );
    }

    registrations
}

/// Build the portable graph from the same registration declarations used by
/// the concrete factories. No GPU objects are created while constructing it.
pub fn production_graph() -> Graph<Uid> {
    production_graph_with_wavefront(true)
}

pub fn production_graph_with_wavefront(wavefront_layout_supported: bool) -> Graph<Uid> {
    let registrations = production_pass_registrations_with_wavefront(wavefront_layout_supported);
    let nodes = registrations
        .iter()
        .flat_map(|registration| registration.passes.iter().cloned());
    let dependencies = registrations
        .iter()
        .flat_map(|registration| registration.dependencies.iter().cloned());
    Graph::from_parts(nodes, dependencies)
}

pub fn validate_production_graph() -> Result<GraphValidation, GraphError> {
    production_graph().validate()
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
        .map(|(pass, enabled)| production_pass_node(pass.name().to_owned(), *enabled))
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
        let initial_dimensions = FrameDimensions::new(window.width(), window.height(), 1.0)
            .expect("clamped initial viewer dimensions must fit checked arithmetic");
        let initial_width = initial_dimensions.surface.width;
        let initial_height = initial_dimensions.surface.height;
        let context_rc = context.clone();
        let _renderer = Renderer::new(window.handle(), context, move |render_context| {
            // The surface starts with renderer defaults; make the clamped window
            // dimensions authoritative before creating any size-dependent resources.
            render_context.set_frame_dimensions(initial_dimensions);
            Self::create_data_buffers(render_context, initial_width, initial_height);
            Self::create_systems(&context_rc, render_context);
            Self::create_render_targets(render_context, initial_width, initial_height);
            Self::create_render_passes(&context_rc, render_context, DEBUG_OPTIONS);
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
    #[cfg(not(target_arch = "wasm32"))]
    fn native_cornell_smoke_enabled() -> bool {
        let parser = inox_commands::CommandParser::from_command_line();
        parser.has("native_cornell_smoke")
            || parser
                .get_values_of::<String>("load_file")
                .iter()
                .any(|path| path.to_ascii_lowercase().contains("cornell_box"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn report_cornell_geometry_diagnostics(render_context: &RenderContext) {
        static REPORTED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        if REPORTED.get().is_some() {
            return;
        }
        let buffers = render_context.global_buffers();
        let active = buffers.vector_with_id::<GPUInstance>(ACTIVE_INSTANCE_DATA_ID);
        let commands = buffers.vector::<DrawIndexedCommand>();
        let active_id = active.read().unwrap().buffer_id();
        let commands_id = commands.read().unwrap().buffer_id();
        let active_gpu = render_context.read_buffer_as::<GPUInstance>(active_id);
        let commands_gpu = render_context.read_buffer_as::<DrawIndexedCommand>(commands_id);
        let preparation = render_context
            .read_buffer_as::<DrawPreparationResults>(DRAW_PREPARATION_RESULT_BUFFER_ID);
        let packed = buffers.packed_scene_data();
        let scene_gpu = render_context.read_buffer_as::<u32>(
            buffers
                .vector_with_id::<u32>(WAVEFRONT_SCENE_ID)
                .read()
                .unwrap()
                .buffer_id(),
        );
        let mesh_range = packed.range(SceneTableKind::Meshes);
        let mesh_materials = (0..(mesh_range.count_words / 16))
            .map(|index| packed.words()[mesh_range.offset_words as usize + index as usize * 16 + 3])
            .collect::<Vec<_>>();
        let instance_range = packed.range(SceneTableKind::Instances);
        let packed_instances = (0..(instance_range.count_words / 4))
            .map(|index| {
                let base = instance_range.offset_words as usize + index as usize * 4;
                [
                    packed.words()[base],
                    packed.words()[base + 1],
                    packed.words()[base + 2],
                ]
            })
            .collect::<Vec<_>>();
        let visibility_samples =
            render_context
                .read_render_target_r32uint(0)
                .map(|(_, _, values)| {
                    let mut histogram = std::collections::BTreeMap::new();
                    let mut first = None;
                    for value in values.iter().filter(|value| **value != 0) {
                        if first.is_none() {
                            first = Some(*value);
                        }
                        *histogram.entry(*value).or_insert(0usize) += 1;
                    }
                    (histogram.into_iter().take(8).collect::<Vec<_>>(), first)
                });
        let visibility_ids = visibility_samples
            .as_ref()
            .map(|(histogram, _)| histogram.clone());
        let visibility_first = visibility_samples.as_ref().and_then(|(_, first)| *first);
        let visibility = visibility_ids
            .as_ref()
            .map(|values| values.iter().map(|(_, count)| *count).sum::<usize>());
        let visibility_tuple = visibility_first.and_then(|value| {
            let encoded_instance = value >> 8;
            if encoded_instance == 0 {
                return None;
            }
            let instance_id = encoded_instance - 1;
            let instance_base = instance_range.offset_words as usize + instance_id as usize * 4;
            let instance = packed.words().get(instance_base..instance_base + 4)?;
            let transform_id = instance[0];
            let mesh_id = instance[1];
            let meshlet_id = instance[2];
            let mesh_base = mesh_range.offset_words as usize + mesh_id as usize * 16;
            let meshlet_range = packed.range(SceneTableKind::Meshlets);
            let meshlet_base = meshlet_range.offset_words as usize + meshlet_id as usize * 20;
            let material_id = *packed.words().get(mesh_base + 3)?;
            packed.words().get(meshlet_base + 2)?;
            Some((
                value,
                instance_id,
                value & 255,
                transform_id,
                mesh_id,
                meshlet_id,
                material_id,
            ))
        });
        let gbuffer_world = buffers.vector::<RayPackedData>();
        let gbuffer_material = buffers.vector::<RadiancePackedData>();
        let world_gpu = render_context
            .read_buffer_as::<RayPackedData>(gbuffer_world.read().unwrap().buffer_id());
        let material_gpu = render_context
            .read_buffer_as::<RadiancePackedData>(gbuffer_material.read().unwrap().buffer_id());
        let world_nonzero = world_gpu
            .as_ref()
            .map(|values| values.iter().filter(|value| value.0 != 0.0).count());
        let material_histogram = material_gpu.as_ref().map(|values| {
            let mut histogram = std::collections::BTreeMap::new();
            for value in values {
                *histogram.entry(value.0.to_bits()).or_insert(0usize) += 1;
            }
            histogram
        });
        let active_nonzero = active_gpu.as_ref().map(|values| {
            values
                .iter()
                .filter(|instance| instance.command_id >= 0)
                .count()
        });
        let command_nonzero = commands_gpu.as_ref().map(|values| {
            values
                .iter()
                .filter(|command| command.instance_count > 0 || command.vertex_count > 0)
                .count()
        });
        eprintln!(
            "INOX_NATIVE_CORNELL_GEOMETRY scene_gpu={:?} active_cpu={} commands_cpu={} active_gpu_nonzero={:?} command_gpu_nonzero={:?} prep={:?} visibility_nonzero={:?} visibility_first={:?} visibility_tuple={:?} visibility_ids={:?} world_nonzero={:?} material_histogram={:?} mesh_materials={:?} packed_instances={:?}",
            scene_gpu.as_ref().map(|values| values.iter().take(12).copied().collect::<Vec<_>>()),
            active.read().unwrap().len(),
            commands.read().unwrap().len(),
            active_nonzero,
            command_nonzero,
            preparation.as_ref().and_then(|values| values.first()).map(|value| (
                value.draw_vertices_count,
                value.active_instances_count,
                value.commands_count,
                value.instances_count,
            )),
            visibility,
            visibility_first,
            visibility_tuple,
            visibility_ids,
            world_nonzero,
            material_histogram,
            mesh_materials,
            packed_instances,
        );
        let _ = REPORTED.set(());
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_cornell_scene_ready(render_context: &RenderContext) -> bool {
        let packed = render_context.global_buffers().packed_scene_data();
        [
            SceneTableKind::Instances,
            SceneTableKind::Meshes,
            SceneTableKind::Materials,
            SceneTableKind::Lights,
        ]
        .iter()
        .all(|kind| packed.range(*kind).count_words > 0)
    }

    fn create_systems(context: &ContextRc, render_context: &RenderContextRc) {
        let render_update_system = UpdateSystem::new(render_context, context);
        let rendering_draw_system = RenderingSystem::new(render_context, context);
        let mut ui_system = if ADD_UI_PASS {
            Some(UISystem::new(context))
        } else {
            None
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            render_context.add_post_submit_callback(report_wavefront_queue_diagnostics);
            if Self::native_cornell_smoke_enabled() {
                // Read back at most eight completed frames. Asset loading can take several
                // submissions, while the explicit mode keeps normal production readback-free.
                let attempts = Arc::new(AtomicU32::new(0));
                let attempts_for_callback = attempts.clone();
                render_context.add_post_submit_callback(move |context| {
                    // Asset loading can span many submitted frames. Do not
                    // spend the bounded image attempts on an empty packed
                    // scene; once mesh/material/light tables exist, every
                    // attempt still uses the unchanged image thresholds.
                    if !Self::native_cornell_scene_ready(context)
                        || context.completed_frame_target().is_none()
                    {
                        return;
                    }
                    Self::report_cornell_geometry_diagnostics(context);
                    let attempt = attempts_for_callback.fetch_add(1, Ordering::AcqRel);
                    if attempt < 8 && report_cornell_image_diagnostics(context) {
                        attempts_for_callback.store(8, Ordering::Release);
                    }
                });
                eprintln!("INOX_NATIVE_CORNELL_IMAGE_SMOKE_ENABLED attempts=8");
            }
        }
        // Browser queue/output markers are emitted only by completed GPU work.
        // Construction proves neither initialized headers nor finite/nonzero pixels.
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
        let hiz_mip_count = FrameDimensions::new(width, height, 1.0)
            .expect("clamped viewer dimensions must fit checked arithmetic")
            .hiz_mip_count();
        let _hzb = render_context.create_mip_chain_render_target(
            (width, height, single_sample, 1, hiz_mip_count),
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
    fn create_render_passes(
        context: &ContextRc,
        render_context: &RenderContextRc,
        debug_options: DebugOptions,
    ) {
        let wavefront_layout_supported = render_context.capabilities().wavefront_layout_supported;
        let registrations =
            production_pass_registrations_with_wavefront(wavefront_layout_supported);
        let graph = production_graph_with_wavefront(wavefront_layout_supported);
        let validation = match graph.validate() {
            Ok(validation) => validation,
            Err(error) => {
                eprintln!("wavefront production graph validation failed: {error}");
                return;
            }
        };
        let declared_active = registrations
            .iter()
            .flat_map(|registration| registration.passes.iter())
            .filter(|pass| pass.enabled)
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>();
        if declared_active
            != validation
                .order
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        {
            eprintln!("wavefront production graph order diverges from concrete registration order");
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!(
            "INOX_NATIVE_WAVEFRONT_GRAPH_OK order={} active_nodes={}",
            validation.order.len(),
            declared_active.len(),
        );
        #[cfg(target_arch = "wasm32")]
        signal_browser_smoke_marker(
            "graph_order",
            &format!("validated:{}", validation.order.len()),
        );
        for registration in &registrations {
            if registration.construct {
                (registration.factory)(context, render_context);
            }
        }

        let observed = registered_pass_metadata(render_context);
        let expected = registrations
            .iter()
            .filter(|registration| registration.construct)
            .flat_map(|registration| registration.passes.iter().cloned())
            .collect::<Vec<_>>();
        assert_eq!(
            observed, expected,
            "actual production pass registration diverged from the registration table"
        );

        Self::create_debug_pass(context, render_context, debug_options);
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
    fn create_debug_pass(
        context: &ContextRc,
        render_context: &RenderContextRc,
        options: DebugOptions,
    ) {
        if !options.is_enabled() {
            return;
        }
        let dimensions = render_context.frame_dimensions();
        let mut debug_pass =
            DebugPass::create_with_options(context, render_context, dimensions, options);
        debug_pass
            .set_visibility_texture(
                &render_context.render_target_id(RenderTargetType::Visibility as usize),
            )
            .set_depth_texture(&render_context.render_target_id(RenderTargetType::Depth as usize))
            .set_finalize_texture(
                &render_context.render_target_id(RenderTargetType::Frame0 as usize),
            );
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
