use std::num::NonZeroU32;

use inox_resources::ResourceId;
use inox_uid::{Uid, INVALID_UID};

use crate::{
    platform::required_gpu_features, AsBinding, BindingDataBuffer, RenderContext,
    RenderCoreContext, ShaderStage, TextureHandler, TextureId, MAX_TEXTURE_ATLAS_COUNT,
};
pub struct BindingInfo {
    pub group_index: usize,
    pub binding_index: usize,
    pub stage: ShaderStage,
    pub read_only: bool,
    pub cpu_accessible: bool,
    pub is_indirect: bool,
}

impl Default for BindingInfo {
    fn default() -> Self {
        Self {
            group_index: 0,
            binding_index: 0,
            stage: ShaderStage::VertexAndFragment,
            read_only: true,
            cpu_accessible: false,
            is_indirect: false,
        }
    }
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
    UnfilteredSampler,
    DepthSampler,
    TextureArray(Box<[TextureId; MAX_TEXTURE_ATLAS_COUNT as usize]>),
    StorageTextures(Vec<TextureId>),
}

#[derive(Default)]
pub struct BindingData {
    bind_group_layout: Vec<wgpu::BindGroupLayout>,
    bind_group: Vec<wgpu::BindGroup>,
    binding_types: Vec<Vec<BindingType>>,
    bind_group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    vertex_buffers: Vec<ResourceId>,
    index_buffer: ResourceId,
    is_dirty: bool,
}

impl BindingData {
    pub fn bind_groups(&self) -> &[wgpu::BindGroup] {
        self.bind_group.as_slice()
    }
    pub fn bind_group_layouts(&self) -> &[wgpu::BindGroupLayout] {
        self.bind_group_layout.as_slice()
    }
    pub fn mark_as_dirty(&mut self) {
        self.is_dirty = true;
    }
    fn create_group_and_binding_index(&mut self, group_index: usize) {
        if group_index >= self.bind_group_layout_entries.len() {
            self.bind_group_layout_entries
                .resize(group_index as usize + 1, Default::default());
            self.binding_types
                .resize(group_index as usize + 1, Default::default());
        }
    }
    pub fn set_vertex_buffer<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        binding_data_buffer: &BindingDataBuffer,
        index: usize,
        data: &mut T,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        let usage =
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
        let (id, is_changed) = binding_data_buffer.bind_buffer(data, usage, render_core_context);

        if index <= self.vertex_buffers.len() {
            self.vertex_buffers.resize(index + 1, INVALID_UID);
        }
        self.vertex_buffers[index] = id;

        if is_changed {
            self.is_dirty = true;
        }
        self
    }
    pub fn set_index_buffer<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        binding_data_buffer: &BindingDataBuffer,
        data: &mut T,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        let usage =
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX;
        let (id, is_changed) = binding_data_buffer.bind_buffer(data, usage, render_core_context);

        self.index_buffer = id;

        if is_changed {
            self.is_dirty = true;
        }
        self
    }

    pub fn vertex_buffers_count(&self) -> usize {
        self.vertex_buffers.len()
    }
    pub fn vertex_buffer(&self, index: usize) -> &ResourceId {
        &self.vertex_buffers[index]
    }
    pub fn index_buffer(&self) -> &ResourceId {
        &self.index_buffer
    }

    pub fn add_uniform_data<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        binding_data_buffer: &BindingDataBuffer,
        data: &mut T,
        info: BindingInfo,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let (id, is_changed) = binding_data_buffer.bind_buffer(data, usage, render_core_context);

        self.bind_uniform_buffer(&id, info);
        if is_changed {
            self.is_dirty = true;
        }
        self
    }

    pub fn bind_uniform_buffer(&mut self, id: &Uid, info: BindingInfo) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if info.binding_index >= self.bind_group_layout_entries[info.group_index].len() {
            let layout_entry = wgpu::BindGroupLayoutEntry {
                binding: info.binding_index as _,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            self.bind_group_layout_entries[info.group_index].push(layout_entry);
            self.is_dirty = true;
        }

        if info.binding_index >= self.binding_types[info.group_index].len() {
            self.binding_types[info.group_index].push(BindingType::Buffer(*id));
            self.is_dirty = true;
        } else if let BindingType::Buffer(old_id) =
            &mut self.binding_types[info.group_index][info.binding_index]
        {
            if *old_id != *id {
                *old_id = *id;
                self.is_dirty = true;
            }
        }
        self
    }

    pub fn add_storage_data<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        binding_data_buffer: &BindingDataBuffer,
        data: &mut T,
        info: BindingInfo,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        let mut usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        if !info.read_only {
            usage |= wgpu::BufferUsages::COPY_SRC;
        }
        if info.cpu_accessible {
            usage |= wgpu::BufferUsages::MAP_READ;
        }
        if info.is_indirect {
            usage |= wgpu::BufferUsages::INDIRECT;
        }
        let (id, is_changed) = binding_data_buffer.bind_buffer(data, usage, render_core_context);

        self.bind_storage_buffer(&id, info);

        if is_changed {
            self.is_dirty = true;
        }
        self
    }

    pub fn bind_storage_buffer(&mut self, id: &Uid, info: BindingInfo) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if info.binding_index >= self.bind_group_layout_entries[info.group_index].len() {
            let layout_entry = wgpu::BindGroupLayoutEntry {
                binding: info.binding_index as _,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: info.read_only,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            self.bind_group_layout_entries[info.group_index].push(layout_entry);
            self.is_dirty = true;
        }

        if info.binding_index >= self.binding_types[info.group_index].len() {
            self.binding_types[info.group_index].push(BindingType::Buffer(*id));
            self.is_dirty = true;
        } else if let BindingType::Buffer(old_id) =
            &mut self.binding_types[info.group_index][info.binding_index]
        {
            if *old_id != *id {
                *old_id = *id;
                self.is_dirty = true;
            }
        }
        self
    }

    pub fn add_storage_textures(
        &mut self,
        texture_handler: &TextureHandler,
        textures: &[&TextureId],
        info: BindingInfo,
    ) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if self.bind_group_layout_entries[info.group_index].len() < textures.len() {
            textures.iter().enumerate().for_each(|(i, id)| {
                self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: i as _,
                    visibility: info.stage.into(),
                    ty: wgpu::BindingType::StorageTexture {
                        access: if info.read_only {
                            wgpu::StorageTextureAccess::ReadOnly
                        } else {
                            wgpu::StorageTextureAccess::ReadWrite
                        },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        format: if let Some(textures_atlas) = texture_handler.get_texture_atlas(id)
                        {
                            *textures_atlas.texture_format()
                        } else {
                            wgpu::TextureFormat::Rgba32Float
                        },
                    },
                    count: None,
                });
            });
            self.is_dirty = true;
        }

        if self.binding_types[info.group_index].len() <= info.binding_index {
            self.binding_types[info.group_index].push(BindingType::StorageTextures(
                textures.iter().map(|&id| *id).collect(),
            ));
            self.is_dirty = true;
        } else if let BindingType::StorageTextures(old_textures) =
            &self.binding_types[info.group_index][info.binding_index]
        {
            old_textures.iter().enumerate().for_each(|(index, id)| {
                if textures[index] != id {
                    self.is_dirty = true;
                }
            });
            if self.is_dirty {
                self.binding_types[info.group_index][info.binding_index] =
                    BindingType::StorageTextures(textures.iter().map(|&id| *id).collect());
            }
        }
        self
    }

    pub fn add_textures_data(
        &mut self,
        texture_handler: &TextureHandler,
        render_target: Option<&TextureId>,
        depth_target: Option<&TextureId>,
        info: BindingInfo,
    ) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if self.bind_group_layout_entries[info.group_index].is_empty() {
            self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            self.is_dirty = true;
        }
        if self.bind_group_layout_entries[info.group_index].len() < 2 {
            self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            });
            self.is_dirty = true;
        }
        if self.bind_group_layout_entries[info.group_index].len() < 3 {
            self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            });
            self.is_dirty = true;
        }
        if self.binding_types[info.group_index].is_empty() {
            self.binding_types[info.group_index].push(BindingType::DefaultSampler);
            self.is_dirty = true;
        }
        if self.binding_types[info.group_index].len() < 2 {
            self.binding_types[info.group_index].push(BindingType::UnfilteredSampler);
            self.is_dirty = true;
        }
        if self.binding_types[info.group_index].len() < 3 {
            self.binding_types[info.group_index].push(BindingType::DepthSampler);
            self.is_dirty = true;
        }

        let textures_bind_group_layout_index = 3;
        let mut bind_group_layout_count = textures_bind_group_layout_index;
        if required_gpu_features().contains(wgpu::Features::TEXTURE_BINDING_ARRAY) {
            if self.bind_group_layout_entries[info.group_index].len()
                <= textures_bind_group_layout_index
            {
                self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: {
                        bind_group_layout_count += 1;
                        (bind_group_layout_count - 1) as _
                    },
                    visibility: info.stage.into(),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(MAX_TEXTURE_ATLAS_COUNT),
                });
                self.is_dirty = true;
            }
        } else if self.bind_group_layout_entries[info.group_index].len()
            <= (textures_bind_group_layout_index + MAX_TEXTURE_ATLAS_COUNT as usize)
        {
            (0..MAX_TEXTURE_ATLAS_COUNT).for_each(|_| {
                self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: {
                        bind_group_layout_count += 1;
                        (bind_group_layout_count - 1) as _
                    },
                    visibility: info.stage.into(),
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

        if self.binding_types[info.group_index].len() <= textures_bind_group_layout_index {
            self.binding_types[info.group_index]
                .push(BindingType::TextureArray(Box::new(textures)));
            self.is_dirty = true;
        } else if let BindingType::TextureArray(old_textures) =
            &self.binding_types[info.group_index][textures_bind_group_layout_index]
        {
            old_textures.iter().enumerate().for_each(|(index, id)| {
                if textures[index] != *id {
                    self.is_dirty = true;
                }
            });
            if self.is_dirty {
                self.binding_types[info.group_index][textures_bind_group_layout_index] =
                    BindingType::TextureArray(Box::new(textures));
            }
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

                let data_bind_group_layout = render_context.core.device.create_bind_group_layout(
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
                let mut storage_textures = Vec::new();
                binding_type_array.iter().for_each(|binding_type| {
                    if let BindingType::TextureArray(textures) = binding_type {
                        textures.iter().for_each(|id| {
                            if let Some(texture) =
                                render_context.texture_handler.get_texture_atlas(id)
                            {
                                textures_view.push(texture.texture());
                            }
                        });
                    } else if let BindingType::StorageTextures(textures) = binding_type {
                        textures.iter().for_each(|id| {
                            if let Some(texture) =
                                render_context.texture_handler.get_texture_atlas(id)
                            {
                                storage_textures.push(texture.texture());
                            }
                        });
                    }
                });
                let bind_data_buffer = render_context.binding_data_buffer.buffers.read().unwrap();
                let mut bind_group = Vec::new();
                binding_type_array
                    .iter()
                    .enumerate()
                    .for_each(|(index, binding_type)| match binding_type {
                        BindingType::Buffer(id) => {
                            if let Some(buffer) = bind_data_buffer.get(id) {
                                debug_assert!(
                                    buffer.gpu_buffer().is_some(),
                                    "Buffer is not uploaded to gpu already"
                                );
                                debug_assert!(
                                    buffer.size() != 0,
                                    "Group {} Buffer {} has size 0",
                                    group_index,
                                    index
                                );
                                let entry = wgpu::BindGroupEntry {
                                    binding: index as _,
                                    resource: buffer.gpu_buffer().unwrap().as_entire_binding(),
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
                        BindingType::UnfilteredSampler => {
                            bind_group.push(wgpu::BindGroupEntry {
                                binding: index as _,
                                resource: wgpu::BindingResource::Sampler(
                                    render_context.texture_handler.unfiltered_sampler(),
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
                        BindingType::StorageTextures(_) => {
                            storage_textures
                                .iter()
                                .enumerate()
                                .for_each(|(i, &texture)| {
                                    bind_group.push(wgpu::BindGroupEntry {
                                        binding: index as u32 + i as u32,
                                        resource: wgpu::BindingResource::TextureView(texture),
                                    });
                                });
                        }
                    });
                let data_bind_group =
                    render_context
                        .core
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
