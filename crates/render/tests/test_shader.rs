use std::path::Path;

use inox_render::resources::shader::format_compilation_info;

#[test]
fn formats_shader_compilation_diagnostics_with_source_location() {
    let info = wgpu::CompilationInfo {
        messages: vec![wgpu::CompilationMessage {
            message: "expected ';'".to_string(),
            message_type: wgpu::CompilationMessageType::Error,
            location: Some(wgpu::SourceLocation {
                line_number: 42,
                line_position: 17,
                offset: 0,
                length: 1,
            }),
        }],
    };

    let formatted = format_compilation_info(Path::new("data/ui.wgsl"), &info);

    assert_eq!(
        formatted,
        "Shader compilation diagnostics for data/ui.wgsl:\nerror at line 42, column 17: expected ';'"
    );
}
