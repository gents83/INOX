// src/lib.rs

use naga::{
    back::wgsl::{Writer, WriterFlags},
    compact,
    front::wgsl::parse_str,
    valid::{Capabilities, ValidationFlags, Validator},
};

/// Removes comments and unused functions, types, and variables from a WGSL shader.
pub fn clean_unused_definitions(shader_source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Parse the WGSL shader source into a Naga module.
    let mut module = match parse_str(shader_source) {
        Ok(module) => module,
        Err(e) => {
            let error_string = e.emit_to_string_with_path(shader_source, "shader.wgsl");
            return Err(format!("Shader parsing failed:\n{error_string}").into());
        }
    };

    // 2. Use Naga's built-in `compact` function to remove all unused global items.
    // This is the correct, modern way to perform dead code elimination on globals.
    // We pass `KeepUnused::No` to ensure all unreferenced items are pruned.
    compact::compact(&mut module);

    // 4. Validate the pruned module to ensure it's still correct.
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let info = match validator.validate(&module) {
        Ok(info) => info,
        Err(e) => {
            // On validation failure, it's safer to just return the error without trying to write the module.
            return Err(e.into());
        }
    };

    // 5. Write the validated, pruned module back to a WGSL string.
    let mut final_wgsl = String::new();
    let mut writer = Writer::new(&mut final_wgsl, WriterFlags::empty());
    writer.write(&module, &info)?;

    Ok(final_wgsl)
}
