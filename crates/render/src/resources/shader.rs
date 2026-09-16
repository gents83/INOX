use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on;

use inox_messenger::MessageHubRc;

use inox_resources::{
    DataTypeResource, ResourceId, ResourceTrait, SerializableResource, SharedDataRc,
};
use inox_serialize::{read_from_file, SerializationType};
use wgpu::ShaderModule;

use crate::{RenderContext, ShaderData, SHADER_EXTENSION};

pub type ShaderId = ResourceId;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShaderType {
    Invalid,
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadError {
    pub path: PathBuf,
    pub stage: String,
    pub source_location: Option<String>,
    pub diagnostic: String,
    pub previous_resource_active: bool,
}

impl ReloadError {
    pub fn new(
        path: &Path,
        stage: impl Into<String>,
        diagnostic: impl Into<String>,
        previous_resource_active: bool,
    ) -> Self {
        Self {
            path: path.to_path_buf(),
            stage: stage.into(),
            source_location: None,
            diagnostic: diagnostic.into(),
            previous_resource_active,
        }
    }
}

pub type ShaderReloadError = ReloadError;

pub const SHADER_EXTENSION_SPV: &str = "spv";
pub const SHADER_EXTENSION_WGSL: &str = "wgsl";

pub const SHADER_ENTRY_POINT: &str = "main";
pub const VERTEX_SHADER_ENTRY_POINT: &str = "vs_main";
pub const FRAGMENT_SHADER_ENTRY_POINT: &str = "fs_main";

pub fn is_shader(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension == SHADER_EXTENSION_SPV || extension == SHADER_EXTENSION_WGSL
        })
}

pub fn read_spirv_from_bytes<Data: ::std::io::Read + ::std::io::Seek>(
    data: &mut Data,
) -> ::std::vec::Vec<u32> {
    let size = data.seek(::std::io::SeekFrom::End(0)).unwrap();
    if !size.is_multiple_of(4) {
        panic!("Input data length not divisible by 4");
    }
    if size > usize::MAX as u64 {
        panic!("Input data too long");
    }
    let words = (size / 4) as usize;
    let mut result = Vec::<u32>::with_capacity(words);
    data.rewind().unwrap();
    unsafe {
        data.read_exact(::std::slice::from_raw_parts_mut(
            result.as_mut_ptr() as *mut u8,
            words * 4,
        ))
        .unwrap();
        result.set_len(words);
    }
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() {
        if result[0] == MAGIC_NUMBER.swap_bytes() {
            for word in &mut result {
                *word = word.swap_bytes();
            }
        } else if result[0] != MAGIC_NUMBER {
            panic!("Input data is missing SPIR-V magic number");
        }
    } else {
        panic!("Input data is empty");
    }
    result
}
pub struct Shader {
    path: PathBuf,
    data: ShaderData,
    module: Option<ShaderModule>,
    last_reload_error: Option<ReloadError>,
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
            module: None,
            last_reload_error: self.last_reload_error.clone(),
        }
    }
}

impl ResourceTrait for Shader {
    fn invalidate(&mut self) -> &mut Self {
        self.module = None;
        self
    }
    fn is_initialized(&self) -> bool {
        self.module.is_some()
    }
}

impl SerializableResource for Shader {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        SHADER_EXTENSION
    }

    fn deserialize_data(path: &std::path::Path, f: Box<dyn FnMut(Self::DataType) + 'static>) {
        read_from_file::<Self::DataType>(path, SerializationType::Binary, f);
    }
}

impl DataTypeResource for Shader {
    type DataType = ShaderData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            data: ShaderData::default(),
            module: None,
            last_reload_error: None,
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut shader = Self::new(id, shared_data, message_hub);
        shader.data = data.clone();
        shader
    }
}

pub fn format_compilation_info(path: &Path, info: &wgpu::CompilationInfo) -> String {
    let mut diagnostics = format!("Shader compilation diagnostics for {}:", path.display());

    for message in &info.messages {
        let message_type = match message.message_type {
            wgpu::CompilationMessageType::Error => "error",
            wgpu::CompilationMessageType::Warning => "warning",
            wgpu::CompilationMessageType::Info => "info",
        };
        let location = message
            .location
            .map(|location| {
                format!(
                    " at line {}, column {}",
                    location.line_number, location.line_position
                )
            })
            .unwrap_or_default();
        diagnostics.push_str(&format!("\n{message_type}{location}: {}", message.message));
    }

    diagnostics
}

#[cfg(not(target_arch = "wasm32"))]
fn create_shader_module(
    device: &wgpu::Device,
    descriptor: wgpu::ShaderModuleDescriptor<'_>,
    path: &Path,
) -> Result<wgpu::ShaderModule, (String, Option<String>)> {
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let module = device.create_shader_module(descriptor);
    let compilation_info = block_on(module.get_compilation_info());
    let validation_error = block_on(error_scope.pop());

    let has_error_diagnostic = compilation_info
        .messages
        .iter()
        .any(|message| message.message_type == wgpu::CompilationMessageType::Error);
    if let Some(error) = validation_error {
        return Err((
            format!(
                "wgpu shader validation error: {error}\n{}",
                format_compilation_info(path, &compilation_info)
            ),
            compilation_info.messages.iter().find_map(|message| {
                message.location.map(|location| {
                    format!(
                        "line {}, column {}",
                        location.line_number, location.line_position
                    )
                })
            }),
        ));
    }
    if has_error_diagnostic {
        return Err((
            format_compilation_info(path, &compilation_info),
            compilation_info.messages.iter().find_map(|message| {
                message.location.map(|location| {
                    format!(
                        "line {}, column {}",
                        location.line_number, location.line_position
                    )
                })
            }),
        ));
    }

    Ok(module)
}

#[cfg(target_arch = "wasm32")]
fn create_shader_module(
    device: &wgpu::Device,
    descriptor: wgpu::ShaderModuleDescriptor<'_>,
    _path: &Path,
) -> Result<wgpu::ShaderModule, (String, Option<String>)> {
    Ok(device.create_shader_module(descriptor))
}

impl Shader {
    pub fn init(&mut self, context: &RenderContext) -> Result<(), ShaderReloadError> {
        if self.module.is_some() {
            return Ok(());
        }

        inox_profiler::scoped_profile!("shader::init({:?})", self.path);
        let shader_name = format!(
            "Shader {}",
            self.path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        );
        let result = if !self.data.spirv_code.is_empty() {
            create_shader_module(
                &context.webgpu.device,
                wgpu::ShaderModuleDescriptor {
                    label: Some(shader_name.as_str()),
                    source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(
                        self.data.spirv_code.as_slice(),
                    )),
                },
                &self.path,
            )
        } else if !self.data.wgsl_code.is_empty() {
            create_shader_module(
                &context.webgpu.device,
                wgpu::ShaderModuleDescriptor {
                    label: Some(shader_name.as_str()),
                    source: wgpu::ShaderSource::Wgsl(self.data.wgsl_code.clone().into()),
                },
                &self.path,
            )
        } else {
            Err(("shader source is empty".to_string(), None))
        };

        match result {
            Ok(module) => {
                self.module = Some(module);
                self.last_reload_error = None;
                Ok(())
            }
            Err((diagnostic, source_location)) => {
                let mut error =
                    ReloadError::new(&self.path, "shader", diagnostic, self.module.is_some());
                error.source_location = source_location;
                inox_log::debug_log!(
                    "Shader reload failed for {:?} (stage {}): {}",
                    error.path,
                    error.stage,
                    error.diagnostic
                );
                self.last_reload_error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn module(&self) -> &ShaderModule {
        self.module.as_ref().unwrap()
    }

    pub fn last_reload_error(&self) -> Option<&ReloadError> {
        self.last_reload_error.as_ref()
    }
}
