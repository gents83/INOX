use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ConstantDataRw, GPUBuffer, GPUInstance,
    GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture, GPUTransform, GPUVector,
    GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, LoadOperation, Pass, RenderContext,
    RenderContextRc, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, SamplerType,
    ShaderStage, StoreOperation, TextureId, TextureView, CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE,
    CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::DebugPackedData;
use crate::SIZE_OF_DATA_BUFFER_ELEMENT;

pub const DEBUG_PIPELINE: &str = "pipelines/Debug.render_pipeline";
pub const DEBUG_PASS_NAME: &str = "DebugPass";

/// Optional diagnostics owned by the viewer rather than the production graph.
/// Every flag is disabled by default so constructing a renderer cannot change
/// production output or add debug work accidentally.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugOptions {
    pub pass_graph: bool,
    pub queue_headers: bool,
    pub dispatch_args: bool,
    pub resource_ids: bool,
    pub visual_buffer: DebugVisualBuffer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebugVisualBuffer {
    #[default]
    None,
    FinalAccumulator,
    PathtraceRecords,
}

impl DebugOptions {
    pub const fn disabled() -> Self {
        Self {
            pass_graph: false,
            queue_headers: false,
            dispatch_args: false,
            resource_ids: false,
            visual_buffer: DebugVisualBuffer::None,
        }
    }

    pub const fn is_enabled(self) -> bool {
        self.pass_graph
            || self.queue_headers
            || self.dispatch_args
            || self.resource_ids
            || !matches!(self.visual_buffer, DebugVisualBuffer::None)
    }
}

pub struct DebugPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    indices: GPUBuffer<GPUVertexIndices>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    textures: GPUBuffer<GPUTexture>,
    materials: GPUBuffer<GPUMaterial>,
    lights: GPUBuffer<GPULight>,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    finalize_texture: TextureId,
    data_buffer_debug: GPUVector<DebugPackedData>,
    dimensions: inox_render::FrameDimensions,
    options: DebugOptions,
}
unsafe impl Send for DebugPass {}
unsafe impl Sync for DebugPass {}

impl Pass for DebugPass {
    fn name(&self) -> &str {
        DEBUG_PASS_NAME
    }
    fn static_name() -> &'static str {
        DEBUG_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        self.options.is_enabled()
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("debug_pass::create");

        let data = RenderPassData {
            name: DEBUG_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
            load_depth: LoadOperation::Load,
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(DEBUG_PIPELINE),
            ..Default::default()
        };

        let data_buffer_debug = render_context.global_buffers().vector::<DebugPackedData>();
        let dimensions = render_context.frame_dimensions();
        resize_debug_buffer(&data_buffer_debug, dimensions);

        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            data_buffer_debug: render_context.global_buffers().vector::<DebugPackedData>(),
            dimensions,
            options: DebugOptions::disabled(),
            binding_data: BindingData::new(render_context, DEBUG_PASS_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_position: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            finalize_texture: INVALID_UID,
            visibility_texture: INVALID_UID,
            depth_texture: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("debug_pass::init");

        if self.indices.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
            || self.visibility_texture.is_nil()
            || self.depth_texture.is_nil()
            || self.finalize_texture.is_nil()
        {
            return;
        }

        let visual_flag = match self.options.visual_buffer {
            DebugVisualBuffer::None => 0,
            DebugVisualBuffer::FinalAccumulator => CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER,
            DebugVisualBuffer::PathtraceRecords => CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE,
        };
        if visual_flag != 0 {
            self.constant_data
                .write()
                .unwrap()
                .add_flag(render_context, visual_flag);
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Index,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_position.write().unwrap(),
                Some("Vertices Position"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Vertex,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("Vertices Attributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("Transforms"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.visibility_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.finalize_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_debug.write().unwrap(),
                Some("DataBuffer_Debug"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            );

        self.binding_data
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
                SamplerType::Unfiltered,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            });
        pass.init(render_context, &mut self.binding_data, None, None);
    }
    fn resize(&mut self, dimensions: inox_render::FrameDimensions) {
        self.dimensions = dimensions;
        resize_debug_buffer(&self.data_buffer_debug, dimensions);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("debug_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        if self.indices.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
            || self.visibility_texture.is_nil()
            || self.depth_texture.is_nil()
            || self.finalize_texture.is_nil()
        {
            return;
        }

        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler().render_targets();

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.webgpu,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        #[allow(unused_mut)]
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "debug_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

fn resize_debug_buffer(
    buffer: &GPUVector<DebugPackedData>,
    dimensions: inox_render::FrameDimensions,
) {
    let pixels = dimensions
        .storage_pixel_count()
        .expect("validated frame dimensions fit storage arithmetic") as usize;
    // GPUVector::resize takes typed elements, while each debug pixel reserves
    // four f32 words for packed records. Keep this conversion explicit.
    let elements = pixels
        .checked_mul(SIZE_OF_DATA_BUFFER_ELEMENT)
        .expect("validated frame dimensions fit debug buffer arithmetic");
    buffer
        .write()
        .unwrap()
        .resize(elements, DebugPackedData(0.));
}

impl DebugPass {
    pub fn create_with_options(
        context: &ContextRc,
        render_context: &RenderContextRc,
        dimensions: inox_render::FrameDimensions,
        options: DebugOptions,
    ) -> Self {
        let mut pass = <Self as Pass>::create(context, render_context);
        pass.resize(dimensions);
        pass.options = options;
        pass
    }

    pub fn options(&self) -> DebugOptions {
        self.options
    }

    pub fn dimensions(&self) -> inox_render::FrameDimensions {
        self.dimensions
    }

    pub fn set_options(&mut self, options: DebugOptions) -> &mut Self {
        self.options = options;
        self
    }

    pub fn set_visibility_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.visibility_texture = *texture_id;
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
    pub fn set_finalize_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.finalize_texture = *texture_id;
        self
    }
}
