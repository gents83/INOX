use std::collections::HashSet;

use inox_commands::CommandParser;
use inox_graphics::{
    wavefront_auxiliary_bindings, wavefront_bounce_stage, wavefront_primary_bindings,
    wavefront_primary_shade_bindings, WAVEFRONT_AO_ID, WAVEFRONT_HIT_DISPATCH_ARGS_ID,
    WAVEFRONT_HIT_HEADER_ID, WAVEFRONT_HIT_QUEUE_ID, WAVEFRONT_PRIMARY_QUEUE_ID,
    WAVEFRONT_TRANSPARENT_HEADER_ID, WAVEFRONT_TRANSPARENT_QUEUE_ID,
};
use inox_platform::window_size_from_parser;
use inox_viewer::viewer::{production_construction_records, production_pass_registrations, ProductionPassMetadata};

fn pass_metadata() -> Vec<ProductionPassMetadata> {
    production_pass_registrations()
        .into_iter()
        .flat_map(|registration| registration.passes)
        .collect()
}

fn active_pass_names() -> Vec<String> {
    pass_metadata()
        .into_iter()
        .filter(|pass| pass.enabled)
        .map(|pass| pass.name)
        .collect()
}

#[test]
fn initial_viewer_dimensions_are_clamped_before_resource_creation() {
    let source = include_str!("../src/viewer.rs");
    assert!(source.contains("Window::create("));
    assert!(source.contains("FrameDimensions::new(window.width(), window.height(), 1.0)"));
    assert!(source.contains("render_context.set_frame_dimensions(initial_dimensions)"));
    assert!(source.contains("initial_width, initial_height"));

    let parser = CommandParser::from_string("-window_size 320 240");
    assert_eq!(window_size_from_parser(&parser, 1920, 1080), (320, 240));
}

#[test]
fn window_system_republishes_initial_physical_metrics_after_ui_registration() {
    let source = include_str!("../../../core/src/systems/window_system.rs");
    assert!(source.contains("WindowEvent::ScaleFactorChanged(self.window.scale_factor())"));
    assert!(source.contains("WindowEvent::SizeChanged("));
    assert!(source.contains("self.window.width()"));
    assert!(source.contains("self.window.height()"));
}

#[test]
fn viewer_camera_projection_uses_clamped_resize_event_dimensions() {
    let source = include_str!("../src/systems/viewer_system.rs");
    assert!(source.contains("fn update_camera_projections(&mut self, dimensions: (u32, u32))"));
    assert!(source.contains("dimensions.0.max(1)"));
    assert!(source.contains("dimensions.1.max(1)"));
    assert!(source.contains("if let WindowEvent::SizeChanged(width, height) = event {"));
    assert!(source.contains("resize_dimensions = Some((*width, *height));"));
    assert!(source.contains("if let Some(dimensions) = resize_dimensions {"));
    assert!(source.contains("self.update_camera_projections(dimensions);"));
}

#[test]
fn viewer_reapplies_current_projection_for_late_camera_resources() {
    let source = include_str!("../src/systems/viewer_system.rs");
    let events = source
        .find("fn update_events(&mut self)")
        .expect("viewer event update must exist");
    let fallback = source[events..]
        .find("let dimensions = self.render_context.frame_dimensions().surface;")
        .expect("event update must refresh projection dimensions");
    let fallback = &source[events + fallback..];
    assert!(fallback.contains("self.update_camera_projections((dimensions.width, dimensions.height));"));
    assert!(source.contains("self.check_command_line_arguments();"));
    assert!(source.contains("self.update_camera_projections((dimensions.width, dimensions.height));"));
}

#[test]
fn production_metadata_has_unique_entries_and_complete_queue_chain() {
    let metadata = pass_metadata();
    let active = active_pass_names();
    let names: HashSet<_> = active.iter().map(String::as_str).collect();
    assert_eq!(names.len(), active.len());
    assert_eq!(active.len(), 63);

    let fixed_prefix = [
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
    ];
    assert_eq!(&active[..fixed_prefix.len()], fixed_prefix);

    let mut position = fixed_prefix.len();
    let mut source_queue = WAVEFRONT_PRIMARY_QUEUE_ID;
    let mut source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    for bounce in 1..=8 {
        let stage = wavefront_bounce_stage(bounce, source_queue, source_dispatch);
        let expected = [
            stage.name.as_str(),
            stage.dispatch_name.as_str(),
            stage.traversal_name.as_str(),
            stage.hit_dispatch_name.as_str(),
            stage.shade_name.as_str(),
        ];
        assert_eq!(&active[position..position + expected.len()], expected);
        position += expected.len();
        source_queue = stage.bindings.destination_ray_queue;
        source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    }
    assert_eq!(
        active[position..]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "ComputeWavefrontCompositePass",
            "ComputeWavefrontTransparencyResolvePass",
            "ComputeWavefrontDenoiseTemporalPass",
            "ComputeWavefrontDenoiseBilateralPass",
            "ComputeWavefrontUpscalePass",
            "FinalizePass",
            "BlitPass",
        ]
    );

    let disabled = metadata
        .into_iter()
        .filter(|pass| !pass.enabled)
        .map(|pass| pass.name)
        .collect::<HashSet<_>>();
    assert_eq!(
        disabled,
        [
            "ComputeGBufferPass",
            "ComputeShadowAOPass",
            "ComputeDiffuseLightingPass",
            "ComputeSpecularLightingPass",
            "ComputeIndirectSetupPass",
            "ComputeIndirectBouncePass",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
}

#[test]
fn registration_table_exposes_actual_auxiliary_and_bounce_handoffs() {
    let registrations = production_pass_registrations();
    let construction = production_construction_records();
    let construction_for = |pass: &str| {
        construction
            .iter()
            .find(|record| record.pass == pass)
            .unwrap_or_else(|| panic!("missing construction record for {pass}"))
    };
    let auxiliary = wavefront_auxiliary_bindings();
    assert_eq!(
        construction_for("ComputeWavefrontTransparencyPass").resources,
        vec![auxiliary.transparent_queue, auxiliary.transparent_header]
    );
    assert_eq!(
        construction_for("ComputeWavefrontTransparencyResolvePass").resources,
        vec![auxiliary.transparent_queue, auxiliary.transparent_header]
    );
    assert_eq!(
        construction_for("ComputeWavefrontDirectCandidatesPass").resources,
        vec![auxiliary.ao_state]
    );
    assert_eq!(
        construction_for("ComputeWavefrontAoResolvePass").resources,
        vec![auxiliary.ao_state]
    );
    for legacy in [
        "ComputeGBufferPass",
        "ComputeShadowAOPass",
        "ComputeDiffuseLightingPass",
        "ComputeSpecularLightingPass",
        "ComputeIndirectSetupPass",
        "ComputeIndirectBouncePass",
    ] {
        assert!(!construction_for(legacy).enabled);
    }
    let dependency = |pass: &str| {
        registrations
            .iter()
            .flat_map(|registration| registration.dependencies.iter())
            .find(|entry| entry.pass == pass)
            .unwrap_or_else(|| panic!("missing dependency metadata for {pass}"))
    };

    let bounce_registration = registrations
        .iter()
        .find(|registration| {
            registration
                .passes
                .iter()
                .any(|pass| pass.name == "ComputeWavefrontBounce1")
        })
        .expect("missing generated bounce registration");
    assert_eq!(bounce_registration.passes.len(), 40);
    assert_eq!(bounce_registration.dependencies.len(), 40);
    for stage in inox_graphics::wavefront_bounce_stages() {
        assert_eq!(
            construction_for(&stage.name).resources,
            vec![
                stage.bindings.source_ray_queue,
                stage.bindings.source_hit_header,
                stage.bindings.destination_ray_queue,
                stage.bindings.destination_ray_header,
                stage.bindings.dispatch_args,
            ]
        );
        assert_eq!(
            construction_for(&stage.dispatch_name).resources,
            vec![
                stage.bindings.destination_ray_header,
                stage.destination_dispatch_args
            ]
        );
    }
    assert_eq!(
        dependency("ComputeWavefrontTransparencyResolvePass").predecessor,
        "ComputeWavefrontTransparencyPass"
    );
    assert_eq!(
        dependency("ComputeWavefrontTransparencyResolvePass").resource,
        WAVEFRONT_TRANSPARENT_QUEUE_ID
    );
    assert_eq!(
        dependency("ComputeWavefrontAoResolvePass").predecessor,
        "ComputeWavefrontSurfacePass"
    );
    assert_eq!(
        dependency("ComputeWavefrontAoResolvePass").resource,
        WAVEFRONT_AO_ID
    );
    assert_eq!(
        dependency("ComputeWavefrontVisibilityPass").predecessor,
        "ComputeWavefrontDirectCandidatesPass"
    );
    assert_eq!(
        dependency("ComputeWavefrontVisibilityPass").resource,
        inox_graphics::WAVEFRONT_RESERVOIR_ID
    );

    let stage = wavefront_bounce_stage(
        1,
        WAVEFRONT_PRIMARY_QUEUE_ID,
        WAVEFRONT_HIT_DISPATCH_ARGS_ID,
    );
    assert_eq!(
        dependency(&stage.name).predecessor,
        "ComputeWavefrontShadePass"
    );
    assert_eq!(dependency(&stage.dispatch_name).predecessor, stage.name);
    assert_eq!(
        dependency(&stage.traversal_name).predecessor,
        stage.dispatch_name
    );
    assert_eq!(
        dependency(&stage.hit_dispatch_name).predecessor,
        stage.traversal_name
    );
    assert_eq!(
        dependency(&stage.shade_name).predecessor,
        stage.hit_dispatch_name
    );
}

#[test]
fn wavefront_graph_contract_connects_queues_headers_and_dispatch_resources() {
    let primary = wavefront_primary_bindings();
    assert_eq!(primary.ray_queue, WAVEFRONT_PRIMARY_QUEUE_ID);
    assert_eq!(
        primary.ray_header,
        inox_graphics::WAVEFRONT_PRIMARY_HEADER_ID
    );
    assert_eq!(
        primary.dispatch_args,
        inox_graphics::WAVEFRONT_DISPATCH_ARGS_ID
    );
    let primary_shade = wavefront_primary_shade_bindings();
    assert_eq!(primary_shade.ray_queue, primary.ray_queue);
    assert_eq!(primary_shade.ray_header, primary.ray_header);
    assert_eq!(primary_shade.hit_queue, primary.hit_queue);
    assert_eq!(primary_shade.hit_header, primary.hit_header);
    assert_eq!(primary_shade.dispatch_args, WAVEFRONT_HIT_DISPATCH_ARGS_ID);
    assert_eq!(primary.hit_queue, WAVEFRONT_HIT_QUEUE_ID);
    assert_eq!(primary.hit_header, WAVEFRONT_HIT_HEADER_ID);

    let mut source_queue = WAVEFRONT_PRIMARY_QUEUE_ID;
    let mut source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    for index in 1..=8 {
        let stage = wavefront_bounce_stage(index, source_queue, source_dispatch);
        assert_eq!(stage.bindings.source_ray_queue, source_queue);
        assert_eq!(stage.bindings.source_hit_queue, WAVEFRONT_HIT_QUEUE_ID);
        assert_eq!(stage.bindings.source_hit_header, WAVEFRONT_HIT_HEADER_ID);
        assert_eq!(stage.bindings.dispatch_args, source_dispatch);
        assert_eq!(
            stage.traversal_bindings.ray_queue,
            stage.bindings.destination_ray_queue
        );
        assert_eq!(
            stage.traversal_bindings.ray_header,
            stage.bindings.destination_ray_header
        );
        assert_eq!(
            stage.traversal_bindings.dispatch_args,
            stage.destination_dispatch_args
        );
        assert_eq!(stage.traversal_bindings.hit_queue, WAVEFRONT_HIT_QUEUE_ID);
        assert_eq!(stage.traversal_bindings.hit_header, WAVEFRONT_HIT_HEADER_ID);
        assert_eq!(
            stage.shade_bindings.ray_queue,
            stage.traversal_bindings.ray_queue
        );
        assert_eq!(
            stage.shade_bindings.ray_header,
            stage.traversal_bindings.ray_header
        );
        assert_eq!(stage.shade_bindings.hit_queue, WAVEFRONT_HIT_QUEUE_ID);
        assert_eq!(stage.shade_bindings.hit_header, WAVEFRONT_HIT_HEADER_ID);
        assert_eq!(
            stage.shade_bindings.dispatch_args,
            WAVEFRONT_HIT_DISPATCH_ARGS_ID
        );
        source_queue = stage.bindings.destination_ray_queue;
        source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    }

    let auxiliary = wavefront_auxiliary_bindings();
    assert_eq!(auxiliary.transparent_queue, WAVEFRONT_TRANSPARENT_QUEUE_ID);
    assert_eq!(
        auxiliary.transparent_header,
        WAVEFRONT_TRANSPARENT_HEADER_ID
    );
    assert_eq!(auxiliary.ao_state, WAVEFRONT_AO_ID);
}

#[test]
fn production_pass_contract_keeps_dependencies_and_reconstruction_tail_ordered() {
    let active = active_pass_names();
    let position = |name: &str| active.iter().position(|pass| pass == name).unwrap();
    assert!(position("DepthFirstPass") < position("VisibilityBufferPass"));
    assert!(
        position("ComputeWavefrontSurfacePass") < position("ComputeWavefrontDirectCandidatesPass")
    );
    assert!(
        position("ComputeWavefrontDirectCandidatesPass")
            < position("ComputeWavefrontVisibilityPass")
    );
    assert!(position("ComputeWavefrontVisibilityPass") < position("ComputeWavefrontCompositePass"));
    assert!(position("ComputeWavefrontCompositePass") < position("FinalizePass"));
    assert!(position("ComputeWavefrontUpscalePass") < position("FinalizePass"));
    assert!(
        position("ComputeWavefrontTransparencyPass")
            < position("ComputeWavefrontTransparencyResolvePass")
    );
    assert!(position("ComputeWavefrontAoResolvePass") < position("ComputeWavefrontCompositePass"));
}
