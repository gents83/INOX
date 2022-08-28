use std::num::NonZeroU32;

use inox_resources::Resource;

use crate::{
    platform::required_gpu_features, AsBinding, BindingDataBuffer, BufferId, RenderContext,
    RenderCoreContext, ShaderStage, Texture, TextureHandler, TextureId, MAX_TEXTURE_ATLAS_COUNT,
};

const DEBUG_BINDINGS: bool = false;

pub struct BindingInfo {
    pub group_index: usize,
    pub binding_index: usize,
    pub stage: ShaderStage,
    pub read_only: bool,
    pub cpu_accessible: bool,
    pub is_indirect: bool,
    pub is_vertex: bool,
    pub is_index: bool,
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
            is_vertex: false,
            is_index: false,
        }
    }
}

#[derive(Clone)]
enum BindingType {
    Buffer(usize, BufferId, BufferId),
    DefaultSampler(usize),
    DepthTexture(usize, TextureId),
    TextureArray(usize, Box<[TextureId; MAX_TEXTURE_ATLAS_COUNT as usize]>),
    StorageTextures(usize, Vec<TextureId>),
}

#[derive(Default)]
pub struct BindingData {
    bind_group_layout: Vec<wgpu::BindGroupLayout>,
    bind_group: Vec<wgpu::BindGroup>,
    binding_types: Vec<Vec<BindingType>>,
    bind_group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    vertex_buffers: Vec<BufferId>,
    index_buffer: Option<BufferId>,
    is_layout_changed: bool,
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
        let data_id = data.id();
        let usage =
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
        let (is_changed, _buffer_id) =
            binding_data_buffer.bind_buffer(data_id, data, usage, render_core_context);

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Set VertexBuffer[{}] = {:?} - Changed {:?}",
                index,
                data_id,
                is_changed
            );
        }

        if index <= self.vertex_buffers.len() {
            self.vertex_buffers.resize(index + 1, BufferId::default());
        }
        self.vertex_buffers[index] = data_id;

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
        let data_id = data.id();
        let usage =
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX;
        let (is_changed, _buffer_id) =
            binding_data_buffer.bind_buffer(data_id, data, usage, render_core_context);

        if DEBUG_BINDINGS {
            inox_log::debug_log!("Set IndexBuffer = {:?} - Changed {:?}", data_id, is_changed);
        }
        self.index_buffer = Some(data_id);

        self
    }

    pub fn vertex_buffers_count(&self) -> usize {
        self.vertex_buffers.len()
    }
    pub fn vertex_buffer(&self, index: usize) -> &BufferId {
        &self.vertex_buffers[index]
    }
    pub fn index_buffer(&self) -> &Option<BufferId> {
        &self.index_buffer
    }

    pub fn add_uniform_buffer<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        binding_data_buffer: &BindingDataBuffer,
        data: &mut T,
        info: BindingInfo,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        if data.size() == 0 {
            return self;
        }
        let data_id = data.id();
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let (is_changed, buffer_id) =
            binding_data_buffer.bind_buffer(data_id, data, usage, render_core_context);

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add UniformBuffer[{}][{}] = {:?} with {:?} - Changed {:?}",
                info.group_index,
                info.binding_index,
                data_id,
                buffer_id,
                is_changed
            );
        }
        self.bind_uniform_buffer(data_id, &buffer_id, info);
        self
    }

    fn bind_uniform_buffer(
        &mut self,
        data_id: BufferId,
        buffer_id: &BufferId,
        info: BindingInfo,
    ) -> &mut Self {
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
            self.is_layout_changed = true;
        }

        if info.binding_index >= self.binding_types[info.group_index].len() {
            self.binding_types[info.group_index].push(BindingType::Buffer(
                info.binding_index,
                data_id,
                *buffer_id,
            ));
        } else if let BindingType::Buffer(_, old_data_id, old_buffer_id) =
            &mut self.binding_types[info.group_index][info.binding_index]
        {
            if *old_buffer_id != *buffer_id || *old_data_id != data_id {
                *old_data_id = data_id;
                *old_buffer_id = *buffer_id;
            }
        }
        self
    }

    pub fn add_storage_buffer<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        binding_data_buffer: &BindingDataBuffer,
        data: &mut T,
        info: BindingInfo,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        if data.size() == 0 {
            return self;
        }

        let data_id = data.id();
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
        if info.is_index {
            usage |= wgpu::BufferUsages::INDEX;
        }
        if info.is_vertex {
            usage |= wgpu::BufferUsages::VERTEX;
        }
        let (is_changed, buffer_id) =
            binding_data_buffer.bind_buffer(data_id, data, usage, render_core_context);

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add StorageBuffer[{}][{}] = {:?} with {:?} - Changed {:?}",
                info.group_index,
                info.binding_index,
                data_id,
                buffer_id,
                is_changed
            );
        }
        self.bind_storage_buffer(data_id, &buffer_id, info);

        self
    }

    fn bind_storage_buffer(
        &mut self,
        data_id: BufferId,
        buffer_id: &BufferId,
        info: BindingInfo,
    ) -> &mut Self {
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
            self.is_layout_changed = true;
        }

        if info.binding_index >= self.binding_types[info.group_index].len() {
            self.binding_types[info.group_index].push(BindingType::Buffer(
                info.binding_index,
                data_id,
                *buffer_id,
            ));
        } else if let BindingType::Buffer(_, old_data_id, old_buffer_id) =
            &mut self.binding_types[info.group_index][info.binding_index]
        {
            if *old_buffer_id != *buffer_id || *old_data_id != data_id {
                *old_data_id = data_id;
                *old_buffer_id = *buffer_id;
            }
        }
        self
    }

    pub fn add_storage_textures(
        &mut self,
        render_context: &RenderContext,
        textures: Vec<&Resource<Texture>>,
        info: BindingInfo,
    ) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if self.bind_group_layout_entries[info.group_index].len() < textures.len() {
            textures.iter().enumerate().for_each(|(i, t)| {
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
                        format: t.get().format().into(),
                    },
                    count: None,
                });
            });
            self.is_layout_changed = true;
        } else if textures.is_empty() && self.bind_group_layout_entries[info.group_index].is_empty()
        {
            self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                binding: 0 as _,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::StorageTexture {
                    access: if info.read_only {
                        wgpu::StorageTextureAccess::ReadOnly
                    } else {
                        wgpu::StorageTextureAccess::ReadWrite
                    },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    format: render_context.core.config.format,
                },
                count: None,
            });
        }

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add StorageTextures [{}][{}] - NumTextures: {:?}",
                info.group_index,
                info.binding_index,
                textures.len()
            );
        }

        if self.binding_types[info.group_index].len() <= info.binding_index {
            self.binding_types[info.group_index].push(BindingType::StorageTextures(
                info.binding_index,
                textures.iter().map(|t| *t.id()).collect(),
            ));
        } else if let BindingType::StorageTextures(_, old_textures) =
            &self.binding_types[info.group_index][info.binding_index]
        {
            if old_textures
                .iter()
                .enumerate()
                .any(|(index, id)| textures[index].id() != id)
            {
                self.binding_types[info.group_index][info.binding_index] =
                    BindingType::StorageTextures(
                        info.binding_index,
                        textures.iter().map(|&t| *t.id()).collect(),
                    );
            }
        }
        self
    }

    pub fn add_sampler_and_textures(
        &mut self,
        texture_handler: &TextureHandler,
        render_targets: Vec<&TextureId>,
        depth_target: Option<&TextureId>,
        info: BindingInfo,
    ) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if self.bind_group_layout_entries[info.group_index].is_empty() {
            self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                binding: info.binding_index as _,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            self.is_layout_changed = true;
        }
        if self.binding_types[info.group_index].is_empty() {
            self.binding_types[info.group_index]
                .push(BindingType::DefaultSampler(info.binding_index));
        }

        let mut first_valid_texture = None;
        let mut textures = [TextureId::default(); MAX_TEXTURE_ATLAS_COUNT as usize];
        let textures_atlas = texture_handler.textures_atlas();
        let num_textures = textures_atlas.len();

        for i in 0..MAX_TEXTURE_ATLAS_COUNT as usize {
            if first_valid_texture.is_none() {
                first_valid_texture = Some(i);
            }
            let mut index = i;
            if i >= num_textures
                || textures_atlas[index]
                    .texture_format()
                    .describe()
                    .sample_type
                    != (wgpu::TextureSampleType::Float { filterable: true })
                || render_targets
                    .iter()
                    .any(|&id| textures_atlas[i].texture_id() == id)
            {
                index = first_valid_texture.unwrap();
            } else if let Some(id) = depth_target {
                if textures_atlas[i].texture_id() == id {
                    index = first_valid_texture.unwrap();
                }
            }
            textures[i] = *textures_atlas[index].texture_id();
        }

        let textures_bind_group_layout_index = info.binding_index + 1;
        let mut bind_group_layout_count = textures_bind_group_layout_index;
        if required_gpu_features().contains(wgpu::Features::TEXTURE_BINDING_ARRAY) {
            if self.bind_group_layout_entries[info.group_index].len()
                <= textures_bind_group_layout_index
            {
                self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: bind_group_layout_count as _,
                    visibility: info.stage.into(),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(MAX_TEXTURE_ATLAS_COUNT),
                });
                self.is_layout_changed = true;
            }
        } else if self.bind_group_layout_entries[info.group_index].len()
            < (textures_bind_group_layout_index + MAX_TEXTURE_ATLAS_COUNT as usize)
        {
            (0..MAX_TEXTURE_ATLAS_COUNT).for_each(|_| {
                self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: bind_group_layout_count as _,
                    visibility: info.stage.into(),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                });
                bind_group_layout_count += 1;
            });
            self.is_layout_changed = true;
        }

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add Textures [{}][{}] - NumTextures: {:?}",
                info.group_index,
                info.binding_index,
                textures.len()
            );
        }

        if self.binding_types[info.group_index].len() <= textures_bind_group_layout_index {
            self.binding_types[info.group_index].push(BindingType::TextureArray(
                textures_bind_group_layout_index,
                Box::new(textures),
            ));
        } else if let BindingType::TextureArray(_, old_textures) =
            &self.binding_types[info.group_index][textures_bind_group_layout_index]
        {
            if old_textures
                .iter()
                .enumerate()
                .any(|(index, id)| textures[index] != *id)
            {
                self.binding_types[info.group_index][textures_bind_group_layout_index] =
                    BindingType::TextureArray(textures_bind_group_layout_index, Box::new(textures));
            }
        }

        self
    }

    pub fn add_depth_texture(
        &mut self,
        texture_handler: &TextureHandler,
        depth_target: &TextureId,
        info: BindingInfo,
    ) -> &mut Self {
        self.create_group_and_binding_index(info.group_index);

        if self.bind_group_layout_entries[info.group_index].len() <= info.binding_index {
            let textures_atlas = texture_handler.textures_atlas();
            textures_atlas.iter().for_each(|t| {
                if t.texture_id() == depth_target {
                    self.bind_group_layout_entries[info.group_index].push(
                        wgpu::BindGroupLayoutEntry {
                            binding: info.binding_index as _,
                            visibility: info.stage.into(),
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                    );
                }
            });
            self.binding_types[info.group_index]
                .push(BindingType::DepthTexture(info.binding_index, *depth_target));
            self.is_layout_changed = true;
        }
        if self.binding_types[info.group_index].len() > info.binding_index {
            if let BindingType::DepthTexture(_, id) =
                &self.binding_types[info.group_index][info.binding_index]
            {
                if id != depth_target {
                    self.binding_types[info.group_index][info.binding_index] =
                        BindingType::DepthTexture(info.binding_index, *depth_target);
                    self.is_layout_changed = true;
                }
            }
        }

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add Depth Texture [{}][{}] ",
                info.group_index,
                info.binding_index,
            );
        }

        self
    }

    pub fn send_to_gpu(&mut self, render_context: &RenderContext, pass_name: &str) {
        inox_profiler::scoped_profile!("binding_data::send_to_gpu");

        if DEBUG_BINDINGS {
            inox_log::debug_log!("Sending to gpu BindingData of {}", pass_name);
        }

        if self.is_layout_changed {
            self.bind_group_layout.clear();
            self.bind_group_layout_entries.iter().enumerate().for_each(
                |(index, bind_group_layout_entry)| {
                    let label = format!("{} bind group layout {}", pass_name, index);

                    let data_bind_group_layout = render_context
                        .core
                        .device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            entries: bind_group_layout_entry.as_slice(),
                            label: Some(label.as_str()),
                        });
                    self.bind_group_layout.push(data_bind_group_layout);
                },
            );
            self.is_layout_changed = false;
        }

        self.bind_group.clear();
        self.binding_types
            .iter()
            .enumerate()
            .for_each(|(group_index, binding_type_array)| {
                let label = format!("{} bind group {}", pass_name, group_index);

                let mut textures_view = Vec::new();
                let mut storage_textures = Vec::new();
                binding_type_array.iter().for_each(|binding_type| {
                    if let BindingType::TextureArray(_, textures) = binding_type {
                        textures.iter().for_each(|id| {
                            if let Some(texture) =
                                render_context.texture_handler.get_texture_atlas(id)
                            {
                                textures_view.push(texture.texture());
                            }
                        });
                    } else if let BindingType::StorageTextures(_, textures) = binding_type {
                        if textures.is_empty() {
                            storage_textures.push(render_context.surface_view.as_ref().unwrap());
                        } else {
                            textures.iter().for_each(|id| {
                                if let Some(texture) =
                                    render_context.texture_handler.get_texture_atlas(id)
                                {
                                    storage_textures.push(texture.texture());
                                }
                            });
                        }
                    }
                });
                let bind_data_buffer = render_context.binding_data_buffer.buffers.read().unwrap();
                let mut bind_group = Vec::new();
                binding_type_array
                    .iter()
                    .for_each(|binding_type| match binding_type {
                        BindingType::Buffer(binding_index, data_id, buffer_id) => {
                            if DEBUG_BINDINGS {
                                inox_log::debug_log!(
                                    "Binding Buffer[{}][{}] = {:?} with {:?}",
                                    group_index,
                                    binding_index,
                                    data_id,
                                    buffer_id
                                );
                            }
                            if let Some(buffer) = bind_data_buffer.get(data_id) {
                                if buffer.gpu_buffer().is_some() && buffer.size() != 0 {
                                    let entry = wgpu::BindGroupEntry {
                                        binding: *binding_index as _,
                                        resource: buffer.gpu_buffer().unwrap().as_entire_binding(),
                                    };
                                    bind_group.push(entry);
                                } else if DEBUG_BINDINGS {
                                    inox_log::debug_log!(
                                        "Binding Buffer[{}][{}] = {:?} with {:?} but buffer is empty",
                                        group_index,
                                        binding_index,
                                        data_id,
                                        buffer_id
                                    );
                                }
                            } else if DEBUG_BINDINGS {
                                inox_log::debug_log!(
                                    "Binding Buffer[{}][{}] = {:?} with {:?} but buffer is not found",
                                    group_index,
                                    binding_index,
                                    data_id,
                                    buffer_id
                                );
                            }
                        }
                        BindingType::DefaultSampler(binding_index) => {
                            bind_group.push(wgpu::BindGroupEntry {
                                binding: *binding_index as _,
                                resource: wgpu::BindingResource::Sampler(
                                    render_context.texture_handler.default_sampler(),
                                ),
                            });
                        }
                        BindingType::DepthTexture(binding_index, id) => {
                            if let Some(texture) =
                                render_context.texture_handler.get_texture_atlas(id)
                            {
                                bind_group.push(wgpu::BindGroupEntry {
                                    binding: *binding_index as _,
                                    resource: wgpu::BindingResource::TextureView(texture.texture()),
                                });
                            }
                        }
                        BindingType::TextureArray(binding_index, _) => {
                            if DEBUG_BINDINGS {
                                inox_log::debug_log!(
                                    "Binding Textures[{}][{}] - NumTexturesView {:?}",
                                    group_index,
                                    binding_index,
                                    textures_view.len()
                                );
                            }
                            if required_gpu_features()
                                .contains(wgpu::Features::TEXTURE_BINDING_ARRAY)
                            {
                                bind_group.push(wgpu::BindGroupEntry {
                                    binding: *binding_index as _,
                                    resource: wgpu::BindingResource::TextureViewArray({
                                        textures_view.as_slice()
                                    }),
                                });
                            } else {
                                (0..MAX_TEXTURE_ATLAS_COUNT).for_each(|i| {
                                    bind_group.push(wgpu::BindGroupEntry {
                                        binding: *binding_index as u32 + i,
                                        resource: wgpu::BindingResource::TextureView(
                                            textures_view[i as usize],
                                        ),
                                    });
                                });
                            }
                        }
                        BindingType::StorageTextures(binding_index, _) => {
                            if DEBUG_BINDINGS {
                                inox_log::debug_log!(
                                    "Binding StorageTextures[{}][{}] - NumTexturesView {:?}",
                                    group_index,
                                    binding_index,
                                    storage_textures.len()
                                );
                            }
                            storage_textures
                                .iter()
                                .enumerate()
                                .for_each(|(i, &texture)| {
                                    bind_group.push(wgpu::BindGroupEntry {
                                        binding: *binding_index as u32 + i as u32,
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
    }
}
