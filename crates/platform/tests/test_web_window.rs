use inox_commands::CommandParser;
use inox_platform::{
    canvas_mouse_button, map_canvas_mouse_coordinates, window_size_from_parser, MouseButton,
    MouseState,
};

#[test]
fn maps_css_coordinates_to_device_pixels() {
    let (x, y, normalized_x, normalized_y) =
        map_canvas_mouse_coordinates(150.0, 75.0, 50.0, 25.0, 200.0, 100.0, 800, 400);
    assert_eq!((x, y), (400.0, 200.0));
    assert_eq!((normalized_x, normalized_y), (0.5, 0.5));
}

#[test]
fn clamps_coordinates_and_prevents_zero_dimension_division() {
    let (x, y, normalized_x, normalized_y) =
        map_canvas_mouse_coordinates(-10.0, f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0, 0);
    assert_eq!((x, y), (0.0, 0.0));
    assert_eq!((normalized_x, normalized_y), (0.0, 0.0));
}

#[test]
fn movement_does_not_report_a_button_press() {
    assert_eq!(canvas_mouse_button(MouseState::Move, 0), MouseButton::None);
    assert_eq!(canvas_mouse_button(MouseState::Down, 0), MouseButton::Left);
    assert_eq!(canvas_mouse_button(MouseState::Up, 2), MouseButton::Right);
}

#[test]
fn command_line_window_size_is_resolved_before_window_creation() {
    let parser = CommandParser::from_string("-window_size 320 240");
    assert_eq!(window_size_from_parser(&parser, 1920, 1080), (320, 240));
}

#[test]
fn command_line_window_size_clamps_zero_and_preserves_defaults_when_missing() {
    let zero = CommandParser::from_string("-window_size 0 0");
    assert_eq!(window_size_from_parser(&zero, 1920, 1080), (1, 1));

    let missing = CommandParser::from_string("-plugin inox_viewer");
    assert_eq!(window_size_from_parser(&missing, 1920, 1080), (1920, 1080));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn browser_smoke_requires_engine_and_canvas_markers() {
    let smoke = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/smoke.html"),
    )
    .expect("browser smoke page must be available");
    for marker in [
        "data-inox-smoke",
        "status.dataset.inoxSmoke = 'passed'",
        "data-inox-error-count=\"0\"",
        "inoxGraphOrder",
        "inoxDimensions",
        "inoxQueueHeaders",
        "gpu-readback-validated",
        "inoxOutput",
        "finite-nonzero-validated",
        "inoxCanvas",
        "owned",
        "inoxInstance",
        "inoxAdapterRequested",
        "inoxAdapter",
        "inoxDeviceRequested",
        "inoxSkip",
        "instanceReady",
        "adapterRequested",
        "adapterReady",
        "deviceRequested",
        "deviceFailure",
        "inoxDeviceFailure",
        "webgpu-unavailable",
        "adapter-timeout",
        "WebGPU adapter request did not resolve before deadline",
        "WebGPU initialization did not resolve before deadline",
        "__inox_smoke_error",
        "asyncFailure",
        "console.error",
    ] {
        assert!(smoke.contains(marker), "browser smoke is missing {marker}");
    }
    assert!(
        smoke.contains("if (instanceReady && adapterRequested && !adapterReady && !deviceRequested && !deviceReady")
            && smoke.contains("&& !deviceFailure && smokeErrors.length === 0 && !asyncFailure"),
        "adapter-timeout skip must be bounded to a requested, unresolved adapter without init errors"
    );
    assert!(!smoke.contains("gpu-visible"));
    assert!(!smoke.contains("finite-nonzero-contract"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn browser_capture_preserves_dom_text_for_script_smoke_diagnostics() {
    let script = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/validate_wavefront_pipeline.ps1"),
    )
    .expect("wavefront validation script must be available");
    assert!(
        script.contains("$browserOutput = (& $browser @arguments 2>&1 | Out-String)"),
        "browser output must be captured as one string through Out-String"
    );
    assert!(
        script.contains("$ErrorActionPreference = \"Continue\""),
        "Chrome stderr diagnostics must not abort DOM capture under Stop policy"
    );
    assert!(
        !script.contains("$browserOutput = @(& $browser @arguments 2>&1)"),
        "array capture can hide native Chrome DOM output"
    );
    assert!(
        script.contains("$domText = $browserOutput")
            && script.contains("data-inox-smoke=\"skipped\"")
            && script.contains("adapter-timeout")
            && script.contains("data-inox-smoke=\"passed\"")
            && script.contains("data-inox-error-count=\"0\""),
        "browser smoke must inspect captured DOM text and its status/error markers"
    );
    for marker in [
        "BrowserBackend",
        "BrowserBackend -eq \"swiftshader\"",
        "SwiftShader fallback",
        "--disable-gpu",
        "data-inox-instance=\"ready\"",
        "data-inox-adapter-requested=\"observed\"",
        "data-inox-adapter=\"pending\"",
        "data-inox-device-requested=\"pending\"",
        "data-inox-device=\"pending\"",
        "data-inox-device-failure=\"none\"",
        "$statusTagMatch = [regex]::Match($domText",
        "$statusTag -match 'data-inox-error-count=\"0\"'",
        "retrying once with --disable-gpu",
    ] {
        assert!(
            script.contains(marker),
            "browser smoke fallback contract is missing {marker}"
        );
    }
}

#[cfg(target_arch = "wasm32")]
mod browser_contract {
    use super::*;

    #[test]
    fn device_pixel_dimensions_are_nonzero() {
        let (x, y, normalized_x, normalized_y) =
            map_canvas_mouse_coordinates(0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0, 0);
        assert_eq!((x, y), (0.0, 0.0));
        assert_eq!((normalized_x, normalized_y), (0.0, 0.0));
    }
}
