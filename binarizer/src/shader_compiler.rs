use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::ExtensionHandler;
use nrg_messenger::{Message, MessengerRw};
use nrg_resources::ResourceEvent;

const SOURCE_FOLDER_NAME: &str = "source";
const COMPILED_FOLDER_NAME: &str = "compiled";
const TEMP_FOLDER_NAME: &str = "temp";

const SHADER_EXTENSION: &str = "spv";
const VERTEX_SHADER_EXTENSION: &str = "vert";
const FRAGMENT_SHADER_EXTENSION: &str = "frag";
const GEOMETRY_SHADER_EXTENSION: &str = "geom";

pub struct ShaderCompiler {
    global_messenger: MessengerRw,
    glsl_compiler: PathBuf,
    glsl_validator: PathBuf,
    spirv_validator: PathBuf,
}

impl ShaderCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        let mut vulkan_sdk_path = PathBuf::new();
        if let Ok(vulkan_path) = env::var("VULKAN_SDK") {
            vulkan_sdk_path = PathBuf::from(vulkan_path.as_str());
        }
        Self {
            global_messenger,
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
        from_source_to_temp = from_source_to_temp.replace(SOURCE_FOLDER_NAME, TEMP_FOLDER_NAME);
        from_source_to_temp =
            from_source_to_temp.replace(source_ext.as_str(), destination_ext.as_str());

        Command::new(self.glsl_compiler.to_str().unwrap())
            .args(&[path.to_str().unwrap(), "-o", from_source_to_temp.as_str()])
            .spawn()
            .is_ok()
    }
    fn convert_in_spirv(&self, path: &Path) -> bool {
        let extension = path.extension().unwrap().to_str().unwrap();
        let source_ext = format!(".{}", extension);
        let destination_ext = format!("_{}.{}", extension, SHADER_EXTENSION);
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled =
            from_source_to_compiled.replace(SOURCE_FOLDER_NAME, COMPILED_FOLDER_NAME);
        from_source_to_compiled =
            from_source_to_compiled.replace(source_ext.as_str(), destination_ext.as_str());

        let converted = Command::new(self.glsl_validator.to_str().unwrap())
            .args(&[
                "-o",
                from_source_to_compiled.as_str(),
                "-V",
                path.to_str().unwrap(),
            ])
            .spawn()
            .is_ok();

        if converted {
            let result = Command::new(self.spirv_validator.to_str().unwrap())
                .arg(from_source_to_compiled.as_str())
                .spawn()
                .is_ok();
            if result {
                let dispatcher = self.global_messenger.read().unwrap().get_dispatcher();
                dispatcher
                    .write()
                    .unwrap()
                    .send(ResourceEvent::Reload(PathBuf::from(from_source_to_compiled)).as_boxed())
                    .ok();
            }
        }
        converted
    }
}

impl ExtensionHandler for ShaderCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap().to_string().as_str() {
                VERTEX_SHADER_EXTENSION => {
                    let result = self.compile_assembly(path) && self.convert_in_spirv(path);
                    if !result {
                        eprintln!("Failed to process VERTEX shader {}", path.to_str().unwrap());
                    }
                }
                GEOMETRY_SHADER_EXTENSION => {
                    let result = self.compile_assembly(path) && self.convert_in_spirv(path);
                    if !result {
                        eprintln!(
                            "Failed to process GEOMETRY shader {}",
                            path.to_str().unwrap()
                        );
                    }
                }
                FRAGMENT_SHADER_EXTENSION => {
                    let result = self.compile_assembly(path) && self.convert_in_spirv(path);
                    if !result {
                        eprintln!(
                            "Failed to process FRAGMENT shader {}",
                            path.to_str().unwrap()
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
