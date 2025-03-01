#![allow(dead_code)]

use std::{
    env,
    fs::create_dir_all,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{need_to_binarize, send_reloaded_event, ExtensionHandler};
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
        Self {
            message_hub,
            shared_data,
            data_raw_folder: data_raw_folder.to_path_buf(),
            data_folder: data_folder.to_path_buf(),
            glsl_compiler: vulkan_sdk_path.join("Bin\\glslc.exe"),
            glsl_validator: vulkan_sdk_path.join("Bin\\glslangValidator.exe"),
            spirv_validator: vulkan_sdk_path.join("Bin\\spirv-val.exe"),
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

        Command::new(self.glsl_compiler.to_str().unwrap())
            .args([
                "--target-env=vulkan",
                path.to_str().unwrap(),
                "-o",
                from_source_to_temp.as_str(),
            ])
            .spawn()
            .is_ok()
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
        if need_to_binarize(path, new_path.as_path()) {
            debug_log!("Serializing {:?}", path);
            if let Ok(mut command) = Command::new(self.glsl_validator.to_str().unwrap())
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
                .spawn()
            {
                if command.wait().is_ok()
                    && Command::new(self.spirv_validator.to_str().unwrap())
                        .arg(temp_path.to_str().unwrap())
                        .spawn()
                        .is_ok()
                {
                    let mut file = std::fs::File::open(temp_path.to_str().unwrap()).unwrap();
                    let spirv_code = read_spirv_from_bytes(&mut file);
                    let shader_data = ShaderData {
                        spirv_code,
                        ..Default::default()
                    };
                    shader_data.save_to_file(new_path.as_path(), SerializationType::Binary);
                    send_reloaded_event(&self.message_hub, new_path.as_path());
                }
                delete_file(temp_path);
            }
        }
        true
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
        let preprocessed_code = Self::preprocess_code(&path, shader_code);

        if self.validate_shader(&preprocessed_code, &path, &new_path) {
            debug_log!("into shader {:?}", new_path);
            std::fs::write(&new_path, preprocessed_code).unwrap();
        }
    }

    fn validate_shader(&self, preprocessed_code: &str, path: &Path, new_path: &Path) -> bool {
        let mut validated = false;

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

            if self.validate_shader(&preprocessed_code, path, &new_path) {
                let shader_data = ShaderData {
                    wgsl_code: preprocessed_code,
                    ..Default::default()
                };
                shader_data.save_to_file(new_path.as_path(), SerializationType::Binary);
                send_reloaded_event(&self.message_hub, new_path.as_path());
            }
        }
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
                        let shader_code = std::fs::read_to_string(import_path).unwrap();
                        let import_code = Self::preprocess_code(path, shader_code);
                        string.push_str(&import_code);
                        continue;
                    }
                }
            }
            if !should_skip && !line.is_empty() {
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
