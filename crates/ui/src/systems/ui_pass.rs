use std::path::PathBuf;

use inox_core::ContextRc;
use inox_graphics::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, DrawCommandType, GpuBuffer, MeshFlags,
    Pass, RenderContext, RenderCoreContext, RenderPass, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, VertexBufferLayoutBuilder, VertexFormat,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

const UI_PIPELINE: &str = "pipelines/UI.render_pipeline";
pub const UI_PASS_NAME: &str = "UIPass";

#[repr(C, align(16))]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct UIPassData {
    pub ui_scale: f32,
    is_dirty: bool,
}

impl AsBinding for UIPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as _
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.ui_scale]);
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct UIVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: u32,
}
impl UIVertex {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<[f32; 2]>(VertexFormat::Float32x2.into());
        layout_builder.add_attribute::<[f32; 2]>(VertexFormat::Float32x2.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct UIInstance {
    pub index_start: u32,
    pub index_count: u32,
    pub vertex_start: u32,
    pub texture_index: u32,
}

impl UIInstance {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}

pub struct UIPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    custom_data: UIPassData,
    vertices: Vec<UIVertex>,
    indices: Vec<u32>,
    instances: Vec<UIInstance>,
}
unsafe impl Send for UIPass {}
unsafe impl Sync for UIPass {}

impl Pass for UIPass {
    fn name(&self) -> &str {
        UI_PASS_NAME
    }
    fn static_name() -> &'static str {
        UI_PASS_NAME
    }
    fn is_active(&self, _render_context: &mut RenderContext) -> bool {
        true
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Custom
    }
    fn draw_command_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: UI_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(UI_PIPELINE),
            ..Default::default()
        };
        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
                None,
            ),
            binding_data: BindingData::default(),
            custom_data: UIPassData::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
            instances: Vec::new(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("ui_pass::init");

        if self.instances.is_empty()
            || self.vertices.is_empty()
            || self.instances.is_empty()
            || render_context.render_buffers.textures.is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();
        let render_texture = pass.render_textures_id();
        let depth_texture = pass.depth_texture_id();

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,

                    ..Default::default()
                },
            )
            .add_textures(
                &render_context.texture_handler,
                render_texture,
                depth_texture,
                BindingInfo {
                    group_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .set_vertex_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                0,
                &mut self.vertices,
            )
            .set_vertex_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                1,
                &mut self.instances,
            )
            .set_index_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.indices,
            );
        self.binding_data.send_to_gpu(render_context, UI_PASS_NAME);

        let vertex_layout = UIVertex::descriptor(0);
        let instance_layout = UIInstance::descriptor(vertex_layout.location());
        pass.init(
            render_context,
            &self.binding_data,
            Some(vertex_layout),
            Some(instance_layout),
        );
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("ui_pass::update");

        let pass = self.render_pass.get();
        let buffers = render_context.buffers();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        {
            let mut render_pass = pass.begin(
                render_context,
                &self.binding_data,
                &buffers,
                &pipeline,
                command_buffer,
            );
            self.instances.iter().enumerate().for_each(|(i, instance)| {
                render_pass.draw_indexed(
                    instance.index_start..instance.index_start + instance.index_count,
                    instance.vertex_start as _,
                    i as _..(i + 1) as _,
                );
            });
        }
    }
}

impl UIPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_ui_scale(&mut self, ui_scale: f32) {
        if self.custom_data.ui_scale != ui_scale {
            self.custom_data.ui_scale = ui_scale;
            self.custom_data.is_dirty = true;
        }
    }
    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.vertices.clear();
        self.indices.clear();
    }
    pub fn set_instances(
        &mut self,
        vertices: Vec<UIVertex>,
        indices: Vec<u32>,
        instances: Vec<UIInstance>,
    ) {
        self.vertices = vertices;
        self.indices = indices;
        self.instances = instances;
    }
}
