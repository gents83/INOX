use std::num::NonZeroU32;

use inox_bitmask::bitmask;
use inox_serialize::{Deserialize, Serialize};

use crate::{
    platform::required_gpu_features, AsBinding, BindingDataBufferRc, BufferId, RenderContext,
    RenderCoreContextRc, ShaderStage, TextureHandlerRc, TextureId, MAX_TEXTURE_ATLAS_COUNT,
};

const DEBUG_BINDINGS: bool = false;

#[bitmask]
pub enum BindingFlags {
    Read,
    Write,
    ReadWrite,
    CPUAccessible,
    Indirect,
    Vertex,
    Index,
    Storage,
}

impl From<BindingFlags> for wgpu::BufferUsages {
    fn from(val: BindingFlags) -> Self {
        let mut usage = wgpu::BufferUsages::empty();
        if val.contains(BindingFlags::Read) || val.contains(BindingFlags::ReadWrite) {
            usage |= wgpu::BufferUsages::COPY_SRC;
        }
        if val.contains(BindingFlags::Write) || val.contains(BindingFlags::ReadWrite) {
            usage |= wgpu::BufferUsages::COPY_DST;
        }
        if val.contains(BindingFlags::CPUAccessible) {
            usage |= wgpu::BufferUsages::MAP_READ;
        }
        if val.contains(BindingFlags::Indirect) {
            usage |= wgpu::BufferUsages::INDIRECT;
        }
        if val.contains(BindingFlags::Index) {
            usage |= wgpu::BufferUsages::INDEX;
        }
        if val.contains(BindingFlags::Vertex) {
            usage |= wgpu::BufferUsages::VERTEX;
        }
        usage
    }
}

pub struct BindingInfo {
    pub group_index: usize,
    pub binding_index: usize,
    pub stage: ShaderStage,
    pub flags: BindingFlags,
}

impl Default for BindingInfo {
    fn default() -> Self {
        Self {
            group_index: 0,
            binding_index: 0,
            stage: ShaderStage::VertexAndFragment,
            flags: BindingFlags::Read,
        }
    }
}

#[derive(Clone)]
enum BindingType {
    Buffer(usize, BufferId),
    DefaultSampler(usize),
    Texture(usize, TextureId),
    TextureArray(usize, Box<[TextureId; MAX_TEXTURE_ATLAS_COUNT as usize]>),
}

pub struct BindingData {
    binding_data_buffer: BindingDataBufferRc,
    render_core_context: RenderCoreContextRc,
    texture_handler: TextureHandlerRc,
    bind_group_layout: Vec<wgpu::BindGroupLayout>,
    bind_group: Vec<wgpu::BindGroup>,
    binding_types: Vec<Vec<BindingType>>,
    bind_group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>,
    vertex_buffers: Vec<BufferId>,
    index_buffer: Option<BufferId>,
    pass_name: String,
    is_layout_changed: bool,
    is_data_changed: bool,
}

impl BindingData {
    pub fn new(render_context: &RenderContext, pass_name: &str) -> Self {
        Self {
            binding_data_buffer: render_context.binding_data_buffer.clone(),
            render_core_context: render_context.core.clone(),
            texture_handler: render_context.texture_handler.clone(),
            bind_group_layout: Vec::default(),
            bind_group: Vec::default(),
            binding_types: Vec::default(),
            bind_group_layout_entries: Vec::default(),
            vertex_buffers: Vec::default(),
            index_buffer: None,
            is_layout_changed: false,
            is_data_changed: false,
            pass_name: pass_name.to_string(),
        }
    }
    pub fn bind_groups(&self) -> &[wgpu::BindGroup] {
        self.bind_group.as_slice()
    }
    pub fn bind_group_layouts(&self) -> &[wgpu::BindGroupLayout] {
        self.bind_group_layout.as_slice()
    }
    fn create_group_and_binding_index(&mut self, group_index: usize) {
        inox_profiler::scoped_profile!("binding_data::create_group_and_binding_index");
        if group_index >= self.bind_group_layout_entries.len() {
            self.bind_group_layout_entries
                .resize(group_index + 1, Default::default());
            self.binding_types
                .resize(group_index + 1, Default::default());
        }
    }
    pub fn set_vertex_buffer<T>(
        &mut self,
        index: usize,
        data: &mut T,
        label: Option<&str>,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        inox_profiler::scoped_profile!("binding_data::set_vertex_buffer");

        let usage =
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
        let is_changed =
            self.binding_data_buffer
                .bind_buffer(label, data, usage, &self.render_core_context);

        if DEBUG_BINDINGS {
            inox_log::debug_log!("Set VertexBuffer[{}] - Changed {:?}", index, is_changed);
        }

        if index <= self.vertex_buffers.len() {
            self.vertex_buffers.resize(index + 1, BufferId::default());
        }
        self.vertex_buffers[index] = data.id();

        self
    }
    pub fn set_index_buffer<T>(&mut self, data: &mut T, label: Option<&str>) -> &mut Self
    where
        T: AsBinding,
    {
        inox_profiler::scoped_profile!("binding_data::set_index_buffer");

        let usage =
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX;
        let is_changed =
            self.binding_data_buffer
                .bind_buffer(label, data, usage, &self.render_core_context);

        if DEBUG_BINDINGS {
            inox_log::debug_log!("Set IndexBuffer - Changed {:?}", is_changed);
        }
        self.index_buffer = Some(data.id());

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
        data: &mut T,
        label: Option<&str>,
        info: BindingInfo,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        inox_profiler::scoped_profile!("binding_data::add_uniform_buffer");

        if data.size() == 0 {
            return self;
        }
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let is_changed =
            self.binding_data_buffer
                .bind_buffer(label, data, usage, &self.render_core_context);
        self.is_data_changed |= is_changed;

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add UniformBuffer[{}][{}] with {:?} - Changed {:?}",
                info.group_index,
                info.binding_index,
                data.id(),
                is_changed
            );
        }
        self.bind_uniform_buffer(data.id(), info);
        self
    }

    fn bind_uniform_buffer(&mut self, data_id: BufferId, info: BindingInfo) -> &mut Self {
        inox_profiler::scoped_profile!("binding_data::bind_uniform_buffer");

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
            self.binding_types[info.group_index]
                .push(BindingType::Buffer(info.binding_index, data_id));
            self.is_data_changed = true;
        } else if let BindingType::Buffer(_, old_data_id) =
            &mut self.binding_types[info.group_index][info.binding_index]
        {
            if *old_data_id != data_id {
                *old_data_id = data_id;
                self.is_data_changed = true;
            }
        }
        self
    }

    pub fn add_storage_buffer<T>(
        &mut self,
        data: &mut T,
        label: Option<&str>,
        info: BindingInfo,
    ) -> &mut Self
    where
        T: AsBinding,
    {
        inox_profiler::scoped_profile!("binding_data::add_storage_buffer");

        if data.size() == 0 {
            return self;
        }

        let usage = wgpu::BufferUsages::STORAGE | info.flags.into();
        let is_changed =
            self.binding_data_buffer
                .bind_buffer(label, data, usage, &self.render_core_context);
        self.is_data_changed |= is_changed;

        if DEBUG_BINDINGS {
            inox_log::debug_log!(
                "Add StorageBuffer[{}][{}] with {:?} - Changed {:?}",
                info.group_index,
                info.binding_index,
                data.id(),
                is_changed
            );
        }
        self.bind_storage_buffer(data.id(), info);

        self
    }

    fn bind_storage_buffer(&mut self, data_id: BufferId, info: BindingInfo) -> &mut Self {
        inox_profiler::scoped_profile!("binding_data::bind_storage_buffer");

        self.create_group_and_binding_index(info.group_index);

        if info.binding_index >= self.bind_group_layout_entries[info.group_index].len() {
            let layout_entry = wgpu::BindGroupLayoutEntry {
                binding: info.binding_index as _,
                visibility: info.stage.into(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: info.flags.contains(BindingFlags::Read),
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
            self.binding_types[info.group_index]
                .push(BindingType::Buffer(info.binding_index, data_id));
            self.is_data_changed = true;
        } else if let BindingType::Buffer(_, old_data_id) =
            &mut self.binding_types[info.group_index][info.binding_index]
        {
            if *old_data_id != data_id {
                *old_data_id = data_id;
                self.is_data_changed = true;
            }
        }
        self
    }

    pub fn add_default_sampler(&mut self, info: BindingInfo) -> &mut Self {
        inox_profiler::scoped_profile!("binding_data::add_default_sampler");

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
            self.is_data_changed = true;
        }
        self
    }

    pub fn add_material_textures(&mut self, info: BindingInfo) -> &mut Self {
        inox_profiler::scoped_profile!("binding_data::add_material_textures");

        self.create_group_and_binding_index(info.group_index);

        let mut textures = [TextureId::default(); MAX_TEXTURE_ATLAS_COUNT as usize];
        {
            let texture_atlas = self.texture_handler.textures_atlas();
            let num_textures = texture_atlas.len();

            for i in 0..MAX_TEXTURE_ATLAS_COUNT as usize {
                if i < num_textures {
                    textures[i] = *texture_atlas[i].texture_id();
                } else {
                    textures[i] = *texture_atlas[0].texture_id();
                }
            }
        }

        let textures_bind_group_layout_index = info.binding_index;
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
            self.is_data_changed = true;
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
                self.is_data_changed = true;
            }
        }

        self
    }

    pub fn add_texture(&mut self, texture_id: &TextureId, info: BindingInfo) -> &mut Self {
        inox_profiler::scoped_profile!("binding_data::add_texture");

        self.create_group_and_binding_index(info.group_index);

        if self.bind_group_layout_entries[info.group_index].len() <= info.binding_index {
            let render_targets = self.texture_handler.render_targets();
            if let Some(texture) = render_targets.iter().find(|t| t.id() == texture_id) {
                let format: wgpu::TextureFormat = (*texture.format()).into();
                self.bind_group_layout_entries[info.group_index].push(wgpu::BindGroupLayoutEntry {
                    binding: info.binding_index as _,
                    visibility: info.stage.into(),
                    ty: if info.flags.contains(BindingFlags::Storage) {
                        wgpu::BindingType::StorageTexture {
                            access: if info.flags.contains(BindingFlags::Write) {
                                wgpu::StorageTextureAccess::WriteOnly
                            } else if info.flags.contains(BindingFlags::ReadWrite) {
                                wgpu::StorageTextureAccess::ReadWrite
                            } else {
                                wgpu::StorageTextureAccess::ReadOnly
                            },
                            view_dimension: if texture.layers_count() > 1 {
                                wgpu::TextureViewDimension::D2Array
                            } else {
                                wgpu::TextureViewDimension::D2
                            },
                            format,
                        }
                    } else {
                        wgpu::BindingType::Texture {
                            sample_type: format
                                .sample_type(Some(wgpu::TextureAspect::All))
                                .unwrap_or_default(),
                            view_dimension: if texture.layers_count() > 1 {
                                wgpu::TextureViewDimension::D2Array
                            } else {
                                wgpu::TextureViewDimension::D2
                            },
                            multisampled: false,
                        }
                    },
                    count: None,
                });
            }
            self.is_layout_changed = true;

            self.binding_types[info.group_index]
                .push(BindingType::Texture(info.binding_index, *texture_id));
            self.is_data_changed = true;
        }
        if self.binding_types[info.group_index].len() > info.binding_index {
            if let BindingType::Texture(_, id) =
                &self.binding_types[info.group_index][info.binding_index]
            {
                if id != texture_id {
                    self.binding_types[info.group_index][info.binding_index] =
                        BindingType::Texture(info.binding_index, *texture_id);
                    self.is_data_changed = true;
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

    pub fn set_bind_group_layout(&mut self) {
        inox_profiler::scoped_profile!("binding_data::bind_group_layout");

        if DEBUG_BINDINGS {
            inox_log::debug_log!("BindGroupLauout of {}", self.pass_name);
        }

        if self.is_layout_changed {
            self.bind_group_layout.clear();
            self.bind_group_layout_entries.iter().enumerate().for_each(
                |(index, bind_group_layout_entry)| {
                    inox_profiler::scoped_profile!(
                        "binding_data::create_{}_bind_group_layout_{}",
                        self.pass_name,
                        index
                    );
                    let label = format!("{} bind group layout {}", self.pass_name, index);

                    let data_bind_group_layout = self
                        .render_core_context
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
    }

    pub fn set_bind_groups(&mut self) {
        inox_profiler::scoped_profile!("binding_data::bind_groups");

        if DEBUG_BINDINGS {
            inox_log::debug_log!("BindGroups of {}", self.pass_name);
        }

        if !self.is_data_changed {
            self.binding_types
                .iter()
                .enumerate()
                .for_each(|(_, binding_type_array)| {
                    binding_type_array.iter().for_each(|binding_type| {
                        if let BindingType::Buffer(_, buffer_id) = binding_type {
                            self.is_data_changed |= self.binding_data_buffer.is_changed(buffer_id);
                        }
                    });
                });
        }

        if self.is_data_changed {
            let render_targets = self.texture_handler.render_targets();
            let texture_atlas = self.texture_handler.textures_atlas();
            self.bind_group.clear();
            self.binding_types
                .iter()
                .enumerate()
                .for_each(|(group_index, binding_type_array)| {
                    let label = format!("{} bind group {}", self.pass_name, group_index);

                    let mut textures_view = Vec::new();
                    binding_type_array.iter().for_each(|binding_type| {
                        if let BindingType::TextureArray(_, textures) = binding_type {
                            textures.iter().for_each(|id| {
                                if let Some(texture) =
                                    texture_atlas.iter().find(|t| t.texture_id() == id)
                                {
                                    textures_view.push(texture.texture_view().as_wgpu());
                                }
                                if let Some(texture) = render_targets.iter().find(|t| t.id() == id)
                                {
                                    textures_view.push(texture.view().as_wgpu());
                                }
                            });
                        }
                    });
                    let bind_data_buffer = self.binding_data_buffer.buffers.read().unwrap();
                    let mut bind_group = Vec::new();
                    binding_type_array
                        .iter()
                        .for_each(|binding_type| match binding_type {
                            BindingType::Buffer(binding_index, data_id) => {
                                if DEBUG_BINDINGS {
                                    inox_log::debug_log!(
                                        "Binding Buffer[{}][{}] = {:?}",
                                        group_index,
                                        binding_index,
                                        data_id
                                    );
                                }
                                if let Some(buffer) = bind_data_buffer.get(data_id) {
                                    if buffer.gpu_buffer().is_some() && buffer.size() != 0 {
                                        let entry = wgpu::BindGroupEntry {
                                            binding: *binding_index as _,
                                            resource: buffer
                                                .gpu_buffer()
                                                .unwrap()
                                                .as_entire_binding(),
                                        };
                                        bind_group.push(entry);
                                    } else if DEBUG_BINDINGS {
                                        inox_log::debug_log!(
                                            "Binding Buffer[{}][{}] = {:?} but buffer is empty",
                                            group_index,
                                            binding_index,
                                            data_id
                                        );
                                    }
                                } else if DEBUG_BINDINGS {
                                    inox_log::debug_log!(
                                        "Binding Buffer[{}][{}] = {:?} but buffer is not found",
                                        group_index,
                                        binding_index,
                                        data_id
                                    );
                                }
                            }
                            BindingType::DefaultSampler(binding_index) => {
                                if DEBUG_BINDINGS {
                                    inox_log::debug_log!(
                                        "Binding Default sampler [{}][{}]",
                                        group_index,
                                        binding_index
                                    );
                                }
                                bind_group.push(wgpu::BindGroupEntry {
                                    binding: *binding_index as _,
                                    resource: wgpu::BindingResource::Sampler(
                                        self.texture_handler.default_sampler(),
                                    ),
                                });
                            }
                            BindingType::Texture(binding_index, id) => {
                                if DEBUG_BINDINGS {
                                    inox_log::debug_log!(
                                        "Binding Texture[{}][{}] with id {:?}",
                                        group_index,
                                        binding_index,
                                        id
                                    );
                                }
                                if let Some(texture) = render_targets.iter().find(|t| t.id() == id)
                                {
                                    bind_group.push(wgpu::BindGroupEntry {
                                        binding: *binding_index as _,
                                        resource: wgpu::BindingResource::TextureView(
                                            texture.view().as_wgpu(),
                                        ),
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
                        });
                    {
                        inox_profiler::scoped_profile!(
                            "binding_data::create_{}_bind_group_{}",
                            self.pass_name,
                            group_index
                        );
                        let data_bind_group = self.render_core_context.device.create_bind_group(
                            &wgpu::BindGroupDescriptor {
                                layout: &self.bind_group_layout[group_index],
                                entries: bind_group.as_slice(),
                                label: Some(label.as_str()),
                            },
                        );
                        self.bind_group.push(data_bind_group);
                    }
                });
            self.is_data_changed = false;
        }
    }
}
