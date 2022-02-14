use crate::{
    DataBuffer, LightData, RenderContext, ShaderMaterialData, TextureData, MAX_NUM_LIGHTS,
    MAX_NUM_MATERIALS, MAX_NUM_TEXTURES,
};

#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConstantData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for ConstantData {
    fn default() -> Self {
        Self {
            view: [[0.; 4]; 4],
            proj: [[0.; 4]; 4],
            screen_width: 0.,
            screen_height: 0.,
        }
    }
}

#[repr(C, align(4))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DynamicData {
    pub light_data: [LightData; MAX_NUM_LIGHTS],
    pub textures_data: [TextureData; MAX_NUM_TEXTURES],
    pub materials_data: [ShaderMaterialData; MAX_NUM_MATERIALS],
    pub num_lights: u32,
}

impl Default for DynamicData {
    fn default() -> Self {
        Self {
            light_data: [LightData::default(); MAX_NUM_LIGHTS],
            textures_data: [TextureData::default(); MAX_NUM_TEXTURES],
            materials_data: [ShaderMaterialData::default(); MAX_NUM_MATERIALS],
            num_lights: 0,
        }
    }
}

#[derive(Default)]
pub struct ShaderData {
    pub constant_data: DataBuffer<ConstantData, 1>,
    pub dynamic_data: DataBuffer<DynamicData, 1>,
    data_bind_group_layout: Option<wgpu::BindGroupLayout>,
    data_bind_group: Option<wgpu::BindGroup>,
}

impl ShaderData {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.data_bind_group.as_ref().unwrap()
    }
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.data_bind_group_layout.as_ref().unwrap()
    }

    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        self.constant_data.send_to_gpu(context);
        self.dynamic_data.send_to_gpu(context);

        if self.data_bind_group.is_none() {
            let (data_bind_group_layout, data_bind_group) =
                Self::create_data(context, &mut self.constant_data, &mut self.dynamic_data);
            self.data_bind_group = Some(data_bind_group);
            self.data_bind_group_layout = Some(data_bind_group_layout);
        }
    }

    pub fn constant_data_mut(&mut self) -> &mut ConstantData {
        &mut self.constant_data.data_mut()[0]
    }

    pub fn light_data_mut(&mut self) -> &mut [LightData] {
        &mut self.dynamic_data.data_mut()[0].light_data
    }

    pub fn textures_data_mut(&mut self) -> &mut [TextureData] {
        &mut self.dynamic_data.data_mut()[0].textures_data
    }

    pub fn material_data_mut(&mut self) -> &mut [ShaderMaterialData] {
        &mut self.dynamic_data.data_mut()[0].materials_data
    }
    pub fn set_num_lights(&mut self, num_lights: usize) {
        self.dynamic_data.data_mut()[0].num_lights = num_lights as _;
    }

    fn create_data(
        render_context: &RenderContext,
        constant_data: &mut DataBuffer<ConstantData, 1>,
        dynamic_data: &mut DataBuffer<DynamicData, 1>,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let typename = std::any::type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();

        let label = format!("{} bind group layout", typename);
        let data_bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some(label.as_str()),
                });

        let label = format!("{} bind group", typename);
        let data_bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &data_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: constant_data.data_buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: dynamic_data.data_buffer().as_entire_binding(),
                    },
                ],
                label: Some(label.as_str()),
            });

        (data_bind_group_layout, data_bind_group)
    }
}
