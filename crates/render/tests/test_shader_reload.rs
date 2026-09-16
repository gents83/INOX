use std::path::{Path, PathBuf};

use std::sync::Arc;

use futures::executor::block_on;
use inox_messenger::MessageHub;
use inox_render::{
    BindingData, ComputePipeline, RenderContext, RenderPipeline, RenderPipelineData, Shader,
    ShaderData, TextureFormat,
};
use inox_resources::{
    DataTypeResource, ResourceHandle, ResourceTrait, SerializableResource, SharedData,
};
use inox_uid::generate_random_uid;

use inox_render::resources::shader::{is_shader, ReloadError};

#[test]
fn failed_reload_reports_location_and_retains_active_resource() {
    let error = ReloadError {
        path: PathBuf::from("data/shaders/lighting.wgsl"),
        stage: "fragment".to_string(),
        source_location: Some("line 12, column 4".to_string()),
        diagnostic: "expected expression".to_string(),
        previous_resource_active: true,
    };

    assert_eq!(error.path, Path::new("data/shaders/lighting.wgsl"));
    assert_eq!(error.stage, "fragment");
    assert_eq!(error.source_location.as_deref(), Some("line 12, column 4"));
    assert_eq!(error.diagnostic, "expected expression");
    assert!(error.previous_resource_active);
}

#[test]
fn reload_path_filter_rejects_paths_without_shader_extensions() {
    assert!(is_shader(Path::new("data/shaders/lighting.wgsl")));
    assert!(is_shader(Path::new("data/shaders/lighting.spv")));
    assert!(!is_shader(Path::new("data/shaders/lighting")));
    assert!(!is_shader(Path::new(
        "data/pipelines/lighting.compute_pipeline"
    )));
}

fn reload_context() -> (Arc<SharedData>, Arc<MessageHub>) {
    (
        Arc::new(SharedData::default()),
        Arc::new(MessageHub::default()),
    )
}

fn headless_render_context(
    shared_data: &Arc<SharedData>,
    message_hub: &Arc<MessageHub>,
) -> Arc<RenderContext> {
    Arc::new(
        block_on(RenderContext::create_headless(
            shared_data.clone(),
            message_hub.clone(),
        ))
        .expect("headless WebGPU adapter/device required for reload integration tests"),
    )
}

fn shader_resource(
    shared_data: &Arc<SharedData>,
    message_hub: &Arc<MessageHub>,
    path: &Path,
    wgsl_code: &str,
) -> Arc<ResourceHandle<Shader>> {
    let mut shader = Shader::create_from_data(
        shared_data,
        message_hub,
        generate_random_uid(),
        &ShaderData {
            spirv_code: Vec::new(),
            wgsl_code: wgsl_code.to_string(),
        },
    );
    shader.set_path(path);
    Arc::new(ResourceHandle::new(generate_random_uid(), shader))
}

const VALID_COMPUTE_WGSL: &str = "@compute @workgroup_size(1) fn main() {}";
const VALID_VERTEX_ONLY_WGSL: &str = "
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
";
const VALID_RENDER_WGSL: &str = "
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = select(-1.0, 1.0, (index & 1u) == 1u);
    let y = select(-1.0, 1.0, (index & 2u) == 2u);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
";

fn submit_compute_pass(context: &RenderContext, pipeline: &wgpu::ComputePipeline) {
    let device = &context.webgpu.device;
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Reload Compute Test Encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Reload Compute Test Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(1, 1, 1);
    }
    context.webgpu.queue.submit(Some(encoder.finish()));
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(10)),
        })
        .expect("compute submission must complete");
    assert!(
        block_on(error_scope.pop()).is_none(),
        "retained compute pipeline must remain usable"
    );
}

fn submit_shader_module(context: &RenderContext, module: &wgpu::ShaderModule) {
    let layout = context
        .webgpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Reload Shader Test Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
    let pipeline = context
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Reload Shader Test Pipeline"),
            layout: Some(&layout),
            module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
        .expect("retained shader module must create a pipeline");
    submit_compute_pass(context, &pipeline);
}

fn submit_render_pass(context: &RenderContext, pipeline: &wgpu::RenderPipeline) {
    let device = &context.webgpu.device;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Reload Render Test Target"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Reload Render Test Encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Reload Render Test Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.draw(0..3, 0..1);
    }
    context.webgpu.queue.submit(Some(encoder.finish()));
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(10)),
        })
        .expect("render submission must complete");
    assert!(
        block_on(error_scope.pop()).is_none(),
        "retained render pipeline must remain usable"
    );
}

#[test]
fn concrete_shader_init_retains_module_across_invalid_reload_then_commits() {
    let (shared_data, message_hub) = reload_context();
    let context = headless_render_context(&shared_data, &message_hub);
    let path = Path::new("reload_shader.wgsl");
    let mut shader = Shader::create_from_data(
        &shared_data,
        &message_hub,
        generate_random_uid(),
        &ShaderData {
            spirv_code: Vec::new(),
            wgsl_code: VALID_COMPUTE_WGSL.to_string(),
        },
    );
    shader.set_path(path);
    assert!(shader.init(&context).is_ok());
    submit_shader_module(&context, shader.module());

    let mut invalid = Shader::create_from_data(
        &shared_data,
        &message_hub,
        generate_random_uid(),
        &ShaderData {
            spirv_code: Vec::new(),
            wgsl_code: "@compute @workgroup_size(1) fn main( {".to_string(),
        },
    );
    invalid.set_path(path);
    shader.replace_from(&invalid);
    let error = shader.init(&context).expect_err("invalid shader must fail");
    submit_shader_module(&context, shader.module());
    assert!(shader.last_reload_error().is_some());
    assert!(error.previous_resource_active);

    let valid = Shader::create_from_data(
        &shared_data,
        &message_hub,
        generate_random_uid(),
        &ShaderData {
            spirv_code: Vec::new(),
            wgsl_code: VALID_COMPUTE_WGSL.to_string(),
        },
    );
    shader.replace_from(&valid);
    assert!(shader.init(&context).is_ok());
    assert!(shader.last_reload_error().is_none());
    let _committed_module = shader.module();
}

#[test]
fn concrete_compute_pipeline_init_retains_pipeline_across_invalid_reload_then_commits() {
    let (shared_data, message_hub) = reload_context();
    let context = headless_render_context(&shared_data, &message_hub);
    let shader_path = Path::new("reload_compute.wgsl");
    let shader = shader_resource(&shared_data, &message_hub, shader_path, VALID_COMPUTE_WGSL);
    assert!(shader.get_mut().init(&context).is_ok());
    let mut pipeline = ComputePipeline::from_shader(
        PathBuf::from("reload.compute_pipeline"),
        &shared_data,
        &message_hub,
        shader.clone(),
    );
    let binding_data = BindingData::new(&context, "reload");
    assert!(pipeline.init(&context, &binding_data, None));
    submit_compute_pass(&context, pipeline.compute_pipeline());

    let invalid = shader_resource(
        &shared_data,
        &message_hub,
        shader_path,
        "@compute @workgroup_size(1) fn other() {}",
    );
    shader.get_mut().replace_from(&invalid.get());
    pipeline.check_shaders_to_reload(shader_path.to_string_lossy().into_owned());
    assert!(!pipeline.init(&context, &binding_data, None));
    submit_compute_pass(&context, pipeline.compute_pipeline());
    assert_eq!(
        pipeline
            .last_reload_error()
            .map(|error| error.stage.as_str()),
        Some("compute pipeline")
    );

    let valid = shader_resource(&shared_data, &message_hub, shader_path, VALID_COMPUTE_WGSL);
    shader.get_mut().replace_from(&valid.get());
    pipeline.check_shaders_to_reload(shader_path.to_string_lossy().into_owned());
    assert!(pipeline.init(&context, &binding_data, None));
    assert!(pipeline.last_reload_error().is_none());
    submit_compute_pass(&context, pipeline.compute_pipeline());
}

#[test]
fn concrete_render_pipeline_init_retains_pipeline_across_invalid_reload_then_commits() {
    let (shared_data, message_hub) = reload_context();
    let context = headless_render_context(&shared_data, &message_hub);
    let shader_path = Path::new("reload_render.wgsl");
    let shader = shader_resource(&shared_data, &message_hub, shader_path, VALID_RENDER_WGSL);
    assert!(shader.get_mut().init(&context).is_ok());
    let data = RenderPipelineData {
        vertex_shader: shader_path.to_path_buf(),
        fragment_shader: shader_path.to_path_buf(),
        ..RenderPipelineData::default()
    };
    let mut pipeline = RenderPipeline::from_shaders(
        PathBuf::from("reload.render_pipeline"),
        data,
        &shared_data,
        &message_hub,
        shader.clone(),
        shader.clone(),
    );
    let binding_data = BindingData::new(&context, "reload");
    let format = TextureFormat::Bgra8UnormSrgb;
    assert!(pipeline.init(&context, vec![&format], None, &binding_data, None, None));
    submit_render_pass(&context, pipeline.render_pipeline());

    let invalid = shader_resource(
        &shared_data,
        &message_hub,
        shader_path,
        VALID_VERTEX_ONLY_WGSL,
    );
    shader.get_mut().replace_from(&invalid.get());
    pipeline.check_shaders_to_reload(shader_path.to_string_lossy().into_owned());
    assert!(!pipeline.init(&context, vec![&format], None, &binding_data, None, None));
    submit_render_pass(&context, pipeline.render_pipeline());
    assert_eq!(
        pipeline
            .last_reload_error()
            .map(|error| error.stage.as_str()),
        Some("render pipeline")
    );

    let valid = shader_resource(&shared_data, &message_hub, shader_path, VALID_RENDER_WGSL);
    shader.get_mut().replace_from(&valid.get());
    pipeline.check_shaders_to_reload(shader_path.to_string_lossy().into_owned());
    assert!(pipeline.init(&context, vec![&format], None, &binding_data, None, None));
    assert!(pipeline.last_reload_error().is_none());
    submit_render_pass(&context, pipeline.render_pipeline());
}
