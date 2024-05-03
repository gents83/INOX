use std::path::PathBuf;

use inox_render::{
    create_arrow, create_circumference, create_colored_quad, create_cube_from_min_max, create_line,
    create_sphere, AsBinding, BindingData, BindingInfo, CommandBuffer, ConstantDataRw,
    DrawCommandType, DrawEvent, DrawIndexedCommand, LoadOperation, MeshData, MeshFlags, Pass,
    RenderContext, RenderContextRc, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget,
    ShaderStage, StoreOperation, TextureView, VertexBufferLayoutBuilder, VertexFormat,
    VextexBindingType, View,
};

use inox_core::ContextRc;
use inox_math::{Mat4Ops, Matrix4};
use inox_messenger::Listener;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.render_pipeline";
pub const WIREFRAME_PASS_NAME: &str = "WireframePass";

#[derive(Default, Clone, Copy, PartialEq)]
pub struct DebugVertex {
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

pub struct WireframePass {
    context: ContextRc,
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    vertices: Vec<DebugVertex>,
    indices: Vec<u32>,
    instances: Vec<DrawIndexedCommand>,
    instance_count: usize,
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
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
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
            constant_data: render_context.global_buffers().constant_data.clone(),
            vertices: Vec::<DebugVertex>::default(),
            indices: Vec::<u32>::default(),
            instances: Vec::<DrawIndexedCommand>::default(),
            instance_count: 0,
            listener,
            binding_data: BindingData::new(render_context, WIREFRAME_PASS_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("wireframe_pass::init");

        self.process_messages(render_context);

        if self.vertices.is_empty() || self.indices.is_empty() {
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
            .set_vertex_buffer(
                VextexBindingType::Vertex,
                &mut self.vertices,
                Some("DebugVertices"),
            )
            .set_index_buffer(&mut self.indices, Some("DebugIndices"))
            .bind_buffer(&mut self.instances, Some("DebugInstances"))
            .bind_buffer(&mut self.instance_count, Some("DebugInstance Count"));

        let vertex_layout = DebugVertex::descriptor(0);
        pass.init(
            render_context,
            &mut self.binding_data,
            Some(vertex_layout),
            None,
        );
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("wireframe_pass::update");

        if self.vertices.is_empty() || self.indices.is_empty() {
            return;
        }

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
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
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "wireframe_pass",
            );

            pass.indirect_draw(
                render_context,
                &self.instances,
                &self.instance_count,
                render_pass,
            );
        }
    }
}

impl WireframePass {
    fn add_mesh(
        commands: &mut Vec<DrawIndexedCommand>,
        vertices: &mut Vec<DebugVertex>,
        indices: &mut Vec<u32>,
        mesh_data: MeshData,
    ) {
        let instance_index = commands.len();
        commands.push(DrawIndexedCommand {
            base_index: indices.len() as _,
            vertex_count: mesh_data.indices.len() as _,
            vertex_offset: vertices.len() as _,
            base_instance: instance_index as _,
            instance_count: 1,
        });
        for i in 0..mesh_data.vertex_count() {
            vertices.push(DebugVertex {
                position: mesh_data.position(i as _).into(),
                color: mesh_data.packed_color(i),
            });
        }
        indices.extend_from_slice(&mesh_data.indices);
    }
    fn process_messages(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("WireframePass::process_messages");

        let mut camera_pos = None;
        if let Some(view) = self
            .context
            .shared_data()
            .match_resource(|v: &View| v.view_index() == 0)
        {
            camera_pos = Some(view.get().view().translation());
        }

        self.vertices.clear();
        self.instances.clear();
        self.indices.clear();
        self.indices.mark_as_dirty(render_context);
        self.vertices.mark_as_dirty(render_context);

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

                    let mesh_data = create_cube_from_min_max(min, max, color);
                    Self::add_mesh(
                        &mut self.instances,
                        &mut self.vertices,
                        &mut self.indices,
                        mesh_data,
                    );
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
        self.indices.mark_as_dirty(render_context);
        self.vertices.mark_as_dirty(render_context);
        self.instances.mark_as_dirty(render_context);

        self.instance_count = self.instances.len();
        self.instance_count.mark_as_dirty(render_context);
    }
}
