use crate::print_field_size;

use inox_bitmask::bitmask;
use inox_math::decode_half;
use inox_resources::ResourceTrait;
use inox_serialize::{Deserialize, Serialize};

#[bitmask]
pub enum TextureUsage {
    CopySrc,
    CopyDst,
    TextureBinding,
    StorageBinding,
    RenderTarget,
}

impl From<TextureUsage> for wgpu::TextureUsages {
    fn from(v: TextureUsage) -> Self {
        Self::from_bits(v.bits()).unwrap()
    }
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub sample_count: u32,
    pub is_LUT: bool,
    pub data: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "inox_serialize")]
pub enum TextureType {
    BaseColor = 0,
    MetallicRoughness = 1,
    Normal = 2,
    Emissive = 3,
    Occlusion = 4,
    SpecularGlossiness = 5,
    Diffuse = 6,
    Specular = 7,
    SpecularColor = 8,
    Transmission = 9,
    Thickness = 10,
    EmptyForPadding3 = 11,
    EmptyForPadding4 = 12,
    EmptyForPadding5 = 13,
    EmptyForPadding6 = 14,
    EmptyForPadding7 = 15,
    Count = 16,
}

impl From<TextureType> for usize {
    fn from(val: TextureType) -> Self {
        val as _
    }
}
impl From<usize> for TextureType {
    fn from(value: usize) -> Self {
        match value {
            0 => TextureType::BaseColor,
            1 => TextureType::MetallicRoughness,
            2 => TextureType::Normal,
            3 => TextureType::Emissive,
            4 => TextureType::Occlusion,
            5 => TextureType::SpecularGlossiness,
            6 => TextureType::Diffuse,
            7 => TextureType::Specular,
            8 => TextureType::SpecularColor,
            9 => TextureType::Transmission,
            10 => TextureType::Thickness,
            16 => TextureType::Count,
            _ => panic!("Invalid TextureType value: {value}"),
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct GPUTexture {
    pub texture_and_layer_index: i32, //negatives are LUT textures - 29bit + 3bit
    pub min: u32,
    pub max: u32,
    pub size: u32,
}

impl ResourceTrait for GPUTexture {
    fn is_initialized(&self) -> bool {
        self.min > 0 && self.max > 0 && self.size > 0
    }

    fn invalidate(&mut self) -> &mut Self {
        self.min = 0;
        self.max = 0;
        self.size = 0;
        self
    }
}

impl GPUTexture {
    #[allow(non_snake_case)]
    pub fn is_LUT(&self) -> bool {
        self.texture_and_layer_index.is_negative()
    }
    pub fn texture_index(&self) -> u32 {
        self.texture_and_layer_index.unsigned_abs() >> 3
    }
    pub fn layer_index(&self) -> u32 {
        (self.texture_and_layer_index & 0x00000007) as _
    }
    pub fn total_width(&self) -> u32 {
        decode_half((self.size & 0x0000FFFF) as _) as _
    }
    pub fn total_height(&self) -> u32 {
        decode_half(((self.size & 0xFFFF0000) >> 16) as _) as _
    }
    pub fn x(&self) -> u32 {
        decode_half((self.min & 0x0000FFFF) as _) as _
    }
    pub fn y(&self) -> u32 {
        decode_half(((self.min & 0xFFFF0000) >> 16) as _) as _
    }
    pub fn width(&self) -> u32 {
        decode_half((self.max & 0x0000FFFF) as _) as _
    }
    pub fn height(&self) -> u32 {
        decode_half(((self.max & 0xFFFF0000) >> 16) as _) as _
    }
}

impl GPUTexture {
    #[allow(deref_nullptr)]
    pub fn debug_size(alignment_size: usize) {
        let total_size = std::mem::size_of::<Self>();
        println!("ShaderTextureData info: Total size [{total_size}]");

        let mut s = 0;
        print_field_size!(s, texture_and_layer_index, u32, 1);
        print_field_size!(s, min, u32, 1);
        print_field_size!(s, max, u32, 1);
        print_field_size!(s, size, u32, 1);

        println!(
            "Alignment result: {} -> {}",
            if s == total_size && s % alignment_size == 0 {
                "OK"
            } else {
                "TO ALIGN"
            },
            (s as f32 / alignment_size as f32).ceil() as usize * alignment_size
        );
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialOrd, PartialEq)]
#[serde(crate = "inox_serialize")]
pub enum AstcBlock {
    B4x4,
    B5x4,
    B5x5,
    B6x5,
    B6x6,
    B8x5,
    B8x6,
    B8x8,
    B10x5,
    B10x6,
    B10x8,
    B10x10,
    B12x10,
    B12x12,
}

impl From<AstcBlock> for wgpu::AstcBlock {
    fn from(val: AstcBlock) -> Self {
        match val {
            AstcBlock::B4x4 => wgpu::AstcBlock::B4x4,
            AstcBlock::B5x4 => wgpu::AstcBlock::B5x4,
            AstcBlock::B5x5 => wgpu::AstcBlock::B5x5,
            AstcBlock::B6x5 => wgpu::AstcBlock::B6x5,
            AstcBlock::B6x6 => wgpu::AstcBlock::B6x6,
            AstcBlock::B8x5 => wgpu::AstcBlock::B8x5,
            AstcBlock::B8x6 => wgpu::AstcBlock::B8x6,
            AstcBlock::B8x8 => wgpu::AstcBlock::B8x8,
            AstcBlock::B10x5 => wgpu::AstcBlock::B10x5,
            AstcBlock::B10x6 => wgpu::AstcBlock::B10x6,
            AstcBlock::B10x8 => wgpu::AstcBlock::B10x8,
            AstcBlock::B10x10 => wgpu::AstcBlock::B10x10,
            AstcBlock::B12x10 => wgpu::AstcBlock::B12x10,
            AstcBlock::B12x12 => wgpu::AstcBlock::B12x12,
        }
    }
}

impl From<wgpu::AstcBlock> for crate::AstcBlock {
    fn from(val: wgpu::AstcBlock) -> Self {
        match val {
            wgpu::AstcBlock::B4x4 => crate::AstcBlock::B4x4,
            wgpu::AstcBlock::B5x4 => crate::AstcBlock::B5x4,
            wgpu::AstcBlock::B5x5 => crate::AstcBlock::B5x5,
            wgpu::AstcBlock::B6x5 => crate::AstcBlock::B6x5,
            wgpu::AstcBlock::B6x6 => crate::AstcBlock::B6x6,
            wgpu::AstcBlock::B8x5 => crate::AstcBlock::B8x5,
            wgpu::AstcBlock::B8x6 => crate::AstcBlock::B8x6,
            wgpu::AstcBlock::B8x8 => crate::AstcBlock::B8x8,
            wgpu::AstcBlock::B10x5 => crate::AstcBlock::B10x5,
            wgpu::AstcBlock::B10x6 => crate::AstcBlock::B10x6,
            wgpu::AstcBlock::B10x8 => crate::AstcBlock::B10x8,
            wgpu::AstcBlock::B10x10 => crate::AstcBlock::B10x10,
            wgpu::AstcBlock::B12x10 => crate::AstcBlock::B12x10,
            wgpu::AstcBlock::B12x12 => crate::AstcBlock::B12x12,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialOrd, PartialEq)]
#[serde(crate = "inox_serialize")]
pub enum AstcChannel {
    Unorm,
    UnormSrgb,
    Hdr,
}

impl From<AstcChannel> for wgpu::AstcChannel {
    fn from(val: AstcChannel) -> Self {
        match val {
            AstcChannel::Unorm => wgpu::AstcChannel::Unorm,
            AstcChannel::UnormSrgb => wgpu::AstcChannel::UnormSrgb,
            AstcChannel::Hdr => wgpu::AstcChannel::Hdr,
        }
    }
}

impl From<wgpu::AstcChannel> for crate::AstcChannel {
    fn from(val: wgpu::AstcChannel) -> Self {
        match val {
            wgpu::AstcChannel::Unorm => crate::AstcChannel::Unorm,
            wgpu::AstcChannel::UnormSrgb => crate::AstcChannel::UnormSrgb,
            wgpu::AstcChannel::Hdr => crate::AstcChannel::Hdr,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialOrd, PartialEq)]
#[serde(crate = "inox_serialize")]
pub enum TextureFormat {
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R16Uint,
    R16Sint,
    R16Unorm,
    R16Snorm,
    R16Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,
    R32Uint,
    R32Sint,
    R32Float,
    Rg16Uint,
    Rg16Sint,
    Rg16Unorm,
    Rg16Snorm,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgb10a2Uint,
    Rgb10a2Unorm,
    Rg11b10Float,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Unorm,
    Rgba16Snorm,
    Rgba16Float,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
    Stencil8,
    Depth16Unorm,
    Depth32Float,
    Depth32FloatStencil8,
    Depth24Plus,
    Depth24PlusStencil8,
    NV12,
    Rgb9e5Ufloat,
    Bc1RgbaUnorm,
    Bc1RgbaUnormSrgb,
    Bc2RgbaUnorm,
    Bc2RgbaUnormSrgb,
    Bc3RgbaUnorm,
    Bc3RgbaUnormSrgb,
    Bc4RUnorm,
    Bc4RSnorm,
    Bc5RgUnorm,
    Bc5RgSnorm,
    Bc6hRgbUfloat,
    Bc6hRgbFloat,
    Bc7RgbaUnorm,
    Bc7RgbaUnormSrgb,
    Etc2Rgb8Unorm,
    Etc2Rgb8UnormSrgb,
    Etc2Rgb8A1Unorm,
    Etc2Rgb8A1UnormSrgb,
    Etc2Rgba8Unorm,
    Etc2Rgba8UnormSrgb,
    EacR11Unorm,
    EacR11Snorm,
    EacRg11Unorm,
    EacRg11Snorm,
    Astc {
        block: AstcBlock,
        channel: AstcChannel,
    },
}

impl TextureFormat {
    pub fn aspect(&self) -> wgpu::TextureAspect {
        let fmt: wgpu::TextureFormat = (*self).into();
        if fmt.has_depth_aspect() && !fmt.has_stencil_aspect() {
            wgpu::TextureAspect::DepthOnly
        } else if fmt.has_stencil_aspect() && !fmt.has_depth_aspect() {
            wgpu::TextureAspect::StencilOnly
        } else {
            wgpu::TextureAspect::All
        }
    }
}

impl From<TextureFormat> for wgpu::TextureFormat {
    fn from(format: TextureFormat) -> Self {
        match format {
            crate::TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            crate::TextureFormat::R8Snorm => wgpu::TextureFormat::R8Snorm,
            crate::TextureFormat::R8Uint => wgpu::TextureFormat::R8Uint,
            crate::TextureFormat::R8Sint => wgpu::TextureFormat::R8Sint,
            crate::TextureFormat::R16Uint => wgpu::TextureFormat::R16Uint,
            crate::TextureFormat::R16Sint => wgpu::TextureFormat::R16Sint,
            crate::TextureFormat::R16Unorm => wgpu::TextureFormat::R16Unorm,
            crate::TextureFormat::R16Snorm => wgpu::TextureFormat::R16Snorm,
            crate::TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
            crate::TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            crate::TextureFormat::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
            crate::TextureFormat::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
            crate::TextureFormat::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
            crate::TextureFormat::R32Uint => wgpu::TextureFormat::R32Uint,
            crate::TextureFormat::R32Sint => wgpu::TextureFormat::R32Sint,
            crate::TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
            crate::TextureFormat::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
            crate::TextureFormat::Rg16Sint => wgpu::TextureFormat::Rg16Sint,
            crate::TextureFormat::Rg16Unorm => wgpu::TextureFormat::Rg16Unorm,
            crate::TextureFormat::Rg16Snorm => wgpu::TextureFormat::Rg16Snorm,
            crate::TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
            crate::TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            crate::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            crate::TextureFormat::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
            crate::TextureFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            crate::TextureFormat::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
            crate::TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            crate::TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            crate::TextureFormat::Rgb10a2Uint => wgpu::TextureFormat::Rgb10a2Uint,
            crate::TextureFormat::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
            crate::TextureFormat::Rg11b10Float => wgpu::TextureFormat::Rg11b10Float,
            crate::TextureFormat::Rg32Uint => wgpu::TextureFormat::Rg32Uint,
            crate::TextureFormat::Rg32Sint => wgpu::TextureFormat::Rg32Sint,
            crate::TextureFormat::Rg32Float => wgpu::TextureFormat::Rg32Float,
            crate::TextureFormat::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
            crate::TextureFormat::Rgba16Sint => wgpu::TextureFormat::Rgba16Sint,
            crate::TextureFormat::Rgba16Unorm => wgpu::TextureFormat::Rgba16Unorm,
            crate::TextureFormat::Rgba16Snorm => wgpu::TextureFormat::Rgba16Snorm,
            crate::TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            crate::TextureFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
            crate::TextureFormat::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
            crate::TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
            crate::TextureFormat::Stencil8 => wgpu::TextureFormat::Stencil8,
            crate::TextureFormat::Depth16Unorm => wgpu::TextureFormat::Depth16Unorm,
            crate::TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
            crate::TextureFormat::Depth32FloatStencil8 => wgpu::TextureFormat::Depth32FloatStencil8,
            crate::TextureFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
            crate::TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
            crate::TextureFormat::NV12 => wgpu::TextureFormat::NV12,
            crate::TextureFormat::Rgb9e5Ufloat => wgpu::TextureFormat::Rgb9e5Ufloat,
            crate::TextureFormat::Bc1RgbaUnorm => wgpu::TextureFormat::Bc1RgbaUnorm,
            crate::TextureFormat::Bc1RgbaUnormSrgb => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
            crate::TextureFormat::Bc2RgbaUnorm => wgpu::TextureFormat::Bc2RgbaUnorm,
            crate::TextureFormat::Bc2RgbaUnormSrgb => wgpu::TextureFormat::Bc2RgbaUnormSrgb,
            crate::TextureFormat::Bc3RgbaUnorm => wgpu::TextureFormat::Bc3RgbaUnorm,
            crate::TextureFormat::Bc3RgbaUnormSrgb => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
            crate::TextureFormat::Bc4RUnorm => wgpu::TextureFormat::Bc4RUnorm,
            crate::TextureFormat::Bc4RSnorm => wgpu::TextureFormat::Bc4RSnorm,
            crate::TextureFormat::Bc5RgUnorm => wgpu::TextureFormat::Bc5RgUnorm,
            crate::TextureFormat::Bc5RgSnorm => wgpu::TextureFormat::Bc5RgSnorm,
            crate::TextureFormat::Bc6hRgbUfloat => wgpu::TextureFormat::Bc6hRgbUfloat,
            crate::TextureFormat::Bc6hRgbFloat => wgpu::TextureFormat::Bc6hRgbFloat,
            crate::TextureFormat::Bc7RgbaUnorm => wgpu::TextureFormat::Bc7RgbaUnorm,
            crate::TextureFormat::Bc7RgbaUnormSrgb => wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            crate::TextureFormat::Etc2Rgb8Unorm => wgpu::TextureFormat::Etc2Rgb8Unorm,
            crate::TextureFormat::Etc2Rgb8UnormSrgb => wgpu::TextureFormat::Etc2Rgb8UnormSrgb,
            crate::TextureFormat::Etc2Rgb8A1Unorm => wgpu::TextureFormat::Etc2Rgb8A1Unorm,
            crate::TextureFormat::Etc2Rgb8A1UnormSrgb => wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb,
            crate::TextureFormat::Etc2Rgba8Unorm => wgpu::TextureFormat::Etc2Rgba8Unorm,
            crate::TextureFormat::Etc2Rgba8UnormSrgb => wgpu::TextureFormat::Etc2Rgba8UnormSrgb,
            crate::TextureFormat::EacR11Unorm => wgpu::TextureFormat::EacR11Unorm,
            crate::TextureFormat::EacR11Snorm => wgpu::TextureFormat::EacR11Snorm,
            crate::TextureFormat::EacRg11Unorm => wgpu::TextureFormat::EacRg11Unorm,
            crate::TextureFormat::EacRg11Snorm => wgpu::TextureFormat::EacRg11Snorm,
            crate::TextureFormat::Astc {
                block: b,
                channel: c,
            } => wgpu::TextureFormat::Astc {
                block: b.into(),
                channel: c.into(),
            },
        }
    }
}

impl From<wgpu::TextureFormat> for crate::TextureFormat {
    fn from(format: wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::R8Unorm => crate::TextureFormat::R8Unorm,
            wgpu::TextureFormat::R8Snorm => crate::TextureFormat::R8Snorm,
            wgpu::TextureFormat::R8Uint => crate::TextureFormat::R8Uint,
            wgpu::TextureFormat::R8Sint => crate::TextureFormat::R8Sint,
            wgpu::TextureFormat::R16Uint => crate::TextureFormat::R16Uint,
            wgpu::TextureFormat::R16Sint => crate::TextureFormat::R16Sint,
            wgpu::TextureFormat::R16Unorm => crate::TextureFormat::R16Unorm,
            wgpu::TextureFormat::R16Snorm => crate::TextureFormat::R16Snorm,
            wgpu::TextureFormat::R16Float => crate::TextureFormat::R16Float,
            wgpu::TextureFormat::Rg8Unorm => crate::TextureFormat::Rg8Unorm,
            wgpu::TextureFormat::Rg8Snorm => crate::TextureFormat::Rg8Snorm,
            wgpu::TextureFormat::Rg8Uint => crate::TextureFormat::Rg8Uint,
            wgpu::TextureFormat::Rg8Sint => crate::TextureFormat::Rg8Sint,
            wgpu::TextureFormat::R32Uint => crate::TextureFormat::R32Uint,
            wgpu::TextureFormat::R32Sint => crate::TextureFormat::R32Sint,
            wgpu::TextureFormat::R32Float => crate::TextureFormat::R32Float,
            wgpu::TextureFormat::Rg16Uint => crate::TextureFormat::Rg16Uint,
            wgpu::TextureFormat::Rg16Sint => crate::TextureFormat::Rg16Sint,
            wgpu::TextureFormat::Rg16Unorm => crate::TextureFormat::Rg16Unorm,
            wgpu::TextureFormat::Rg16Snorm => crate::TextureFormat::Rg16Snorm,
            wgpu::TextureFormat::Rg16Float => crate::TextureFormat::Rg16Float,
            wgpu::TextureFormat::Rgba8Unorm => crate::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb => crate::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Snorm => crate::TextureFormat::Rgba8Snorm,
            wgpu::TextureFormat::Rgba8Uint => crate::TextureFormat::Rgba8Uint,
            wgpu::TextureFormat::Rgba8Sint => crate::TextureFormat::Rgba8Sint,
            wgpu::TextureFormat::Bgra8Unorm => crate::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb => crate::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgb10a2Uint => crate::TextureFormat::Rgb10a2Uint,
            wgpu::TextureFormat::Rgb10a2Unorm => crate::TextureFormat::Rgb10a2Unorm,
            wgpu::TextureFormat::Rg11b10Float => crate::TextureFormat::Rg11b10Float,
            wgpu::TextureFormat::Rg32Uint => crate::TextureFormat::Rg32Uint,
            wgpu::TextureFormat::Rg32Sint => crate::TextureFormat::Rg32Sint,
            wgpu::TextureFormat::Rg32Float => crate::TextureFormat::Rg32Float,
            wgpu::TextureFormat::Rgba16Uint => crate::TextureFormat::Rgba16Uint,
            wgpu::TextureFormat::Rgba16Sint => crate::TextureFormat::Rgba16Sint,
            wgpu::TextureFormat::Rgba16Unorm => crate::TextureFormat::Rgba16Unorm,
            wgpu::TextureFormat::Rgba16Snorm => crate::TextureFormat::Rgba16Snorm,
            wgpu::TextureFormat::Rgba16Float => crate::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba32Uint => crate::TextureFormat::Rgba32Uint,
            wgpu::TextureFormat::Rgba32Sint => crate::TextureFormat::Rgba32Sint,
            wgpu::TextureFormat::Rgba32Float => crate::TextureFormat::Rgba32Float,
            wgpu::TextureFormat::Stencil8 => crate::TextureFormat::Stencil8,
            wgpu::TextureFormat::Depth16Unorm => crate::TextureFormat::Depth16Unorm,
            //wgpu::TextureFormat::Depth24UnormStencil8 => crate::TextureFormat::Depth24PlusStencil8,
            wgpu::TextureFormat::Depth32Float => crate::TextureFormat::Depth32Float,
            wgpu::TextureFormat::Depth32FloatStencil8 => crate::TextureFormat::Depth32FloatStencil8,
            wgpu::TextureFormat::Depth24Plus => crate::TextureFormat::Depth24Plus,
            wgpu::TextureFormat::Depth24PlusStencil8 => crate::TextureFormat::Depth24PlusStencil8,
            wgpu::TextureFormat::NV12 => crate::TextureFormat::NV12,
            wgpu::TextureFormat::Rgb9e5Ufloat => crate::TextureFormat::Rgb9e5Ufloat,
            wgpu::TextureFormat::Bc1RgbaUnorm => crate::TextureFormat::Bc1RgbaUnorm,
            wgpu::TextureFormat::Bc1RgbaUnormSrgb => crate::TextureFormat::Bc1RgbaUnormSrgb,
            wgpu::TextureFormat::Bc2RgbaUnorm => crate::TextureFormat::Bc2RgbaUnorm,
            wgpu::TextureFormat::Bc2RgbaUnormSrgb => crate::TextureFormat::Bc2RgbaUnormSrgb,
            wgpu::TextureFormat::Bc3RgbaUnorm => crate::TextureFormat::Bc3RgbaUnorm,
            wgpu::TextureFormat::Bc3RgbaUnormSrgb => crate::TextureFormat::Bc3RgbaUnormSrgb,
            wgpu::TextureFormat::Bc4RUnorm => crate::TextureFormat::Bc4RUnorm,
            wgpu::TextureFormat::Bc4RSnorm => crate::TextureFormat::Bc4RSnorm,
            wgpu::TextureFormat::Bc5RgUnorm => crate::TextureFormat::Bc5RgUnorm,
            wgpu::TextureFormat::Bc5RgSnorm => crate::TextureFormat::Bc5RgSnorm,
            wgpu::TextureFormat::Bc6hRgbUfloat => crate::TextureFormat::Bc6hRgbUfloat,
            wgpu::TextureFormat::Bc6hRgbFloat => crate::TextureFormat::Bc6hRgbFloat,
            wgpu::TextureFormat::Bc7RgbaUnorm => crate::TextureFormat::Bc7RgbaUnorm,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb => crate::TextureFormat::Bc7RgbaUnormSrgb,
            wgpu::TextureFormat::Etc2Rgb8Unorm => crate::TextureFormat::Etc2Rgb8Unorm,
            wgpu::TextureFormat::Etc2Rgb8UnormSrgb => crate::TextureFormat::Etc2Rgb8UnormSrgb,
            wgpu::TextureFormat::Etc2Rgb8A1Unorm => crate::TextureFormat::Etc2Rgb8A1Unorm,
            wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => crate::TextureFormat::Etc2Rgb8A1UnormSrgb,
            wgpu::TextureFormat::Etc2Rgba8Unorm => crate::TextureFormat::Etc2Rgba8Unorm,
            wgpu::TextureFormat::Etc2Rgba8UnormSrgb => crate::TextureFormat::Etc2Rgba8UnormSrgb,
            wgpu::TextureFormat::EacR11Unorm => crate::TextureFormat::EacR11Unorm,
            wgpu::TextureFormat::EacR11Snorm => crate::TextureFormat::EacR11Snorm,
            wgpu::TextureFormat::EacRg11Unorm => crate::TextureFormat::EacRg11Unorm,
            wgpu::TextureFormat::EacRg11Snorm => crate::TextureFormat::EacRg11Snorm,
            wgpu::TextureFormat::Astc {
                block: b,
                channel: c,
            } => crate::TextureFormat::Astc {
                block: b.into(),
                channel: c.into(),
            },
        }
    }
}
