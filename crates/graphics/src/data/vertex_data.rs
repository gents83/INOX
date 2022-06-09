pub const MAX_TEXTURE_COORDS_SETS: usize = 4;
pub struct VertexBufferLayoutBuilder<'a> {
    layout: wgpu::VertexBufferLayout<'a>,
    attributes: Vec<wgpu::VertexAttribute>,
    offset: wgpu::BufferAddress,
    location: u32,
}

impl<'a> VertexBufferLayoutBuilder<'a> {
    pub fn create_draw_vertex_format() -> Self {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.add_attribute::<u32>(wgpu::VertexFormat::Uint32);
        layout_builder.add_attribute::<i32>(wgpu::VertexFormat::Sint32);
        layout_builder.add_attribute::<i32>(wgpu::VertexFormat::Sint32);
        layout_builder.add_attribute::<i32>(wgpu::VertexFormat::Sint32);
        for _ in 0..MAX_TEXTURE_COORDS_SETS {
            layout_builder.add_attribute::<u32>(wgpu::VertexFormat::Sint32);
        }
        layout_builder.layout.array_stride = layout_builder.offset;
        layout_builder
    }

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
