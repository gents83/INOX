use std::{
    env,
    fs::create_dir_all,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{need_to_binarize, send_reloaded_event, ExtensionHandler};
use inox_filesystem::delete_file;
use inox_graphics::{read_spirv_from_bytes, ShaderData, SHADER_EXTENSION};
use inox_messenger::MessageHubRc;
use inox_resources::{Data, SharedDataRc};
use inox_serialize::SerializeFile;

const SHADERS_FOLDER_NAME: &str = "shaders";

const WGSL_EXTENSION: &str = "wgsl";
const SPV_EXTENSION: &str = "spv";
const VERTEX_SHADER_EXTENSION: &str = "vert";
const FRAGMENT_SHADER_EXTENSION: &str = "frag";
const GEOMETRY_SHADER_EXTENSION: &str = "geom";

pub struct ShaderCompiler {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    glsl_compiler: PathBuf,
    glsl_validator: PathBuf,
    spirv_validator: PathBuf,
}

impl ShaderCompiler {
    pub fn new(shared_data: SharedDataRc, message_hub: MessageHubRc) -> Self {
        let mut vulkan_sdk_path = PathBuf::new();
        if let Ok(vulkan_path) = env::var("VULKAN_SDK") {
            vulkan_sdk_path = PathBuf::from(vulkan_path.as_str());
        }
        let shader_raw_folder: PathBuf = Data::data_raw_folder()
            .canonicalize()
            .unwrap()
            .join(SHADERS_FOLDER_NAME);
        let shader_data_folder: PathBuf = Data::data_folder()
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
            glsl_compiler: vulkan_sdk_path.join("Bin\\glslc.exe"),
            glsl_validator: vulkan_sdk_path.join("Bin\\glslangValidator.exe"),
            spirv_validator: vulkan_sdk_path.join("Bin\\spirv-val.exe"),
        }
    }

    fn compile_assembly(&self, path: &Path) -> bool {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{}", extension);
        let destination_ext = format!("_{}.{}_assembly", extension, SHADER_EXTENSION);
        let mut from_source_to_temp = path.to_str().unwrap().to_string();
        from_source_to_temp = from_source_to_temp.replace(
            Data::data_raw_folder()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            Data::data_folder()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        from_source_to_temp =
            from_source_to_temp.replace(source_ext.as_str(), destination_ext.as_str());

        Command::new(self.glsl_compiler.to_str().unwrap())
            .args(&[
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
        let source_ext = format!(".{}", extension);
        let temp_ext = format!("_{}.{}", extension, SPV_EXTENSION);
        let destination_ext = format!("_{}.{}", extension, SHADER_EXTENSION);
        let mut from_source_to_temp = path.to_str().unwrap().to_string();
        from_source_to_temp = from_source_to_temp.replace(source_ext.as_str(), temp_ext.as_str());
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            Data::data_raw_folder()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            Data::data_folder()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        from_source_to_compiled =
            from_source_to_compiled.replace(source_ext.as_str(), destination_ext.as_str());
        let temp_path = PathBuf::from(from_source_to_temp);
        let new_path = PathBuf::from(from_source_to_compiled);
        if need_to_binarize(path, new_path.as_path()) {
            if let Ok(mut command) = Command::new(self.glsl_validator.to_str().unwrap())
                .args(&[
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
                    shader_data
                        .save_to_file(new_path.as_path(), self.shared_data.serializable_registry());
                    send_reloaded_event(&self.message_hub, new_path.as_path());
                }
                delete_file(temp_path);
            }
        }
        true
    }
    fn create_wgsl_shader_data(&self, path: &Path) {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{}", extension);
        let destination_ext = format!("_{}.{}", extension, SHADER_EXTENSION);
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            Data::data_raw_folder()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            Data::data_folder()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        from_source_to_compiled =
            from_source_to_compiled.replace(source_ext.as_str(), destination_ext.as_str());
        let new_path = PathBuf::from(from_source_to_compiled);

        if need_to_binarize(path, new_path.as_path()) {
            let mut file = std::fs::File::open(path.to_str().unwrap()).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            let shader_data = ShaderData {
                wgsl_code: String::from_utf8(data).unwrap(),
                ..Default::default()
            };
            shader_data.save_to_file(new_path.as_path(), self.shared_data.serializable_registry());
            send_reloaded_event(&self.message_hub, new_path.as_path());
        }
    }
}

impl ExtensionHandler for ShaderCompiler {
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
