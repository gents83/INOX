use std::{any::type_name, num::NonZeroU32};

use inox_uid::{generate_uid_from_string, Uid};

use crate::{
    platform::required_gpu_features, DataBuffer, RenderContext, ShaderStage, TextureHandler,
    TextureId, MAX_TEXTURE_ATLAS_COUNT,
};

pub trait AsBufferBinding {
    fn id() -> Uid
    where
        Self: Sized,
    {
        let typename = type_name::<Self>();
        generate_uid_from_string(typename)
    }
    fn mark_as_dirty(&self, render_context: &RenderContext)
    where
        Self: Sized,
    {
        render_context.mark_buffer_as_dirty::<Self>();
    }
    fn size(&self) -> u64;
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut DataBuffer);
}

#[derive(PartialEq, Eq)]
pub enum BindingState {
    Changed,
    Bound,
    Error,
}

#[derive(Clone)]
enum BindingType {
    Buffer(Uid),
    DefaultSampler,
    DepthSampler,
    TextureArray(Box<[TextureId; MAX_TEXTURE_ATLAS_COUNT as usize]>),
}
#[derive(Default)]
pub struct BindingData {
    bind_group_layout: Vec<wgpu::BindGroupLayout>,
    bind_group: Vec<wgpu::BindGroup>,
    binding_types: Vec<Vec<BindingType>>,
    bind_group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    is_dirty: bool,
}

impl BindingData {
    pub fn bind_groups(&self) -> &[wgpu::BindGroup] {
        self.bind_group.as_slice()
    }
    pub fn bind_group_layouts(&self) -> &[wgpu::BindGroupLayout] {
        self.bind_group_layout.as_slice()
    }

    fn create_group_and_binding_index(&mut self, group_index: usize) {
        if group_index >= self.bind_group_layout_entries.len() {
            self.bind_group_layout_entries
                .resize(group_index as usize + 1, Default::default());
            self.binding_types
                .resize(group_index as usize + 1, Default::default());
        }
    }

    pub fn add_uniform_data<T>(
        &mut self,
        render_context: &RenderContext,
        group_index: usize,
        binding_index: usize,
        data: &T,
        stage: ShaderStage,
    ) -> &mut Self
    where
        T: AsBufferBinding,
    {
        self.create_group_and_binding_index(group_index);

        if binding_index >= self.bind_group_layout_entries[group_index].len() {
            let layout_entry = wgpu::BindGroupLayoutEntry {
                binding: binding_index as _,
                visibility: stage.into(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            self.bind_group_layout_entries[group_index].push(layout_entry);
            self.is_dirty = true;
        }

        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let id = render_context.bind_buffer(data, usage);

        if binding_index >= self.binding_types[group_index].len() {
            self.binding_types[group_index].push(BindingType::Buffer(id));
            self.is_dirty = true;
        }
        self
    }

    pub fn add_storage_data<T>(
        &mut self,
        render_context: &RenderContext,
        group_index: usize,
        binding_index: usize,
        data: &T,
        stage: ShaderStage,
        read_only: bool,
    ) -> &mut Self
    where
        T: AsBufferBinding,
    {
        self.create_group_and_binding_index(group_index);

        if binding_index >= self.bind_group_layout_entries[group_index].len() {
            let layout_entry = wgpu::BindGroupLayoutEntry {
                binding: binding_index as _,
                visibility: stage.into(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            self.bind_group_layout_entries[group_index].push(layout_entry);
            self.is_dirty = true;
        }

        let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let id = render_context.bind_buffer(data, usage);

        if binding_index >= self.binding_types[group_index].len() {
            self.binding_types[group_index].push(BindingType::Buffer(id));
            self.is_dirty = true;
        }
        self
    }

    pub fn add_textures_data(
        &mut self,
        group_index: usize,
        texture_handler: &TextureHandler,
        render_target: Option<&TextureId>,
        depth_target: Option<&TextureId>,
        stage: ShaderStage,
    ) -> &mut Self {
        self.create_group_and_binding_index(group_index);

        if self.bind_group_layout_entries[group_index].is_empty() {
            self.bind_group_layout_entries[group_index].push(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: stage.into(),
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            self.is_dirty = true;
        }
        if self.bind_group_layout_entries[group_index].len() < 2 {
            self.bind_group_layout_entries[group_index].push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: stage.into(),
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            });
            self.is_dirty = true;
        }
        if self.binding_types[group_index].is_empty() {
            self.binding_types[group_index].push(BindingType::DefaultSampler);
            self.is_dirty = true;
        }
        if self.binding_types[group_index].len() < 2 {
            self.binding_types[group_index].push(BindingType::DepthSampler);
            self.is_dirty = true;
        }

        let mut bind_group_layout_count = 2;
        if required_gpu_features().contains(wgpu::Features::TEXTURE_BINDING_ARRAY) {
            if self.bind_group_layout_entries[group_index].len() < 3 {
                self.bind_group_layout_entries[group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: {
                        bind_group_layout_count += 1;
                        (bind_group_layout_count - 1) as _
                    },
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(MAX_TEXTURE_ATLAS_COUNT),
                });
                self.is_dirty = true;
            }
        } else if self.bind_group_layout_entries[group_index].len()
            < (2 + MAX_TEXTURE_ATLAS_COUNT as usize)
        {
            (0..MAX_TEXTURE_ATLAS_COUNT).for_each(|_| {
                self.bind_group_layout_entries[group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: {
                        bind_group_layout_count += 1;
                        (bind_group_layout_count - 1) as _
                    },
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                });
            });
            self.is_dirty = true;
        }

        let mut first_valid_texture = None;
        let mut textures = [TextureId::default(); MAX_TEXTURE_ATLAS_COUNT as usize];
        let textures_atlas = texture_handler.textures_atlas();
        let num_textures = textures_atlas.len();

        for i in 0..MAX_TEXTURE_ATLAS_COUNT as usize {
            if first_valid_texture.is_none() {
                first_valid_texture = Some(textures_atlas[i].texture_id());
            }
            let mut use_default = false;
            if i >= num_textures {
                use_default = true;
            } else {
                if let Some(id) = render_target {
                    if textures_atlas[i].texture_id() == id {
                        use_default = true;
                    }
                }
                if let Some(id) = depth_target {
                    if textures_atlas[i].texture_id() == id {
                        use_default = true;
                    }
                }
            }
            if use_default {
                textures[i] = **first_valid_texture.as_ref().unwrap();
            } else {
                textures[i] = *textures_atlas[i].texture_id();
            }
        }

        if self.binding_types[group_index].len() < 3 {
            self.binding_types[group_index].push(BindingType::TextureArray(Box::new(textures)));
            self.is_dirty = true;
        } else if let BindingType::TextureArray(old_textures) = &self.binding_types[group_index][2]
        {
            old_textures.iter().enumerate().for_each(|(index, id)| {
                if textures[index] != *id {
                    self.is_dirty = true;
                }
            });
        }

        self
    }

    pub fn send_to_gpu(&mut self, render_context: &RenderContext) -> BindingState {
        inox_profiler::scoped_profile!("binding_data::send_to_gpu");
        if !self.is_dirty && !self.bind_group.is_empty() && !self.bind_group_layout.is_empty() {
            return BindingState::Bound;
        }

        let typename = std::any::type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();

        self.bind_group_layout.clear();
        self.bind_group.clear();

        self.bind_group_layout_entries.iter().enumerate().for_each(
            |(index, bind_group_layout_entry)| {
                let label = format!("{} bind group layout {}", typename, index);

                let data_bind_group_layout = render_context.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: bind_group_layout_entry.as_slice(),
                        label: Some(label.as_str()),
                    },
                );
                self.bind_group_layout.push(data_bind_group_layout);
            },
        );

        self.binding_types
            .iter()
            .enumerate()
            .for_each(|(group_index, binding_type_array)| {
                let label = format!("{} bind group {}", typename, group_index);

                let mut textures_view = Vec::new();
                binding_type_array.iter().for_each(|binding_type| {
                    if let BindingType::TextureArray(textures) = binding_type {
                        textures.iter().for_each(|id| {
                            if let Some(texture) =
                                render_context.texture_handler.get_texture_atlas(id)
                            {
                                textures_view.push(texture.texture());
                            }
                        });
                    }
                });
                let bind_data_buffer = render_context.bind_data_buffer.read().unwrap();
                let mut bind_group = Vec::new();
                binding_type_array
                    .iter()
                    .enumerate()
                    .for_each(|(index, binding_type)| match binding_type {
                        BindingType::Buffer(id) => {
                            if let Some(entry) = bind_data_buffer.get(id) {
                                let entry = wgpu::BindGroupEntry {
                                    binding: index as _,
                                    resource: entry.0.gpu_buffer().unwrap().as_entire_binding(),
                                };
                                bind_group.push(entry);
                            }
                        }
                        BindingType::DefaultSampler => {
                            bind_group.push(wgpu::BindGroupEntry {
                                binding: index as _,
                                resource: wgpu::BindingResource::Sampler(
                                    render_context.texture_handler.default_sampler(),
                                ),
                            });
                        }
                        BindingType::DepthSampler => {
                            bind_group.push(wgpu::BindGroupEntry {
                                binding: index as _,
                                resource: wgpu::BindingResource::Sampler(
                                    render_context.texture_handler.depth_sampler(),
                                ),
                            });
                        }
                        BindingType::TextureArray(_) => {
                            if required_gpu_features()
                                .contains(wgpu::Features::TEXTURE_BINDING_ARRAY)
                            {
                                bind_group.push(wgpu::BindGroupEntry {
                                    binding: index as _,
                                    resource: wgpu::BindingResource::TextureViewArray({
                                        textures_view.as_slice()
                                    }),
                                });
                            } else {
                                (0..MAX_TEXTURE_ATLAS_COUNT).for_each(|i| {
                                    bind_group.push(wgpu::BindGroupEntry {
                                        binding: index as u32 + i,
                                        resource: wgpu::BindingResource::TextureView(
                                            textures_view[i as usize],
                                        ),
                                    });
                                });
                            }
                        }
                    });
                let data_bind_group =
                    render_context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.bind_group_layout[group_index],
                            entries: bind_group.as_slice(),
                            label: Some(label.as_str()),
                        });
                self.bind_group.push(data_bind_group);
            });

        self.is_dirty = false;
        BindingState::Changed
    }
}
