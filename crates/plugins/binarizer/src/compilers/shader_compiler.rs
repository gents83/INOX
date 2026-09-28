#![allow(dead_code)]

use std::{
    collections::HashSet,
    env,
    fs::create_dir_all,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{clean_unused_definitions, need_to_binarize, send_reloaded_event, ExtensionHandler};
use inox_filesystem::{convert_from_local_path, delete_file};
use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_platform::PlatformType;
use inox_render::{
    platform::shader_preprocessor_defs, read_spirv_from_bytes, ShaderData, SHADER_EXTENSION,
};
use inox_resources::SharedDataRc;
use inox_serialize::{SerializationType, SerializeFile};
use inox_uid::generate_random_uid;
use regex::Regex;

const DEBUG_SHADER_GENERATED_CODE: bool = false;

const SHADERS_FOLDER_NAME: &str = "shaders";

const PREPROCESSED_SHADER_EXTENSION: &str = "preprocessed_shader";
const WGSL_EXTENSION: &str = "wgsl";
const SPV_EXTENSION: &str = "spv";
const VERTEX_SHADER_EXTENSION: &str = "vert";
const FRAGMENT_SHADER_EXTENSION: &str = "frag";
const GEOMETRY_SHADER_EXTENSION: &str = "geom";

/// The portable WebGPU profile guarantees only eight storage buffers per stage.
pub const MAX_PORTABLE_STORAGE_BUFFERS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageBinding {
    pub group: u32,
    pub binding: u32,
    pub name: String,
    pub stage: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageBindingReport {
    pub stage: &'static str,
    pub bindings: Vec<StorageBinding>,
}

impl StorageBindingReport {
    pub fn count(&self) -> usize {
        self.bindings.len()
    }
}

struct WgslExtensionRule {
    feature_define: &'static str,
    source_pattern: &'static str,
    directive: &'static str,
}

const WGSL_EXTENSION_RULES: &[WgslExtensionRule] = &[
    WgslExtensionRule {
        feature_define: "FEATURES_TEXTURE_BINDING_ARRAY",
        source_pattern: "binding_array<",
        directive: "enable wgpu_binding_array;",
    },
    WgslExtensionRule {
        feature_define: "FEATURES_PRIMITIVE_INDEX",
        source_pattern: "@builtin(primitive_index)",
        directive: "enable primitive_index;",
    },
];

pub struct ShaderCompiler<const PLATFORM_TYPE: PlatformType> {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
    glsl_compiler: PathBuf,
    glsl_validator: PathBuf,
    spirv_validator: PathBuf,
}

impl<const PLATFORM_TYPE: PlatformType> ShaderCompiler<PLATFORM_TYPE> {
    pub fn new(
        shared_data: SharedDataRc,
        message_hub: MessageHubRc,
        data_raw_folder: &Path,
        data_folder: &Path,
    ) -> Self {
        let mut vulkan_sdk_path = PathBuf::new();
        if let Ok(vulkan_path) = env::var("VULKAN_SDK") {
            vulkan_sdk_path = PathBuf::from(vulkan_path.as_str());
        }
        let shader_raw_folder: PathBuf = data_raw_folder
            .canonicalize()
            .unwrap()
            .join(SHADERS_FOLDER_NAME);
        let shader_data_folder: PathBuf = data_folder
            .canonicalize()
            .unwrap()
            .join(SHADERS_FOLDER_NAME);
        debug_assert!(shader_raw_folder.exists());
        if !shader_data_folder.exists() {
            let result = create_dir_all(shader_data_folder);
            debug_assert!(result.is_ok());
        }
        let bin_folder = if cfg!(windows) { "Bin" } else { "bin" };
        let glslc_name = if cfg!(windows) { "glslc.exe" } else { "glslc" };
        let glslang_name = if cfg!(windows) {
            "glslangValidator.exe"
        } else {
            "glslangValidator"
        };
        let spirv_val_name = if cfg!(windows) {
            "spirv-val.exe"
        } else {
            "spirv-val"
        };

        Self {
            message_hub,
            shared_data,
            data_raw_folder: data_raw_folder.to_path_buf(),
            data_folder: data_folder.to_path_buf(),
            glsl_compiler: vulkan_sdk_path.join(bin_folder).join(glslc_name),
            glsl_validator: vulkan_sdk_path.join(bin_folder).join(glslang_name),
            spirv_validator: vulkan_sdk_path.join(bin_folder).join(spirv_val_name),
        }
    }

    fn compile_assembly(&self, path: &Path) -> bool {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{extension}");
        let destination_ext = format!(
            "_{}_{}.{}_assembly",
            generate_random_uid(),
            extension,
            SHADER_EXTENSION
        );
        let mut from_source_to_temp = path.to_str().unwrap().to_string();
        from_source_to_temp = from_source_to_temp.replace(
            self.data_raw_folder
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            self.data_folder.canonicalize().unwrap().to_str().unwrap(),
        );
        from_source_to_temp =
            from_source_to_temp.replace(source_ext.as_str(), destination_ext.as_str());

        match Command::new(self.glsl_compiler.to_str().unwrap())
            .args([
                "--target-env=vulkan",
                path.to_str().unwrap(),
                "-o",
                from_source_to_temp.as_str(),
            ])
            .status()
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }
    fn convert_in_spirv(&self, path: &Path) -> bool {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{extension}");
        let temp_ext = format!("_{}_{}.{}", generate_random_uid(), extension, SPV_EXTENSION);
        let destination_ext = format!(".{SHADER_EXTENSION}");
        let mut from_source_to_temp = path.to_str().unwrap().to_string();
        from_source_to_temp = from_source_to_temp.replace(source_ext.as_str(), temp_ext.as_str());
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            self.data_raw_folder
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            self.data_folder.canonicalize().unwrap().to_str().unwrap(),
        );
        from_source_to_compiled =
            from_source_to_compiled.replace(source_ext.as_str(), destination_ext.as_str());
        let temp_path = PathBuf::from(from_source_to_temp);
        let new_path = PathBuf::from(from_source_to_compiled);
        if !need_to_binarize(path, new_path.as_path()) {
            return true;
        }
        debug_log!("Serializing {:?}", path);
        let validator_status = Command::new(self.glsl_validator.to_str().unwrap())
            .args([
                "-Os",
                "--quiet",
                "-w",
                "-t",
                "-g0",
                "-V",
                path.to_str().unwrap(),
                "-o",
                temp_path.to_str().unwrap(),
            ])
            .status();
        let validator_ok = validator_status
            .map(|status| status.success())
            .unwrap_or(false);
        if !validator_ok {
            delete_file(temp_path);
            return false;
        }
        let spirv_status = Command::new(self.spirv_validator.to_str().unwrap())
            .arg(temp_path.to_str().unwrap())
            .status();
        let spirv_ok = spirv_status.map(|status| status.success()).unwrap_or(false);
        if !spirv_ok || !temp_path.exists() {
            delete_file(temp_path);
            return false;
        }
        let mut file = match std::fs::File::open(temp_path.as_path()) {
            Ok(file) => file,
            Err(_) => {
                delete_file(temp_path);
                return false;
            }
        };
        let spirv_code = read_spirv_from_bytes(&mut file);
        let shader_data = ShaderData {
            spirv_code,
            ..Default::default()
        };
        if !new_path.exists() {
            let result = create_dir_all(new_path.parent().unwrap());
            if result.is_err() {
                delete_file(temp_path);
                return false;
            }
        }
        shader_data.save_to_file(new_path.as_path(), SerializationType::Binary);
        send_reloaded_event(&self.message_hub, new_path.as_path());
        delete_file(temp_path);
        true
    }

    /// Apply the exact preprocessing and binding-validation input used by the
    /// production binarizer. Tests and native smoke must call this instead of
    /// maintaining a second include expander.
    pub fn preprocess_wgsl_source(path: &Path, source: String) -> Result<String, String> {
        let source = Self::preprocess_code(path, source);
        let source = Self::add_required_wgsl_extensions(
            source,
            &shader_preprocessor_defs::<PLATFORM_TYPE>(),
        );
        let source = Self::remove_comments(&source);
        let source = clean_unused_definitions(&source)
            .map_err(|error| format!("unable to remove unused WGSL definitions: {error:?}"))?;
        Ok(Self::add_required_wgsl_extensions(
            source,
            &shader_preprocessor_defs::<PLATFORM_TYPE>(),
        ))
    }

    pub fn preprocess_shader(&self, local_path: &Path) {
        let path = convert_from_local_path(self.data_raw_folder.as_path(), local_path);
        debug_log!("Preproces shader {:?}", path);

        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{extension}");
        let destination_ext = format!(".{PREPROCESSED_SHADER_EXTENSION}");
        let mut from_source_to_preprocessed = path.to_str().unwrap().to_string();
        from_source_to_preprocessed =
            from_source_to_preprocessed.replace(source_ext.as_str(), destination_ext.as_str());
        let new_path = PathBuf::from(from_source_to_preprocessed);

        let mut file = std::fs::File::open(path.to_str().unwrap()).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let shader_code = String::from_utf8(data).unwrap();
        let preprocessed_code = Self::preprocess_wgsl_source(&path, shader_code)
            .expect("production WGSL preprocessing must produce valid source");

        debug_log!("validating shader {:?}", path);
        if self.validate_shader(&preprocessed_code, &path, &new_path) {
            debug_log!("and it's validated as well");
            std::fs::write(&new_path, &preprocessed_code).unwrap();
        }
    }

    pub fn extract_storage_bindings(
        source: &str,
        path: &Path,
    ) -> Result<StorageBindingReport, String> {
        Self::extract_storage_bindings_with_limit(source, path, MAX_PORTABLE_STORAGE_BUFFERS)
    }

    fn is_portable_production_shader(path: &Path) -> bool {
        let shader_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        shader_name.starts_with("compute_wavefront_")
            || matches!(
                shader_name,
                "visibility" | "finalize" | "blit" | "wireframe"
            )
    }

    fn extract_storage_bindings_for_shader(
        source: &str,
        path: &Path,
    ) -> Result<StorageBindingReport, String> {
        let storage_limit = if Self::is_portable_production_shader(path) {
            MAX_PORTABLE_STORAGE_BUFFERS
        } else {
            usize::MAX
        };
        Self::extract_storage_bindings_with_limit(source, path, storage_limit)
    }

    fn extract_storage_bindings_with_limit(
        source: &str,
        path: &Path,
        storage_limit: usize,
    ) -> Result<StorageBindingReport, String> {
        // Match an entire declaration, rather than one source line. WGSL permits
        // attributes and the `var<storage>` declaration to be formatted across
        // arbitrary lines. Capturing the complete attribute span also prevents
        // attributes from a preceding non-storage declaration from leaking in.
        let declaration_regex = Regex::new(
            r"(?s)(?P<attributes>(?:@[A-Za-z_]\w*(?:\s*\([^)]*\))?\s*)*)var\s*<\s*storage(?:\s*,[^>]*)?>\s*(?P<name>[A-Za-z_]\w*)",
        )
        .unwrap();
        let group_regex = Regex::new(r"@group\s*\(\s*(\d+)\s*\)").unwrap();
        let binding_regex = Regex::new(r"@binding\s*\(\s*(\d+)\s*\)").unwrap();
        let stage = if source.contains("@compute") {
            "compute"
        } else if source.contains("@vertex") || source.contains("@fragment") {
            "graphics"
        } else {
            "unknown"
        };
        let mut seen = HashSet::new();
        let mut bindings = Vec::new();

        for declaration in declaration_regex.captures_iter(source) {
            let attributes = &declaration["attributes"];
            let name = &declaration["name"];
            let line_number = source[..declaration.get(0).unwrap().start()]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            let Some(group) = group_regex
                .captures(attributes)
                .and_then(|value| value[1].parse::<u32>().ok())
            else {
                return Err(format!(
                    "shader {path:?} stage {stage} storage binding for {name} is missing @group at line {line_number}"
                ));
            };
            let Some(binding) = binding_regex
                .captures(attributes)
                .and_then(|value| value[1].parse::<u32>().ok())
            else {
                return Err(format!(
                    "shader {path:?} stage {stage} storage binding {name} is missing @binding at line {line_number}"
                ));
            };
            if !seen.insert((group, binding)) {
                return Err(format!(
                    "shader {path:?} stage {stage} has duplicate storage binding @group({group}) @binding({binding})"
                ));
            }
            bindings.push(StorageBinding {
                group,
                binding,
                name: name.to_string(),
                stage,
            });
        }

        if bindings.len() > storage_limit {
            return Err(format!(
                "shader {path:?} stage {stage} declares {} storage bindings; portable limit is {}",
                bindings.len(),
                storage_limit
            ));
        }
        Ok(StorageBindingReport { stage, bindings })
    }

    pub fn validate_preprocessed_wgsl(source: &str, path: &Path) -> Result<(), String> {
        let report = Self::extract_storage_bindings_for_shader(source, path)?;
        naga::front::wgsl::parse_str(source)
            .map_err(|error| format!("shader {path:?} WGSL parse failed: {error}"))
            .and_then(|module| {
                naga::valid::Validator::new(
                    naga::valid::ValidationFlags::default(),
                    naga::valid::Capabilities::all(),
                )
                .validate(&module)
                .map(|_| ())
                .map_err(|error| format!("shader {path:?} WGSL validation failed: {error:?}"))
            })?;
        debug_assert!(
            !Self::is_portable_production_shader(path)
                || report.count() <= MAX_PORTABLE_STORAGE_BUFFERS
        );
        Ok(())
    }

    fn validate_shader(&self, preprocessed_code: &str, path: &Path, new_path: &Path) -> bool {
        let mut validated = false;

        if let Err(error) = Self::extract_storage_bindings_for_shader(preprocessed_code, path) {
            eprintln!("Unable to validate shader {path:?}: {error}");
            return false;
        }
        let result = naga::front::wgsl::parse_str(preprocessed_code);
        match result {
            Ok(module) => {
                match naga::valid::Validator::new(
                    naga::valid::ValidationFlags::default(),
                    naga::valid::Capabilities::all(),
                )
                .validate(&module)
                {
                    Ok(info) => {
                        if DEBUG_SHADER_GENERATED_CODE {
                            use naga::back::spv;
                            use rspirv::binary::Disassemble;

                            let mut flags = spv::WriterFlags::LABEL_VARYINGS;
                            flags.set(spv::WriterFlags::DEBUG, true);

                            let options = spv::Options {
                                lang_version: (1, 1),
                                flags,
                                ..spv::Options::default()
                            };

                            module.entry_points.iter().for_each(|ep| {
                                let pipeline_options = spv::PipelineOptions {
                                    entry_point: ep.name.clone(),
                                    shader_stage: ep.stage,
                                };
                                let spv = spv::write_vec(
                                    &module,
                                    &info,
                                    &options,
                                    Some(&pipeline_options),
                                )
                                .unwrap();
                                let dis = rspirv::dr::load_words(spv)
                                    .expect("Produced invalid SPIR-V")
                                    .disassemble();
                                let spv_path =
                                    new_path.parent().unwrap().parent().unwrap().join("spv");
                                std::fs::create_dir_all(spv_path.as_path()).ok();
                                let spv_path = spv_path.join(format!(
                                    "{}_{}.spv",
                                    new_path.file_stem().unwrap().to_str().unwrap(),
                                    ep.name
                                ));
                                std::fs::write(spv_path.as_path(), dis).unwrap();
                            });
                        }

                        validated = true;
                    }
                    Err(e) => {
                        println!(
                            "Unable to compile shader {path:?} with error: \n{}",
                            e.emit_to_string(preprocessed_code),
                        );
                    }
                };
            }
            Err(ref e) => {
                println!(
                    "Unable to compile shader {path:?} with error: \n{}\n{}",
                    e.message(),
                    e.emit_to_string(preprocessed_code),
                );
            }
        }

        validated
    }

    fn create_wgsl_shader_data(&self, path: &Path) {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{extension}");
        let destination_ext = format!(".{SHADER_EXTENSION}");
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            self.data_raw_folder
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            self.data_folder.canonicalize().unwrap().to_str().unwrap(),
        );
        from_source_to_compiled =
            from_source_to_compiled.replace(source_ext.as_str(), destination_ext.as_str());
        let new_path = PathBuf::from(from_source_to_compiled);

        if need_to_binarize(path, new_path.as_path()) {
            debug_log!("Serializing {:?}", path);
            let mut file = std::fs::File::open(path.to_str().unwrap()).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            let shader_code = String::from_utf8(data).unwrap();
            let preprocessed_code = Self::preprocess_code(path, shader_code);
            let mut preprocessed_code = Self::add_required_wgsl_extensions(
                preprocessed_code,
                &shader_preprocessor_defs::<PLATFORM_TYPE>(),
            );
            preprocessed_code = Self::remove_comments(&preprocessed_code);

            let result = clean_unused_definitions(&preprocessed_code);
            match result {
                Ok(code) => {
                    preprocessed_code = Self::add_required_wgsl_extensions(
                        code,
                        &shader_preprocessor_defs::<PLATFORM_TYPE>(),
                    );
                }
                Err(e) => {
                    println!("Unable to optimize shader {path:?} with error: \n{}", e,);
                }
            }

            if self.validate_shader(&preprocessed_code, path, &new_path) {
                let shader_data = ShaderData {
                    wgsl_code: preprocessed_code,
                    ..Default::default()
                };
                if !new_path.exists() {
                    let result = create_dir_all(new_path.parent().unwrap());
                    debug_assert!(result.is_ok());
                }
                shader_data.save_to_file(new_path.as_path(), SerializationType::Binary);
                send_reloaded_event(&self.message_hub, new_path.as_path());
            }
        }
    }

    pub fn validate_pipeline_descriptors(data_raw_folder: &Path) -> Result<(), String> {
        let pipeline_folder = data_raw_folder.join("pipelines");
        let mut errors = Vec::new();
        let entries = std::fs::read_dir(&pipeline_folder).map_err(|error| {
            format!("unable to read pipeline folder {pipeline_folder:?}: {error}")
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("compute_pipeline")
                && path.extension().and_then(|extension| extension.to_str())
                    != Some("render_pipeline")
            {
                continue;
            }
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    errors.push(format!("pipeline {path:?} cannot be read: {error}"));
                    continue;
                }
            };
            let descriptor: serde_json::Value = match serde_json::from_str(&source) {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    errors.push(format!("pipeline {path:?} is invalid JSON: {error}"));
                    continue;
                }
            };
            for field in ["shader", "vertex_shader", "fragment_shader"] {
                let Some(shader) = descriptor.get(field).and_then(serde_json::Value::as_str) else {
                    continue;
                };
                let shader_path = data_raw_folder.join(shader).with_extension(WGSL_EXTENSION);
                if !shader_path.is_file() {
                    errors.push(format!(
                        "pipeline {path:?} field {field} references missing shader {shader_path:?}"
                    ));
                    continue;
                }
                let shader_source = match std::fs::read_to_string(&shader_path) {
                    Ok(source) => source,
                    Err(error) => {
                        errors.push(format!("shader {shader_path:?} cannot be read: {error}"));
                        continue;
                    }
                };
                let preprocessed = Self::preprocess_wgsl_source(&shader_path, shader_source)
                    .map_err(|error| format!("pipeline preprocessing failed: {error}"));
                let result = preprocessed.and_then(|source| {
                    Self::extract_storage_bindings_for_shader(&source, &shader_path)
                });
                if let Err(error) = result {
                    errors.push(format!("pipeline {path:?} field {field}: {error}"));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\\n"))
        }
    }

    pub fn validate_wgsl_file(path: &Path) -> bool {
        let Ok(source) = std::fs::read_to_string(path) else {
            eprintln!("Unable to read WGSL source {path:?}");
            return false;
        };
        let source = match Self::preprocess_wgsl_source(path, source) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("Unable to preprocess WGSL source {path:?}: {error}");
                return false;
            }
        };
        if let Err(error) = Self::extract_storage_bindings_for_shader(&source, path) {
            eprintln!("Unable to validate WGSL source {path:?}: {error}");
            return false;
        }
        let Ok(module) = naga::front::wgsl::parse_str(&source) else {
            eprintln!("Unable to parse WGSL source {path:?}:\n{source}");
            return false;
        };
        match naga::valid::Validator::new(
            naga::valid::ValidationFlags::default(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        {
            Ok(_) => true,
            Err(error) => {
                eprintln!("Unable to validate WGSL source {path:?}: {error:?}");
                false
            }
        }
    }

    pub fn add_required_wgsl_extensions(mut code: String, available_defs: &[String]) -> String {
        let mut directives = String::new();

        for rule in WGSL_EXTENSION_RULES {
            let feature_enabled = available_defs
                .iter()
                .any(|available_def| available_def == rule.feature_define);
            let code_without_comments = Self::remove_comments(&code);
            let extension_used = match rule.source_pattern {
                "binding_array<" => code_without_comments.contains("binding_array"),
                "@builtin(primitive_index)" => {
                    code_without_comments.contains("@builtin")
                        && code_without_comments.contains("primitive_index")
                }
                _ => code_without_comments.contains(rule.source_pattern),
            };
            let directive_already_present = code_without_comments.lines().any(|line| {
                line.trim()
                    .split_once("//")
                    .map_or(line.trim(), |(directive, _)| directive.trim())
                    == rule.directive
            });

            if feature_enabled && extension_used && !directive_already_present {
                directives.push_str(rule.directive);
                directives.push('\n');
            }
        }

        if !directives.is_empty() {
            directives.push_str(&code);
            code = directives;
        }

        code
    }

    fn remove_comments(code: &str) -> String {
        let single_line_comment = Regex::new(r"//.*").unwrap();
        let multi_line_comment = Regex::new(r"/\*.*?\*/").unwrap();
        let code = single_line_comment.replace_all(code, "");
        let code = multi_line_comment.replace_all(&code, "");

        code.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn preprocess_code(path: &Path, code: String) -> String {
        let available_defs = shader_preprocessor_defs::<PLATFORM_TYPE>();
        let mut string = String::new();
        let ifdef_regex = Regex::new(r"^\s*#\s*ifdef\s*([\w|\d|_]+)").unwrap();
        let else_regex = Regex::new(r"^\s*#\s*else").unwrap();
        let endif_regex = Regex::new(r"^\s*#\s*endif").unwrap();
        let import_regex = Regex::new(r"^\s*#\s*import").unwrap();
        let mut should_skip = false;
        for line in code.lines() {
            if let Some(cap) = ifdef_regex.captures(line) {
                let def = cap.get(1).unwrap().as_str().to_string();
                should_skip = !available_defs.contains(&def);
                continue;
            } else if else_regex.is_match(line) {
                should_skip = !should_skip;
                continue;
            } else if endif_regex.is_match(line) {
                should_skip = false;
                continue;
            } else if import_regex.is_match(line) {
                if let Some(first) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        let import_path = line[first + 1..end].to_string();
                        let import_path = PathBuf::from(path.parent().unwrap()).join(import_path);
                        let import_path = import_path.canonicalize().unwrap();
                        let shader_code = std::fs::read_to_string(&import_path).unwrap();
                        let import_code = Self::preprocess_code(&import_path, shader_code);
                        if !import_code.trim().is_empty() {
                            string.push_str(&import_code);
                        }
                        continue;
                    }
                }
            }
            if !should_skip && !line.trim().is_empty() {
                string.push_str(line);
                string.push('\n');
            }
        }
        string
    }
}

impl<const PLATFORM_TYPE: PlatformType> ExtensionHandler for ShaderCompiler<PLATFORM_TYPE> {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap().to_string().as_str() {
                WGSL_EXTENSION => {
                    self.create_wgsl_shader_data(path);
                }
                VERTEX_SHADER_EXTENSION => {
                    let result = self.convert_in_spirv(path);
                    if !result {
                        eprintln!("Failed to process VERTEX shader {}", path.to_str().unwrap());
                    }
                }
                GEOMETRY_SHADER_EXTENSION => {
                    let result = self.convert_in_spirv(path);
                    if !result {
                        eprintln!(
                            "Failed to process GEOMETRY shader {}",
                            path.to_str().unwrap(),
                        );
                    }
                }
                FRAGMENT_SHADER_EXTENSION => {
                    let result = self.convert_in_spirv(path);
                    if !result {
                        eprintln!(
                            "Failed to process FRAGMENT shader {}",
                            path.to_str().unwrap(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
