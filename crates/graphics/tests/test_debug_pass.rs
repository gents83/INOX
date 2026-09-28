use std::{fs, path::PathBuf};

use inox_graphics::{DebugOptions, DebugVisualBuffer};

fn shader_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data_raw/shaders/wgsl")
}

#[test]
fn debug_options_are_disabled_without_explicit_diagnostics() {
    let options = DebugOptions::default();
    assert!(!options.is_enabled());
    assert_eq!(options.visual_buffer, DebugVisualBuffer::None);
}

#[test]
fn debug_pass_contract_uses_runtime_dimensions_and_bounds() {
    let source = fs::read_to_string(shader_root().join("debug.wgsl")).unwrap();
    assert!(!source.contains("DEFAULT_WIDTH"));
    assert!(!source.contains("DEFAULT_HEIGHT"));
    assert!(source.contains("u32(constant_data.screen_width)"));
    assert!(source.contains("u32(constant_data.screen_height)"));
    assert!(source.contains("arrayLength(&data_buffer_debug)"));
    assert!(source.contains("arrayLength(&instances.data)"));
    assert!(source.contains("arrayLength(&meshlets.data)"));
    assert!(source.contains("arrayLength(&transforms.data)"));
    assert!(source.contains("arrayLength(&vertices_positions.data)"));
    assert!(source.contains("arrayLength(&vertices_attributes.data)"));
    assert!(source.contains("MAX_NUM_MATERIALS"));
    assert!(source.contains("valid_visibility_id(visibility_id)"));
    assert!(source.contains("valid_visibility_id(debug_visibility_id)"));
    assert!(source.contains("sanitize_debug_uv"));
    assert!(source.contains("visibility_dimensions - vec2<u32>(1u)"));
    let validation_block = source
        .split("fn valid_visibility_id")
        .nth(1)
        .and_then(|tail| tail.split("fn draw_triangle_from_visibility").next())
        .expect("debug shader must contain a complete visibility validator");
    let attribute_bases = [
        "let attribute_base_x",
        "let attribute_base_y",
        "let attribute_base_z",
    ]
    .map(|assignment| {
        validation_block
            .find(assignment)
            .unwrap_or_else(|| panic!("missing attribute-base assignment: {assignment}"))
    });
    for guard in [
        "vertex_indices.x > (0xFFFFFFFFu - mesh.vertices_attribute_offset) / stride",
        "vertex_indices.y > (0xFFFFFFFFu - mesh.vertices_attribute_offset) / stride",
        "vertex_indices.z > (0xFFFFFFFFu - mesh.vertices_attribute_offset) / stride",
    ] {
        let guard_position = validation_block
            .find(guard)
            .unwrap_or_else(|| panic!("missing vertex-index overflow guard: {guard}"));
        for attribute_base in attribute_bases {
            assert!(
                guard_position < attribute_base,
                "vertex-index overflow guard must precede every attribute-base computation: {guard}"
            );
        }
    }
    assert!(source.contains("debug_index + 6u < max_index"));
    assert!(!source.contains(
        "transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w"
    ));
    assert!(source.contains("transform.bb_max_scale_z.w"));
}

#[test]
fn debug_shader_stays_within_portable_storage_budget() {
    let source = fs::read_to_string(shader_root().join("debug.wgsl")).unwrap();
    assert!(source.matches("var<storage").count() <= 8);
}

#[test]
fn active_shader_sources_do_not_use_default_resolution_for_indexing() {
    for shader in [
        "debug.wgsl",
        "compute_wavefront_surface.wgsl",
        "compute_wavefront_transparency.wgsl",
        "compute_wavefront_direct_candidates.wgsl",
        "compute_wavefront_visibility.wgsl",
        "compute_wavefront_ao_resolve.wgsl",
        "compute_wavefront_composite.wgsl",
    ] {
        let source = fs::read_to_string(shader_root().join(shader)).unwrap();
        assert!(
            !source.contains("DEFAULT_WIDTH"),
            "{shader} uses DEFAULT_WIDTH"
        );
        assert!(
            !source.contains("DEFAULT_HEIGHT"),
            "{shader} uses DEFAULT_HEIGHT"
        );
    }
}
