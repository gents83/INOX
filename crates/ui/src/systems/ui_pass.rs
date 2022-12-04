use std::path::PathBuf;

use inox_core::ContextRc;
use inox_graphics::{
    declare_as_binding_vector, AsBinding, BindingData, BindingInfo, CommandBuffer, ConstantDataRw,
    DrawCommandType, GpuBuffer, MeshFlags, OutputRenderPass, Pass, RenderContext,
    RenderCoreContext, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureView, TexturesBuffer, VertexBufferLayoutBuilder, VertexFormat,
};
use inox_messenger::Listener;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

use crate::UIEvent;

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

declare_as_binding_vector!(VecUIVertex, UIVertex);
declare_as_binding_vector!(VecUIIndex, u32);
declare_as_binding_vector!(VecUIInstance, UIInstance);

pub struct UIPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    textures: TexturesBuffer,
    custom_data: UIPassData,
    vertices: VecUIVertex,
    indices: VecUIIndex,
    instances: VecUIInstance,
    listener: Listener,
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
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Custom
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
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
        let listener = Listener::new(context.message_hub());
        listener.register::<UIEvent>();
        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            textures: render_context.render_buffers.textures.clone(),
            binding_data: BindingData::new(render_context, UI_PASS_NAME),
            custom_data: UIPassData {
                ui_scale: 2.,
                is_dirty: true,
            },
            vertices: VecUIVertex::default(),
            indices: VecUIIndex::default(),
            instances: VecUIInstance::default(),
            listener,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("ui_pass::init");

        self.process_messages();

        if self.instances.data.is_empty()
            || self.vertices.data.is_empty()
            || self.indices.data.is_empty()
            || self.textures.read().unwrap().is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_uniform_buffer(
                &mut self.custom_data,
                Some("UICustomData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,

                    ..Default::default()
                },
            )
            .add_default_sampler(BindingInfo {
                group_index: 2,
                binding_index: 0,
                stage: ShaderStage::Fragment,
                ..Default::default()
            })
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            })
            .set_vertex_buffer(0, &mut self.vertices, Some("UIVertices"))
            .set_vertex_buffer(1, &mut self.instances, Some("UIInstances"))
            .set_index_buffer(&mut self.indices, Some("UIIndices"));

        let vertex_layout = UIVertex::descriptor(0);
        let instance_layout = UIInstance::descriptor(vertex_layout.location());
        pass.init(
            render_context,
            &mut self.binding_data,
            Some(vertex_layout),
            Some(instance_layout),
        );
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("ui_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler.render_targets();

        if self.instances.data.is_empty()
            || self.vertices.data.is_empty()
            || self.indices.data.is_empty()
            || self.textures.read().unwrap().is_empty()
        {
            return;
        }

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.core,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.core.device,
                "ui_pass",
            );
            self.instances
                .data
                .iter()
                .enumerate()
                .for_each(|(i, instance)| {
                    render_pass.draw_indexed(
                        instance.index_start..instance.index_start + instance.index_count,
                        instance.vertex_start as _,
                        i as _..(i + 1) as _,
                    );
                });
        }
    }
}

impl OutputRenderPass for UIPass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
impl UIPass {
    fn process_messages(&mut self) {
        self.listener
            .process_messages(|event: &UIEvent| match event {
                UIEvent::Scale(ui_scale) => {
                    if self.custom_data.ui_scale != *ui_scale {
                        self.custom_data.ui_scale = *ui_scale;
                        self.custom_data.set_dirty(true);
                    }
                }
                UIEvent::DrawData(vertices, indices, instances) => {
                    self.vertices.set(vertices.clone());
                    self.indices.set(indices.clone());
                    self.instances.set(instances.clone());
                }
            });
    }
}
