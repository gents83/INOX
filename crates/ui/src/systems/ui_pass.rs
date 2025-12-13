use std::path::PathBuf;

use inox_core::ContextRc;
use inox_messenger::Listener;
use inox_render::{
    declare_as_binding, AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer,
    ConstantDataRw, GPUBuffer, GPUTexture, LoadOperation, Pass, RenderContext, RenderContextRc,
    RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, SamplerType, ShaderStage,
    StoreOperation, TextureView, VertexBufferLayoutBuilder, VertexFormat, VextexBindingType,
};
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

use crate::UIEvent;

const UI_PIPELINE: &str = "pipelines/UI.render_pipeline";
pub const UI_PASS_NAME: &str = "UIPass";

#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct UIPassData {
    pub ui_scale: f32,
    pub _padding: [f32; 3],
}
declare_as_binding!(UIPassData);

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
    constant_data: ConstantDataRw,
    textures: GPUBuffer<GPUTexture>,
    custom_data: UIPassData,
    vertices: Vec<UIVertex>,
    indices: Vec<u32>,
    instances: Vec<UIInstance>,
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
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: UI_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
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
            constant_data: render_context.global_buffers().constant_data.clone(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            binding_data: BindingData::new(render_context, UI_PASS_NAME),
            custom_data: UIPassData {
                ui_scale: 1.,
                ..Default::default()
            },
            vertices: Vec::default(),
            indices: Vec::default(),
            instances: Vec::default(),
            listener,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("ui_pass::init");

        self.process_messages(render_context);

        if self.instances.is_empty()
            || self.vertices.is_empty()
            || self.indices.is_empty()
            || self.textures.read().unwrap().is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut self.custom_data,
                Some("UICustomData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
                SamplerType::Default,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            })
            .set_vertex_buffer(
                VextexBindingType::Vertex,
                &mut self.vertices,
                Some("UIVertices"),
            )
            .set_vertex_buffer(
                VextexBindingType::Instance,
                &mut self.instances,
                Some("UIInstances"),
            )
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
        let render_targets = render_context.texture_handler().render_targets();

        if self.instances.is_empty()
            || self.vertices.is_empty()
            || self.indices.is_empty()
            || self.textures.read().unwrap().is_empty()
        {
            return;
        }

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.webgpu,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "ui_pass",
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
    fn process_messages(&mut self, render_context: &RenderContext) {
        self.listener
            .process_messages(|event: &UIEvent| match event {
                UIEvent::Scale(ui_scale) => {
                    if self.custom_data.ui_scale != *ui_scale {
                        self.custom_data.ui_scale = *ui_scale;
                        self.custom_data.mark_as_dirty(render_context);
                    }
                }
                UIEvent::DrawData(vertices, indices, instances) => {
                    self.vertices.clone_from(vertices);
                    self.vertices.mark_as_dirty(render_context);
                    self.indices.clone_from(indices);
                    self.indices.mark_as_dirty(render_context);
                    self.instances.clone_from(instances);
                    self.instances.mark_as_dirty(render_context);
                }
            });
    }
}
