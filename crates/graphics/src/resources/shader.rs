use std::path::{Path, PathBuf};

use inox_messenger::MessageHubRc;

use inox_resources::{
    DataTypeResource, ResourceId, ResourceTrait, SerializableResource, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};
use wgpu::ShaderModule;

use crate::{RenderContext, ShaderData, SHADER_EXTENSION};

pub type ShaderId = ResourceId;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ShaderType {
    Invalid,
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
}

pub const SHADER_EXTENSION_SPV: &str = "spv";
pub const SHADER_EXTENSION_WGSL: &str = "wgsl";

pub const SHADER_ENTRY_POINT: &str = "main";
pub const VERTEX_SHADER_ENTRY_POINT: &str = "vs_main";
pub const FRAGMENT_SHADER_ENTRY_POINT: &str = "fs_main";

pub fn is_shader(path: &Path) -> bool {
    path.extension().unwrap() == SHADER_EXTENSION_SPV
        || path.extension().unwrap() == SHADER_EXTENSION_WGSL
}

pub fn read_spirv_from_bytes<Data: ::std::io::Read + ::std::io::Seek>(
    data: &mut Data,
) -> ::std::vec::Vec<u32> {
    let size = data.seek(::std::io::SeekFrom::End(0)).unwrap();
    if size % 4 != 0 {
        panic!("Input data length not divisible by 4");
    }
    if size > usize::max_value() as u64 {
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
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
            module: None,
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

    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
}

impl DataTypeResource for Shader {
    type DataType = ShaderData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            data: ShaderData::default(),
            module: None,
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

impl Shader {
    pub fn init(&mut self, context: &RenderContext) -> bool {
        if self.module.is_none() {
            inox_profiler::scoped_profile!("shader::init({:?})", self.path);
            let shader_name = format!(
                "Shader {}",
                self.path
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            );
            if !self.data.spirv_code.is_empty() {
                let module =
                    context
                        .webgpu
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(shader_name.as_str()),
                            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(
                                self.data.spirv_code.as_slice(),
                            )),
                        });
                self.module = Some(module);
            } else if !self.data.wgsl_code.is_empty() {
                let module =
                    context
                        .webgpu
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(shader_name.as_str()),
                            source: wgpu::ShaderSource::Wgsl(self.data.wgsl_code.clone().into()),
                        });
                self.module = Some(module);
            }
        }
        self.module.is_some()
    }
    pub fn module(&self) -> &ShaderModule {
        self.module.as_ref().unwrap()
    }
}
