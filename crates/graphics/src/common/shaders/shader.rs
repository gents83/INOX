use std::{fs::File, io::Read, path::Path};

use crate::RenderContext;

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
    data.seek(::std::io::SeekFrom::Start(0)).unwrap();
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

fn parse_shader_from(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    String::from_utf8(data).unwrap()
}

pub fn create_shader(context: &RenderContext, path: &Path) -> Option<wgpu::ShaderModule> {
    if let Some(extension) = path.extension() {
        match extension.to_str().unwrap() {
            SHADER_EXTENSION_SPV => unsafe {
                let mut shader_file = File::open(path).unwrap();
                let shader_code = read_spirv_from_bytes(&mut shader_file);

                return Some(context.device.create_shader_module_spirv(
                    &wgpu::ShaderModuleDescriptorSpirV {
                        label: Some("Shader"),
                        source: std::borrow::Cow::Borrowed(shader_code.as_slice()),
                    },
                ));
            },
            SHADER_EXTENSION_WGSL => {
                let shader_code = parse_shader_from(path);
                return Some(
                    context
                        .device
                        .create_shader_module(&wgpu::ShaderModuleDescriptor {
                            label: Some("Shader"),
                            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
                        }),
                );
            }
            _ => {
                eprintln!(
                    "Unsupported shader extension: {}",
                    extension.to_string_lossy()
                );
            }
        }
    }
    None
}
