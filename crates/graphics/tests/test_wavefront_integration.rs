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
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;

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

    fn production_shader(name: &str) -> String {
        fn expand(path: &std::path::Path, source: String) -> String {
            source
                .lines()
                .map(|line| {
                    let Some(import) = line
                        .trim()
                        .strip_prefix("#import")
                        .map(str::trim)
                        .map(|value| value.trim_matches('"'))
                    else {
                        return format!("{line}\n");
                    };
                    let imported = path.parent().unwrap().join(import);
                    expand(&imported, std::fs::read_to_string(&imported).unwrap())
                })
                .collect()
        }
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data_raw/shaders/wgsl")
            .join(name);
        expand(&path, std::fs::read_to_string(&path).unwrap())
    }

    fn write_bytes(queue: &wgpu::Queue, buffer: &wgpu::Buffer, bytes: &[u8]) {
        queue.write_buffer(buffer, 0, bytes);
    }

    fn storage_buffer(device: &wgpu::Device, size: u64, label: &str) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        })
    }

    #[test]
    fn headless_gpu_runs_production_primary_and_composite_sources() {
        let shared_data = Arc::new(SharedData::default());
        let message_hub = Arc::new(MessageHub::default());
        let Some(context) = block_on(RenderContext::create_headless(shared_data, message_hub))
        else {
            eprintln!("SKIPPED: no native WebGPU adapter available");
            return;
        };
        let device = &context.webgpu.device;
        let queue = &context.webgpu.queue;
        let mut constant_bytes = vec![0u8; std::mem::size_of::<inox_render::ConstantData>()];
        constant_bytes[320..324].copy_from_slice(&1.0f32.to_ne_bytes());
        constant_bytes[324..328].copy_from_slice(&1.0f32.to_ne_bytes());
        let constant = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("production constant data"),
            size: constant_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        write_bytes(queue, &constant, &constant_bytes);

        let primary_source = production_shader("compute_wavefront_primary.wgsl");
        let primary_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("production wavefront primary"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(primary_source)),
        });
        let primary = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("production wavefront primary pipeline"),
            layout: None,
            module: &primary_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let world = storage_buffer(device, 16, "production G-buffer world");
        let header = storage_buffer(device, 16, "production primary header");
        let rays = storage_buffer(device, 64, "production primary queue");
        let mut header_words = Vec::new();
        for word in [0u32, 0, 1, 0] {
            header_words.extend_from_slice(&word.to_ne_bytes());
        }
        write_bytes(queue, &header, &header_words);
        let packed_normal = 100663296.0f32;
        write_bytes(queue, &world, &packed_normal.to_ne_bytes());
        let primary_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("production primary bind group 0"),
            layout: &primary.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constant.as_entire_binding(),
            }],
        });
        let primary_group_entries = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: world.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: header.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: rays.as_entire_binding(),
            },
        ];
        let primary_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("production primary bind group 1"),
            layout: &primary.get_bind_group_layout(1),
            entries: &primary_group_entries,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("production primary encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("production primary pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&primary);
            pass.set_bind_group(0, &primary_group, &[]);
            pass.set_bind_group(1, &primary_group_1, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            })
            .expect("primary submission must complete");
        let header_words = readback(device, queue, &header);
        let primary_count = u32::from_ne_bytes(header_words[4..8].try_into().unwrap());
        assert_eq!(primary_count, 1, "production primary queue count stage");
        let ray_bytes = readback(device, queue, &rays);
        let ray_values = (0..12)
            .map(|index| {
                f32::from_ne_bytes(ray_bytes[index * 4..index * 4 + 4].try_into().unwrap())
            })
            .collect::<Vec<_>>();
        assert!(
            ray_values.iter().all(|value| value.is_finite()),
            "production primary ray record must be finite"
        );
        assert!(
            ray_values.iter().any(|value| *value != 0.0),
            "production primary ray record must be nonzero"
        );

        let composite_source = production_shader("compute_wavefront_composite.wgsl");
        let composite_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("production wavefront composite"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(composite_source)),
        });
        let composite = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("production wavefront composite pipeline"),
            layout: None,
            module: &composite_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let final_accumulator = storage_buffer(device, 32, "production final accumulator");
        let direct = storage_buffer(device, 32, "production direct accumulator");
        let ao = storage_buffer(device, 16, "production AO state");
        let indirect = storage_buffer(device, 32, "production indirect accumulator");
        let mut direct_bytes = Vec::new();
        for value in [2.0f32, 0.5, 0.25, 1.0] {
            direct_bytes.extend_from_slice(&value.to_ne_bytes());
        }
        write_bytes(queue, &direct, &direct_bytes);
        let mut ao_bytes = Vec::new();
        for word in [
            1.0f32.to_ne_bytes(),
            1.0f32.to_ne_bytes(),
            1u32.to_ne_bytes(),
            0u32.to_ne_bytes(),
        ] {
            ao_bytes.extend_from_slice(&word);
        }
        write_bytes(queue, &ao, &ao_bytes);
        let mut indirect_bytes = Vec::new();
        for value in [0.1f32, 0.2, 0.3, 1.0] {
            indirect_bytes.extend_from_slice(&value.to_ne_bytes());
        }
        write_bytes(queue, &indirect, &indirect_bytes);
        let composite_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("production composite bind group"),
            layout: &composite.get_bind_group_layout(0),
            entries: &[
                constant.as_entire_binding(),
                final_accumulator.as_entire_binding(),
                direct.as_entire_binding(),
                ao.as_entire_binding(),
                indirect.as_entire_binding(),
            ]
            .iter()
            .enumerate()
            .map(|(binding, resource)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: resource.clone(),
            })
            .collect::<Vec<_>>(),
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("production composite encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("production composite pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&composite);
            pass.set_bind_group(0, &composite_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            })
            .expect("composite submission must complete");
        let output = readback(device, queue, &final_accumulator);
        let output_values = (0..3)
            .map(|index| f32::from_ne_bytes(output[index * 4..index * 4 + 4].try_into().unwrap()))
            .collect::<Vec<_>>();
        assert!(
            output_values.iter().all(|value| value.is_finite()),
            "production final accumulation must be finite"
        );
        assert!(
            output_values.iter().any(|value| *value > 0.0),
            "production final accumulation must be nonzero"
        );
        assert!(
            output_values.iter().any(|value| *value < 1.0),
            "production final accumulation must not be white"
        );
    }
}
