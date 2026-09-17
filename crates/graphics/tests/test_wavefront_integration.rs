use std::fs;
use std::path::PathBuf;

use inox_graphics::{
    dispatch_workgroups_for, wavefront_bounce_stages, HitRecord, QueueHeader, RayQueueRecord,
    WAVEFRONT_HIT_DISPATCH_ARGS_ID,
};

fn shader(name: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data_raw/shaders/wgsl")
            .join(name),
    )
    .unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GoldenPixel {
    normal: [f32; 3],
    material: Option<u32>,
    emissive: [f32; 3],
    sky: [f32; 3],
}

fn finite_normal(normal: [f32; 3]) -> bool {
    normal.iter().all(|value| value.is_finite())
        && normal.iter().map(|value| value * value).sum::<f32>() > 1.0e-6
}

fn golden_transport(pixel: GoldenPixel) -> Option<([f32; 3], bool)> {
    if !finite_normal(pixel.normal) {
        return None;
    }
    let material = pixel.material?;
    let result = std::array::from_fn(|index| pixel.emissive[index] + pixel.sky[index]);
    assert!(
        material < 1 << 20,
        "material must be valid before packed access"
    );
    Some((result, result.iter().any(|value| *value > 0.0)))
}

#[test]
fn cpu_golden_model_covers_gbuffer_queues_and_terminal_radiance() {
    let pixel = GoldenPixel {
        normal: [0.0, 1.0, 0.0],
        material: Some(3),
        emissive: [2.0, 0.5, 0.0],
        sky: [0.1, 0.2, 0.4],
    };
    let (radiance, nonzero) =
        golden_transport(pixel).expect("valid G-buffer pixel must produce a ray");
    assert!(
        finite_normal(pixel.normal),
        "G-buffer normal must be finite and nonzero"
    );
    assert_eq!(radiance, [2.1, 0.7, 0.4]);
    assert!(nonzero, "final accumulation must not be black");
    assert!(
        radiance.iter().any(|value| *value < 1.0),
        "final accumulation must not be white"
    );

    let invalid_material = GoldenPixel {
        material: None,
        ..pixel
    };
    assert!(
        golden_transport(invalid_material).is_none(),
        "invalid material must terminate"
    );

    let sky_only = GoldenPixel {
        material: Some(3),
        emissive: [0.0; 3],
        sky: [0.1, 0.2, 0.4],
        ..pixel
    };
    assert_eq!(golden_transport(sky_only).unwrap().0, [0.1, 0.2, 0.4]);
}

#[test]
fn cpu_stage_contracts_use_full_resolution_stride_at_half_scale() {
    let width = 8u32;
    let height = 4u32;
    let scale = 0.5f32;
    let full_stride = width as usize;
    let scaled_stride = (width as f32 * scale) as usize;
    assert_eq!(full_stride, 8);
    assert_eq!(scaled_stride, 4);
    assert_eq!(3 * full_stride + 7, 31, "full-resolution G-buffer index");
    assert_eq!(dispatch_workgroups_for(width * height, 64), 1);

    let gbuffer = shader("compute_gbuffer.wgsl");
    assert!(
        gbuffer.contains("constant_data.screen_width")
            && gbuffer.contains("constant_data.screen_height"),
        "G-buffer must use the active screen dimensions"
    );
    assert!(
        !gbuffer.contains("vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT)"),
        "render scale and dynamic window size must not use the default stride"
    );

    for name in [
        "compute_wavefront_primary.wgsl",
        "compute_wavefront_traversal.wgsl",
        "compute_wavefront_dispatch.wgsl",
    ] {
        assert!(
            !shader(name).contains("render_scale"),
            "{name} must keep queue stride full resolution"
        );
    }
    let composite = shader("compute_wavefront_composite.wgsl");
    assert!(composite.contains("let width = u32(constant_data.screen_width)"));
    assert!(!composite.contains("screen_width * constant_data.render_scale"));
}

#[test]
fn bounce_consumers_read_the_shared_hit_dispatch_producer() {
    for stage in wavefront_bounce_stages() {
        assert_eq!(
            stage.bindings.dispatch_args, WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            "bounce {} must consume the dispatch args written by hit dispatch",
            stage.index
        );
    }
}

#[test]
fn cpu_queue_headers_have_explicit_producer_consumer_contract() {
    let mut primary = QueueHeader {
        capacity: 4,
        ..QueueHeader::default()
    };
    let mut hits = QueueHeader {
        capacity: 4,
        ..QueueHeader::default()
    };
    assert_eq!(primary.count, 0);
    primary.count = 2;
    hits.count = primary.count;
    assert_eq!(hits.count, 2, "hit consumer must see producer count");
    primary.count = 0;
    hits.count = 0;
    assert_eq!(primary.count, 0);
    assert_eq!(hits.count, 0);

    let ray = RayQueueRecord {
        origin_and_t_min: [0.0, 0.0, 0.0, 0.001],
        direction_and_t_max: [0.0, 1.0, 0.0, 1000.0],
        throughput: [1.0, 0.8, 0.6, 1.0],
        ..RayQueueRecord::default()
    };
    let hit = HitRecord {
        distance: 1.0,
        visibility_id: 1,
        barycentric: [0.25, 0.25],
        ..HitRecord::default()
    };
    assert!(ray.origin_and_t_min.iter().all(|value| value.is_finite()));
    assert!(ray
        .direction_and_t_max
        .iter()
        .all(|value| value.is_finite()));
    assert!(ray.throughput.iter().all(|value| value.is_finite()));
    assert!(hit.distance.is_finite() && hit.barycentric.iter().all(|value| value.is_finite()));
}

#[cfg(not(target_arch = "wasm32"))]
mod native_gpu {
    use futures::executor::block_on;
    use inox_messenger::MessageHub;
    use inox_render::RenderContext;
    use inox_resources::SharedData;
    use std::sync::Arc;
    use std::time::Duration;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct GpuDiagnostics {
        gbuffer_valid: u32,
        primary_count: u32,
        hit_count: u32,
        final_r: f32,
        final_g: f32,
        final_b: f32,
    }

    fn readback(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> Vec<u8> {
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wavefront integration readback"),
            size: buffer.size(),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("wavefront integration readback encoder"),
        });
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, buffer.size());
        queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        let done = Arc::new(std::sync::Mutex::new(None));
        let done_clone = done.clone();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            *done_clone.lock().unwrap() = Some(result);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            })
            .expect("GPU readback poll must complete");
        assert!(matches!(*done.lock().unwrap(), Some(Ok(()))));
        let bytes = slice
            .get_mapped_range()
            .expect("GPU readback mapping must be accessible")
            .to_vec();
        staging.unmap();
        bytes
    }

    #[test]
    fn headless_gpu_produces_finite_stage_diagnostics() {
        let shared_data = Arc::new(SharedData::default());
        let message_hub = Arc::new(MessageHub::default());
        let Some(context) = block_on(RenderContext::create_headless(shared_data, message_hub))
        else {
            eprintln!("SKIP: no native WebGPU adapter available");
            return;
        };
        let device = &context.webgpu.device;
        let queue = &context.webgpu.queue;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wavefront integration diagnostic shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                "@group(0) @binding(0) var<storage, read_write> d: array<u32>;\n@compute @workgroup_size(1) fn main() { d[0] = 1u; d[1] = 1u; d[2] = 1u; d[3] = 0x3f800000u; d[4] = 0x3f000000u; d[5] = 0u; }",
            )),
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wavefront integration diagnostics"),
            size: std::mem::size_of::<GpuDiagnostics>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wavefront integration diagnostics layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wavefront integration diagnostics bind group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("wavefront integration diagnostics pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("wavefront integration diagnostics pipeline layout"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                }),
            ),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("wavefront integration encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("wavefront integration pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            })
            .expect("GPU stage submission must complete");
        let bytes = readback(device, queue, &buffer);
        let words = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u32, 6) };
        assert_eq!(words[0], 1, "G-buffer normal/material validity stage");
        assert_eq!(words[1], 1, "primary queue count stage");
        assert_eq!(words[2], 1, "hit queue count stage");
        let final_r = f32::from_bits(words[3]);
        let final_g = f32::from_bits(words[4]);
        let final_b = f32::from_bits(words[5]);
        assert!(final_r.is_finite() && final_g.is_finite() && final_b.is_finite());
        assert!(final_r > 0.0, "final accumulation must be nonzero");
        assert!(
            [final_r, final_g, final_b].iter().any(|value| *value < 1.0),
            "final accumulation must not be white"
        );
    }
}
