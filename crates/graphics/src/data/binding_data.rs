use inox_resources::{to_u8_slice, Buffer, HashBuffer};
use inox_serialize::{Deserialize, Serialize};
use inox_uid::INVALID_UID;

use crate::{
    DataBuffer, LightData, LightId, Material, MaterialId, RenderContext, ShaderMaterialData,
    TextureData, TextureId, TextureType, INVALID_INDEX, MAX_NUM_LIGHTS, MAX_NUM_MATERIALS,
    MAX_NUM_TEXTURES,
};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct ConstantData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub screen_width: f32,
    pub screen_height: f32,
    pub flags: u32,
}

impl Default for ConstantData {
    fn default() -> Self {
        Self {
            view: [[0.; 4]; 4],
            proj: [[0.; 4]; 4],
            screen_width: 0.,
            screen_height: 0.,
            flags: CONSTANT_DATA_FLAGS_NONE,
        }
    }
}

#[repr(C, align(16))]
#[derive(Default)]
pub struct DynamicData {
    pub textures_data: HashBuffer<TextureId, TextureData, MAX_NUM_TEXTURES>,
    pub materials_data: HashBuffer<MaterialId, ShaderMaterialData, MAX_NUM_MATERIALS>,
    pub lights_data: HashBuffer<LightId, LightData, MAX_NUM_LIGHTS>,
}
impl DynamicData {
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
        self.materials_data
            .insert(material_id, shader_material_data)
    }
}

#[derive(PartialEq, Eq)]
pub enum BindingState {
    Changed,
    Bound,
    Error,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub enum BindingDataType {
    Constant,
    Dynamic,
    Custom,
}

pub struct BindingData {
    data_bind_group_layout: Option<wgpu::BindGroupLayout>,
    data_bind_group: Option<wgpu::BindGroup>,
    custom_data: Buffer,
    custom_data_buffer: DataBuffer,
    custom_data_usage: wgpu::BufferUsages,
    is_read_only: bool,
    is_dirty: bool,
}

impl Default for BindingData {
    fn default() -> Self {
        Self {
            data_bind_group_layout: None,
            data_bind_group: None,
            custom_data: Buffer::default(),
            custom_data_buffer: DataBuffer::default(),
            custom_data_usage: wgpu::BufferUsages::VERTEX,
            is_read_only: false,
            is_dirty: false,
        }
    }
}

impl BindingData {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.data_bind_group.as_ref().unwrap()
    }
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.data_bind_group_layout.as_ref().unwrap()
    }
    pub fn custom_data<T>(&self) -> &T {
        &self.custom_data.total_data()[0]
    }
    pub fn set_custom_data<T>(&mut self, data: T, is_read_only: bool) {
        self.custom_data.allocate_with_size(
            &INVALID_UID,
            to_u8_slice(&[data]),
            std::mem::size_of::<T>(),
        );
        self.is_read_only = is_read_only;
        self.is_dirty = true;
    }

    pub fn send_to_gpu(
        &mut self,
        render_context: &RenderContext,
        binding_data_type: &[BindingDataType],
        constant_data_buffer: &DataBuffer,
        dynamic_data_buffer: &DataBuffer,
    ) -> BindingState {
        if !self.is_dirty && self.data_bind_group.is_some() && self.data_bind_group_layout.is_some()
        {
            return BindingState::Bound;
        }

        if binding_data_type.contains(&BindingDataType::Custom) {
            if self.custom_data.is_empty() {
                return BindingState::Error;
            } else {
                let mut usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
                if !self.is_read_only {
                    usage |= wgpu::BufferUsages::COPY_SRC;
                }
                self.custom_data_buffer.init(
                    render_context,
                    self.custom_data.total_len() as _,
                    usage,
                    "CustomData",
                );
                self.custom_data_buffer
                    .add_to_gpu_buffer(render_context, self.custom_data.data());
            }
        }

        let typename = std::any::type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();

        let mut bind_group_layout = Vec::new();
        let mut bind_group = Vec::new();
        let mut bind_group_count = 0;
        binding_data_type.iter().for_each(|t| match t {
            BindingDataType::Constant => {
                bind_group_layout.push(wgpu::BindGroupLayoutEntry {
                    binding: bind_group_count,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
                bind_group.push(wgpu::BindGroupEntry {
                    binding: bind_group_count,
                    resource: constant_data_buffer
                        .gpu_buffer()
                        .unwrap()
                        .as_entire_binding(),
                });
                bind_group_count += 1;
            }
            BindingDataType::Dynamic => {
                bind_group_layout.push(wgpu::BindGroupLayoutEntry {
                    binding: bind_group_count,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
                bind_group.push(wgpu::BindGroupEntry {
                    binding: bind_group_count,
                    resource: dynamic_data_buffer
                        .gpu_buffer()
                        .unwrap()
                        .as_entire_binding(),
                });
                bind_group_count += 1;
            }
            BindingDataType::Custom => {
                bind_group_layout.push(wgpu::BindGroupLayoutEntry {
                    binding: bind_group_count,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: self.is_read_only,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
                bind_group.push(wgpu::BindGroupEntry {
                    binding: bind_group_count,
                    resource: self
                        .custom_data_buffer
                        .gpu_buffer()
                        .unwrap()
                        .as_entire_binding(),
                });
                bind_group_count += 1;
            }
        });

        let label = format!("{} bind group layout", typename);
        let data_bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: bind_group_layout.as_slice(),
                    label: Some(label.as_str()),
                });

        let label = format!("{} bind group", typename);
        let data_bind_group = render_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &data_bind_group_layout,
                entries: bind_group.as_slice(),
                label: Some(label.as_str()),
            });
        self.data_bind_group = Some(data_bind_group);
        self.data_bind_group_layout = Some(data_bind_group_layout);
        self.is_dirty = false;
        BindingState::Changed
    }
}
