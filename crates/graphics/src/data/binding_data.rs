use std::mem::size_of;

use inox_resources::HashBuffer;

use crate::{
    DataBuffer, LightData, LightId, Material, MaterialId, RenderContext, ShaderMaterialData,
    TextureData, TextureId, TextureType, INVALID_INDEX, MAX_NUM_LIGHTS, MAX_NUM_MATERIALS,
    MAX_NUM_TEXTURES,
};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1;

#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConstantData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub screen_width: f32,
    pub screen_height: f32,
    pub flags: u32,
    pub num_lights: u32,
}

impl Default for ConstantData {
    fn default() -> Self {
        Self {
            view: [[0.; 4]; 4],
            proj: [[0.; 4]; 4],
            screen_width: 0.,
            screen_height: 0.,
            flags: CONSTANT_DATA_FLAGS_NONE,
            num_lights: 0,
        }
    }
}

#[derive(Default)]
struct DynamicData {
    textures_data: HashBuffer<TextureId, TextureData, MAX_NUM_TEXTURES>,
    materials_data: HashBuffer<MaterialId, ShaderMaterialData, MAX_NUM_MATERIALS>,
    lights_data: HashBuffer<LightId, LightData, MAX_NUM_LIGHTS>,
}

#[derive(Default)]
pub struct BindingData {
    constant_data: ConstantData,
    dynamic_data: DynamicData,
    constant_data_buffer: DataBuffer,
    dynamic_data_buffer: DataBuffer,
    data_bind_group_layout: Option<wgpu::BindGroupLayout>,
    data_bind_group: Option<wgpu::BindGroup>,
}

impl BindingData {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.data_bind_group.as_ref().unwrap()
    }
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.data_bind_group_layout.as_ref().unwrap()
    }

    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        let usage = wgpu::BufferUsages::UNIFORM
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST;

        self.constant_data_buffer.init::<ConstantData>(
            context,
            size_of::<ConstantData>() as _,
            usage,
        );

        self.constant_data.num_lights = self.dynamic_data.lights_data.item_count() as _;

        self.constant_data_buffer
            .add_to_gpu_buffer(context, &[self.constant_data]);

        let total_size = size_of::<TextureData>() * MAX_NUM_TEXTURES
            + size_of::<ShaderMaterialData>() * MAX_NUM_MATERIALS
            + size_of::<LightData>() * MAX_NUM_LIGHTS;
        self.dynamic_data_buffer
            .init::<DynamicData>(context, total_size as _, usage);
        self.dynamic_data_buffer
            .add_to_gpu_buffer(context, self.dynamic_data.textures_data.data());
        self.dynamic_data_buffer
            .add_to_gpu_buffer(context, self.dynamic_data.materials_data.data());
        self.dynamic_data_buffer
            .add_to_gpu_buffer(context, self.dynamic_data.lights_data.data());

        self.init_bind_group(context);
    }

    pub fn constant_data_mut(&mut self) -> &mut ConstantData {
        &mut self.constant_data
    }

    pub fn set_light_data(&mut self, light_id: &LightId, data: LightData) -> usize {
        self.dynamic_data.lights_data.insert(light_id, data)
    }

    pub fn set_texture_data(&mut self, texture_id: &TextureId, data: TextureData) -> usize {
        self.dynamic_data.textures_data.insert(texture_id, data)
    }

    pub fn set_material_data(&mut self, material_id: &MaterialId, material: &Material) -> usize {
        let mut textures_indices = [INVALID_INDEX; TextureType::Count as _];
        material
            .textures()
            .iter()
            .enumerate()
            .for_each(|(i, handle_texture)| {
                if let Some(texture) = handle_texture {
                    textures_indices[i] = texture.get().uniform_index() as _;
                }
            });
        let shader_material_data = ShaderMaterialData {
            textures_indices,
            textures_coord_set: *material.textures_coords_set(),
            roughness_factor: material.roughness_factor(),
            metallic_factor: material.metallic_factor(),
            alpha_cutoff: material.alpha_cutoff(),
            alpha_mode: material.alpha_mode() as _,
            base_color: material.base_color().into(),
            emissive_color: material.emissive_color().into(),
            diffuse_color: material.diffuse_color().into(),
            specular_color: material.specular_color().into(),
        };
        self.dynamic_data
            .materials_data
            .insert(material_id, shader_material_data)
    }

    fn init_bind_group(&mut self, render_context: &RenderContext) {
        if self.data_bind_group.is_some() && self.data_bind_group_layout.is_some() {
            return;
        }
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
                        resource: self.constant_data_buffer.gpu_buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.dynamic_data_buffer.gpu_buffer().as_entire_binding(),
                    },
                ],
                label: Some(label.as_str()),
            });
        self.data_bind_group = Some(data_bind_group);
        self.data_bind_group_layout = Some(data_bind_group_layout);
    }
}
