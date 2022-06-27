pub const MAX_TEXTURE_COORDS_SETS: usize = 4;

pub enum VertexFormat {
    Uint8x2 = wgpu::VertexFormat::Uint8x2 as _,
    Uint8x4 = wgpu::VertexFormat::Uint8x4 as _,
    Sint8x2 = wgpu::VertexFormat::Sint8x2 as _,
    Sint8x4 = wgpu::VertexFormat::Sint8x4 as _,
    Unorm8x2 = wgpu::VertexFormat::Unorm8x2 as _,
    Unorm8x4 = wgpu::VertexFormat::Unorm8x4 as _,
    Snorm8x2 = wgpu::VertexFormat::Snorm8x2 as _,
    Snorm8x4 = wgpu::VertexFormat::Snorm8x4 as _,
    Uint16x2 = wgpu::VertexFormat::Uint16x2 as _,
    Uint16x4 = wgpu::VertexFormat::Uint16x4 as _,
    Sint16x2 = wgpu::VertexFormat::Sint16x2 as _,
    Sint16x4 = wgpu::VertexFormat::Sint16x4 as _,
    Unorm16x2 = wgpu::VertexFormat::Unorm16x2 as _,
    Unorm16x4 = wgpu::VertexFormat::Unorm16x4 as _,
    Snorm16x2 = wgpu::VertexFormat::Snorm16x2 as _,
    Snorm16x4 = wgpu::VertexFormat::Snorm16x4 as _,
    Float16x2 = wgpu::VertexFormat::Float16x2 as _,
    Float16x4 = wgpu::VertexFormat::Float16x4 as _,
    Float32 = wgpu::VertexFormat::Float32 as _,
    Float32x2 = wgpu::VertexFormat::Float32x2 as _,
    Float32x3 = wgpu::VertexFormat::Float32x3 as _,
    Float32x4 = wgpu::VertexFormat::Float32x4 as _,
    Uint32 = wgpu::VertexFormat::Uint32 as _,
    Uint32x2 = wgpu::VertexFormat::Uint32x2 as _,
    Uint32x3 = wgpu::VertexFormat::Uint32x3 as _,
    Uint32x4 = wgpu::VertexFormat::Uint32x4 as _,
    Sint32 = wgpu::VertexFormat::Sint32 as _,
    Sint32x2 = wgpu::VertexFormat::Sint32x2 as _,
    Sint32x3 = wgpu::VertexFormat::Sint32x3 as _,
    Sint32x4 = wgpu::VertexFormat::Sint32x4 as _,
    Float64 = wgpu::VertexFormat::Float64 as _,
    Float64x2 = wgpu::VertexFormat::Float64x2 as _,
    Float64x3 = wgpu::VertexFormat::Float64x3 as _,
    Float64x4 = wgpu::VertexFormat::Float64x4 as _,
}

impl From<VertexFormat> for wgpu::VertexFormat {
    fn from(format: VertexFormat) -> wgpu::VertexFormat {
        match format {
            VertexFormat::Uint8x2 => wgpu::VertexFormat::Uint8x2,
            VertexFormat::Uint8x4 => wgpu::VertexFormat::Uint8x4,
            VertexFormat::Sint8x2 => wgpu::VertexFormat::Sint8x2,
            VertexFormat::Sint8x4 => wgpu::VertexFormat::Sint8x4,
            VertexFormat::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            VertexFormat::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
            VertexFormat::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            VertexFormat::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            VertexFormat::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            VertexFormat::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            VertexFormat::Sint16x2 => wgpu::VertexFormat::Sint16x2,
            VertexFormat::Sint16x4 => wgpu::VertexFormat::Sint16x4,
            VertexFormat::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            VertexFormat::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            VertexFormat::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
            VertexFormat::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
            VertexFormat::Float16x2 => wgpu::VertexFormat::Float16x2,
            VertexFormat::Float16x4 => wgpu::VertexFormat::Float16x4,
            VertexFormat::Float32 => wgpu::VertexFormat::Float32,
            VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
            VertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
            VertexFormat::Uint32 => wgpu::VertexFormat::Uint32,
            VertexFormat::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            VertexFormat::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            VertexFormat::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            VertexFormat::Sint32 => wgpu::VertexFormat::Sint32,
            VertexFormat::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            VertexFormat::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            VertexFormat::Sint32x4 => wgpu::VertexFormat::Sint32x4,
            VertexFormat::Float64 => wgpu::VertexFormat::Float64,
            VertexFormat::Float64x2 => wgpu::VertexFormat::Float64x2,
            VertexFormat::Float64x3 => wgpu::VertexFormat::Float64x3,
            VertexFormat::Float64x4 => wgpu::VertexFormat::Float64x4,
        }
    }
}

impl From<wgpu::VertexFormat> for VertexFormat {
    fn from(format: wgpu::VertexFormat) -> Self {
        match format {
            wgpu::VertexFormat::Uint8x2 => VertexFormat::Uint8x2,
            wgpu::VertexFormat::Uint8x4 => VertexFormat::Uint8x4,
            wgpu::VertexFormat::Sint8x2 => VertexFormat::Sint8x2,
            wgpu::VertexFormat::Sint8x4 => VertexFormat::Sint8x4,
            wgpu::VertexFormat::Unorm8x2 => VertexFormat::Unorm8x2,
            wgpu::VertexFormat::Unorm8x4 => VertexFormat::Unorm8x4,
            wgpu::VertexFormat::Snorm8x2 => VertexFormat::Snorm8x2,
            wgpu::VertexFormat::Snorm8x4 => VertexFormat::Snorm8x4,
            wgpu::VertexFormat::Uint16x2 => VertexFormat::Uint16x2,
            wgpu::VertexFormat::Uint16x4 => VertexFormat::Uint16x4,
            wgpu::VertexFormat::Sint16x2 => VertexFormat::Sint16x2,
            wgpu::VertexFormat::Sint16x4 => VertexFormat::Sint16x4,
            wgpu::VertexFormat::Unorm16x2 => VertexFormat::Unorm16x2,
            wgpu::VertexFormat::Unorm16x4 => VertexFormat::Unorm16x4,
            wgpu::VertexFormat::Snorm16x2 => VertexFormat::Snorm16x2,
            wgpu::VertexFormat::Snorm16x4 => VertexFormat::Snorm16x4,
            wgpu::VertexFormat::Float16x2 => VertexFormat::Float16x2,
            wgpu::VertexFormat::Float16x4 => VertexFormat::Float16x4,
            wgpu::VertexFormat::Float32 => VertexFormat::Float32,
            wgpu::VertexFormat::Float32x2 => VertexFormat::Float32x2,
            wgpu::VertexFormat::Float32x3 => VertexFormat::Float32x3,
            wgpu::VertexFormat::Float32x4 => VertexFormat::Float32x4,
            wgpu::VertexFormat::Uint32 => VertexFormat::Uint32,
            wgpu::VertexFormat::Uint32x2 => VertexFormat::Uint32x2,
            wgpu::VertexFormat::Uint32x3 => VertexFormat::Uint32x3,
            wgpu::VertexFormat::Uint32x4 => VertexFormat::Uint32x4,
            wgpu::VertexFormat::Sint32 => VertexFormat::Sint32,
            wgpu::VertexFormat::Sint32x2 => VertexFormat::Sint32x2,
            wgpu::VertexFormat::Sint32x3 => VertexFormat::Sint32x3,
            wgpu::VertexFormat::Sint32x4 => VertexFormat::Sint32x4,
            wgpu::VertexFormat::Float64 => VertexFormat::Float64,
            wgpu::VertexFormat::Float64x2 => VertexFormat::Float64x2,
            wgpu::VertexFormat::Float64x3 => VertexFormat::Float64x3,
            wgpu::VertexFormat::Float64x4 => VertexFormat::Float64x4,
        }
    }
}

pub struct VertexBufferLayoutBuilder<'a> {
    layout: wgpu::VertexBufferLayout<'a>,
    attributes: Vec<wgpu::VertexAttribute>,
    offset: wgpu::BufferAddress,
    location: u32,
}

impl<'a> VertexBufferLayoutBuilder<'a> {
    pub fn vertex() -> Self {
        Self {
            attributes: vec![],
            layout: wgpu::VertexBufferLayout {
                attributes: &[],
                array_stride: 0,
                step_mode: wgpu::VertexStepMode::Vertex,
            },
            offset: 0,
            location: 0,
        }
    }
    pub fn instance() -> Self {
        Self {
            attributes: vec![],
            layout: wgpu::VertexBufferLayout {
                attributes: &[],
                array_stride: 0,
                step_mode: wgpu::VertexStepMode::Instance,
            },
            offset: 0,
            location: 0,
        }
    }
    pub fn add_attribute<T>(&mut self, format: wgpu::VertexFormat) {
        self.attributes.push(wgpu::VertexAttribute {
            offset: self.offset,
            shader_location: self.location,
            format,
        });
        self.offset += std::mem::size_of::<T>() as wgpu::BufferAddress;
        self.location += 1;
    }

    pub fn starting_location(&mut self, location: u32) {
        self.location = location;
    }

    pub fn location(&self) -> u32 {
        self.location
    }

    pub fn build(&'a self) -> wgpu::VertexBufferLayout<'a> {
        let mut layout = self.layout.clone();
        layout.array_stride = self.offset;
        layout.attributes = &self.attributes;
        layout
    }
}
