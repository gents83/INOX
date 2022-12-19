use std::path::PathBuf;

use crate::{
    create_arrow, create_circumference, create_colored_quad, create_line, create_sphere,
    declare_as_binding_vector, AsBinding, BindingData, BindingInfo, CommandBuffer, ConstantDataRw,
    DrawCommandType, DrawEvent, LoadOperation, MeshData, MeshFlags, OutputRenderPass, Pass,
    RenderContext, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureView, VertexBufferLayoutBuilder, VertexFormat, View,
};

use inox_core::ContextRc;
use inox_math::{Mat4Ops, Matrix4};
use inox_messenger::Listener;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.render_pipeline";
pub const WIREFRAME_PASS_NAME: &str = "WireframePass";

#[derive(Default, Clone, Copy, PartialEq)]
struct DebugVertex {
    position: [f32; 3],
    color: u32,
}
impl DebugVertex {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<[f32; 3]>(VertexFormat::Float32x3.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct DebugInstance {
    instance_index: u32,
    index_start: u32,
    index_count: u32,
    vertex_start: u32,
}

impl DebugInstance {
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

declare_as_binding_vector!(VecDebugVertex, DebugVertex);
declare_as_binding_vector!(VecDebugIndex, u32);
declare_as_binding_vector!(VecDebugInstance, DebugInstance);

pub struct WireframePass {
    context: ContextRc,
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    vertices: VecDebugVertex,
    indices: VecDebugIndex,
    instances: VecDebugInstance,
    listener: Listener,
}
unsafe impl Send for WireframePass {}
unsafe impl Sync for WireframePass {}

impl Pass for WireframePass {
    fn name(&self) -> &str {
        WIREFRAME_PASS_NAME
    }
    fn static_name() -> &'static str {
        WIREFRAME_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Wireframe
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("wireframe_pass::create");

        let data = RenderPassData {
            name: WIREFRAME_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
            load_depth: LoadOperation::Load,
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(WIREFRAME_PIPELINE),
            ..Default::default()
        };

        let listener = Listener::new(context.message_hub());
        listener.register::<DrawEvent>();

        Self {
            context: context.clone(),
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            vertices: VecDebugVertex::default(),
            indices: VecDebugIndex::default(),
            instances: VecDebugInstance::default(),
            listener,
            binding_data: BindingData::new(render_context, WIREFRAME_PASS_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("wireframe_pass::init");

        self.process_messages();

        if self.instances.data.is_empty()
            || self.vertices.data.is_empty()
            || self.indices.data.is_empty()
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
            .set_vertex_buffer(0, &mut self.vertices, Some("DebugVertices"))
            .set_vertex_buffer(1, &mut self.instances, Some("DebugInstances"))
            .set_index_buffer(&mut self.indices, Some("DebugIndices"));

        let vertex_layout = DebugVertex::descriptor(0);
        let instance_layout = DebugInstance::descriptor(vertex_layout.location());
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
        inox_profiler::scoped_profile!("wireframe_pass::update");

        if self.instances.data.is_empty()
            || self.vertices.data.is_empty()
            || self.indices.data.is_empty()
        {
            return;
        }

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler.render_targets();

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
                "wireframe_pass",
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

impl OutputRenderPass for WireframePass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}

impl WireframePass {
    fn add_mesh(
        instances: &mut VecDebugInstance,
        vertices: &mut VecDebugVertex,
        indices: &mut VecDebugIndex,
        mesh_data: MeshData,
    ) {
        instances.data.push(DebugInstance {
            index_start: indices.data.len() as _,
            index_count: mesh_data.indices.len() as _,
            vertex_start: vertices.data.len() as _,
            instance_index: instances.data.len() as _,
        });
        mesh_data.vertices.iter().for_each(|v| {
            vertices.data.push(DebugVertex {
                position: mesh_data.position(v.position_and_color_offset as _).into(),
                color: mesh_data.colors[v.position_and_color_offset as usize],
            });
        });
        indices.data.extend_from_slice(&mesh_data.indices);
        instances.set_dirty(true);
        indices.set_dirty(true);
        vertices.set_dirty(true);
    }
    fn process_messages(&mut self) {
        inox_profiler::scoped_profile!("WireframePass::process_messages");

        let mut camera_pos = None;
        if let Some(view) = self
            .context
            .shared_data()
            .match_resource(|v: &View| v.view_index() == 0)
        {
            camera_pos = Some(view.get().view().inverse().translation());
        }

        if !self.vertices.data.is_empty() {
            self.instances.set_dirty(true);
            self.indices.set_dirty(true);
            self.vertices.set_dirty(true);
        }
        self.vertices.data.clear();
        self.instances.data.clear();
        self.indices.data.clear();

        self.listener
            .process_messages(|event: &DrawEvent| match *event {
                DrawEvent::Line(start, end, color) => {
                    inox_profiler::scoped_profile!("DrawEvent::Line");

                    let mesh_data = create_line(start, end, color);
                    Self::add_mesh(
                        &mut self.instances,
                        &mut self.vertices,
                        &mut self.indices,
                        mesh_data,
                    );
                }
                DrawEvent::BoundingBox(min, max, color) => {
                    inox_profiler::scoped_profile!("DrawEvent::BoundingBox");

                    let mut mesh_data: [MeshData; 6] = Default::default();
                    mesh_data[0] =
                        create_colored_quad([min.x, min.y, max.x, max.y].into(), min.z, color);
                    mesh_data[1] =
                        create_colored_quad([min.x, min.y, max.x, max.y].into(), max.z, color);
                    mesh_data[2] = create_line(
                        [min.x, min.y, min.z].into(),
                        [min.x, min.y, max.z].into(),
                        color,
                    );
                    mesh_data[3] = create_line(
                        [min.x, max.y, min.z].into(),
                        [min.x, max.y, max.z].into(),
                        color,
                    );
                    mesh_data[4] = create_line(
                        [max.x, min.y, min.z].into(),
                        [max.x, min.y, max.z].into(),
                        color,
                    );
                    mesh_data[5] = create_line(
                        [max.x, max.y, min.z].into(),
                        [max.x, max.y, max.z].into(),
                        color,
                    );
                    mesh_data.into_iter().for_each(|mesh_data| {
                        Self::add_mesh(
                            &mut self.instances,
                            &mut self.vertices,
                            &mut self.indices,
                            mesh_data,
                        );
                    });
                }
                DrawEvent::Quad(min, max, z, color, _is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Quad");

                    let mesh_data =
                        create_colored_quad([min.x, min.y, max.x, max.y].into(), z, color);
                    Self::add_mesh(
                        &mut self.instances,
                        &mut self.vertices,
                        &mut self.indices,
                        mesh_data,
                    );
                }
                DrawEvent::Arrow(position, direction, color, _is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Arrow");

                    let mesh_data = create_arrow(position, direction, color);
                    Self::add_mesh(
                        &mut self.instances,
                        &mut self.vertices,
                        &mut self.indices,
                        mesh_data,
                    );
                }
                DrawEvent::Sphere(position, radius, color, _is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Sphere");

                    let mesh_data = create_sphere(position, radius, 16, 8, color);
                    Self::add_mesh(
                        &mut self.instances,
                        &mut self.vertices,
                        &mut self.indices,
                        mesh_data,
                    );
                }
                DrawEvent::Circle(position, radius, color, _is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Circle");

                    let mut mesh_data = create_circumference(position, radius, 16, color);
                    if let Some(camera_pos) = camera_pos {
                        let mut matrix = Matrix4::from_translation(position);
                        matrix.look_at(camera_pos);
                        matrix.add_translation(-position);
                        mesh_data.aabb_min = matrix.rotate_point(mesh_data.aabb_min);
                        mesh_data.aabb_max = matrix.rotate_point(mesh_data.aabb_max);
                    }
                    Self::add_mesh(
                        &mut self.instances,
                        &mut self.vertices,
                        &mut self.indices,
                        mesh_data,
                    );
                }
            });
    }
}
