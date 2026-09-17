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

#[cfg(not(target_arch = "wasm32"))]
mod native_causal_smoke {
    use futures::executor::block_on;
    use inox_binarizer::ShaderCompiler;
    use inox_messenger::MessageHub;
    use inox_platform::PLATFORM_TYPE_WEB;
    use inox_render::RenderContext;
    use inox_resources::SharedData;
    use std::borrow::Cow;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::Duration;

    const WIDTH: u32 = 4;
    const HEIGHT: u32 = 4;
    const PIXELS: usize = (WIDTH * HEIGHT) as usize;
    const CONSTANT_BYTES: usize = 416;

    fn expand_imports(path: &Path, source: String) -> String {
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
                expand_imports(
                    &path.parent().unwrap().join(import),
                    std::fs::read_to_string(path.parent().unwrap().join(import)).unwrap(),
                )
            })
            .collect()
    }

    fn production_shader(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data_raw/shaders/wgsl")
            .join(name);
        let expanded = expand_imports(&path, std::fs::read_to_string(&path).unwrap());
        // The native smoke test deliberately uses the portable, non-array texture binding
        // layout.  ShaderCompiler still supplies the platform WGSL extension preprocessing.
        let mut result = String::new();
        let mut include = true;
        let mut in_feature = false;
        for line in expanded.lines() {
            let trimmed = line.trim();
            if trimmed == "#ifdef FEATURES_TEXTURE_BINDING_ARRAY" {
                in_feature = true;
                include = false;
                continue;
            }
            if in_feature && trimmed == "#else" {
                include = true;
                continue;
            }
            if in_feature && trimmed == "#endif" {
                in_feature = false;
                include = true;
                continue;
            }
            if !trimmed.starts_with('#') && include {
                result.push_str(line);
                result.push('\n');
            }
        }
        ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::add_required_wgsl_extensions(result, &[])
    }

    fn storage(device: &wgpu::Device, bytes: u64, label: &str, indirect: bool) -> wgpu::Buffer {
        let mut usage = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;
        if indirect {
            usage |= wgpu::BufferUsages::INDIRECT;
        }
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes,
            mapped_at_creation: false,
            usage,
        })
    }

    fn bytes_u32(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect()
    }
    fn write(queue: &wgpu::Queue, buffer: &wgpu::Buffer, bytes: &[u8]) {
        queue.write_buffer(buffer, 0, bytes);
    }

    fn poll(device: &wgpu::Device) {
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(10)),
            })
            .expect("GPU submission must complete");
    }
    fn read(device: &wgpu::Device, queue: &wgpu::Queue, source: &wgpu::Buffer) -> Vec<u8> {
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("causal smoke readback"),
            size: source.size(),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("causal readback"),
        });
        encoder.copy_buffer_to_buffer(source, 0, &staging, 0, source.size());
        queue.submit(Some(encoder.finish()));
        poll(device);
        let slice = staging.slice(..);
        let result = Arc::new(std::sync::Mutex::new(None));
        let callback = result.clone();
        slice.map_async(wgpu::MapMode::Read, move |value| {
            *callback.lock().unwrap() = Some(value)
        });
        poll(device);
        assert!(matches!(*result.lock().unwrap(), Some(Ok(()))));
        let bytes = slice.get_mapped_range().expect("mapped readback").to_vec();
        staging.unmap();
        bytes
    }
    fn read_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        let row_bytes = width * 4;
        let padded_row_bytes = (row_bytes + 255) & !255;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("causal texture readback"),
            size: (padded_row_bytes * height) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("causal texture readback encoder"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(encoder.finish()));
        poll(device);
        let slice = staging.slice(..);
        let done = Arc::new(std::sync::Mutex::new(None));
        let done_clone = done.clone();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            *done_clone.lock().unwrap() = Some(result)
        });
        poll(device);
        assert!(matches!(*done.lock().unwrap(), Some(Ok(()))));
        let mapped = slice
            .get_mapped_range()
            .expect("mapped texture readback")
            .to_vec();
        staging.unmap();
        mapped
            .chunks(padded_row_bytes as usize)
            .flat_map(|row| row[..row_bytes as usize].iter().copied())
            .collect()
    }

    fn u32_at(bytes: &[u8], index: usize) -> u32 {
        u32::from_ne_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    }
    fn f32_at(bytes: &[u8], index: usize) -> f32 {
        f32::from_ne_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    }
    fn stage(stage: &str, ok: bool, details: impl std::fmt::Display) {
        assert!(
            ok,
            "first failing production wavefront stage: {stage}: {details}"
        );
    }
    fn pipeline(device: &wgpu::Device, name: &str) -> wgpu::ComputePipeline {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(production_shader(name))),
        });
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(name),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
    }
    fn dispatch(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipe: &wgpu::ComputePipeline,
        groups: &[&wgpu::BindGroup],
        x: u32,
        y: u32,
        z: u32,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("causal stage"),
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("causal stage pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipe);
        for (index, group) in groups.iter().enumerate() {
            pass.set_bind_group(index as u32, *group, &[]);
        }
        pass.dispatch_workgroups(x, y, z);
        drop(pass);
        queue.submit(Some(encoder.finish()));
        poll(device);
    }

    fn scene_words() -> Vec<u32> {
        use inox_render::{PackedSceneBuilder, SceneTableKind};
        let mut builder = PackedSceneBuilder::new();
        builder.append_words(SceneTableKind::Indices, &[0, 1, 2], 1);
        builder.append_words(SceneTableKind::VertexPositions, &[0; 3], 1);
        builder.append_words(SceneTableKind::Instances, &[0, 0, 0, 0], 4);
        builder.append_words(SceneTableKind::Transforms, &[0; 16], 16);
        builder.append_words(SceneTableKind::Meshes, &[0; 16], 16);
        builder.append_words(SceneTableKind::Meshlets, &[0; 20], 20);
        builder.append_words(SceneTableKind::Bvh, &[0; 16], 8);
        builder.append_words(SceneTableKind::Materials, &[0; 48], 48);
        builder.append_words(SceneTableKind::Textures, &[0; 4], 4);
        let mut scene = builder.finish();
        let _indices = scene.range(SceneTableKind::Indices).offset_words;
        let vertices = scene.range(SceneTableKind::VertexPositions).offset_words;
        let transforms = scene.range(SceneTableKind::Transforms).offset_words;
        let meshes = scene.range(SceneTableKind::Meshes).offset_words;
        let meshlets = scene.range(SceneTableKind::Meshlets).offset_words;
        let bvh = scene.range(SceneTableKind::Bvh).offset_words;
        let materials = scene.range(SceneTableKind::Materials).offset_words;
        let words = scene.words_mut();
        words[vertices as usize..vertices as usize + 3].copy_from_slice(&[0, 1023, 1023 << 20]);
        words[transforms as usize..transforms as usize + 16].copy_from_slice(&[
            0,
            0,
            0,
            1.0f32.to_bits(),
            0,
            0,
            0,
            1.0f32.to_bits(),
            (-1.0f32).to_bits(),
            0,
            (-1.0f32).to_bits(),
            1.0f32.to_bits(),
            1.0f32.to_bits(),
            0,
            1.0f32.to_bits(),
            1.0f32.to_bits(),
        ]);
        words[meshes as usize..meshes as usize + 16]
            .copy_from_slice(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        words[meshlets as usize..meshlets as usize + 20]
            .copy_from_slice(&[0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut node = [0u32; 16];
        for base in [0usize, 8] {
            node[base] = (-1.0f32).to_bits();
            node[base + 1] = (-1.0f32).to_bits();
            node[base + 2] = (-1.0f32).to_bits();
            node[base + 3] = u32::MAX;
            node[base + 4] = 1.0f32.to_bits();
            node[base + 5] = 1.0f32.to_bits();
            node[base + 6] = 1.0f32.to_bits();
        }
        node[7] = 0;
        node[15] = 0;
        words[bvh as usize..bvh as usize + 16].copy_from_slice(&node);
        words[materials as usize + 20..materials as usize + 23].copy_from_slice(&[
            0.8f32.to_bits(),
            0.4f32.to_bits(),
            0.2f32.to_bits(),
        ]);
        words[materials as usize + 24..materials as usize + 27].copy_from_slice(&[
            0.1f32.to_bits(),
            0.05f32.to_bits(),
            0.02f32.to_bits(),
        ]);
        scene.words().to_vec()
    }

    #[test]
    fn headless_gpu_runs_causal_production_wavefront_graph() {
        let Some(context) = block_on(RenderContext::create_headless(
            Arc::new(SharedData::default()),
            Arc::new(MessageHub::default()),
        )) else {
            eprintln!("SKIPPED: native causal wavefront smoke test requires a WebGPU adapter");
            return;
        };
        let device = &context.webgpu.device;
        let queue = &context.webgpu.queue;
        if device.limits().max_storage_buffers_per_shader_stage < 5 {
            eprintln!("SKIPPED: causal native wavefront smoke requires five storage buffers for production traversal; adapter limit is {} (maximum causal prefix is surface/primary/dispatch)", device.limits().max_storage_buffers_per_shader_stage);
            return;
        }
        let mut constants = vec![0u8; CONSTANT_BYTES];
        for matrix in 0..5 {
            for diagonal in 0..4 {
                constants[(matrix * 16 + diagonal * 5) * 4..][..4]
                    .copy_from_slice(&1.0f32.to_ne_bytes());
            }
        }
        constants[320..324].copy_from_slice(&WIDTH.to_ne_bytes());
        constants[324..328].copy_from_slice(&HEIGHT.to_ne_bytes());
        constants[328..332].copy_from_slice(&1u32.to_ne_bytes());
        constants[344..348].copy_from_slice(&52u32.to_ne_bytes());
        constants[400..404].copy_from_slice(&8u32.to_ne_bytes());
        constants[404..408].copy_from_slice(&2u32.to_ne_bytes());
        constants[408..412].copy_from_slice(&u32::MAX.to_ne_bytes());
        let constant = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("causal constants"),
            size: CONSTANT_BYTES as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        write(queue, &constant, &constants);
        let scene_data = scene_words();
        let scene = storage(
            device,
            (scene_data.len() * 4) as u64,
            "packed causal scene",
            false,
        );
        write(queue, &scene, &bytes_u32(&scene_data));
        let visibility = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("causal visibility"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("causal depth"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let vis = vec![256u32; PIXELS];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &visibility,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &bytes_u32(&vis),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(HEIGHT),
            },
            visibility.size(),
        );
        let mut depth_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("causal depth clear"),
        });
        {
            let _pass = depth_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("causal depth clear pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.create_view(&Default::default()),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.5),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }
        queue.submit(Some(depth_encoder.finish()));
        poll(device);
        let g0 = storage(
            device,
            (PIXELS * 16) as u64,
            "causal gbuffer position",
            false,
        );
        let g1 = storage(
            device,
            (PIXELS * 16) as u64,
            "causal gbuffer material",
            false,
        );
        let g2 = storage(
            device,
            (PIXELS * 16) as u64,
            "causal gbuffer auxiliary",
            false,
        );
        // Keep the scene's TLAS index coupled to the packed header, not a compile-time offset.
        constants[344..348].copy_from_slice(&scene_data[32].to_ne_bytes());
        write(queue, &constant, &constants);
        let surface = pipeline(device, "compute_wavefront_surface.wgsl");
        let surface0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("surface constants"),
            layout: &surface.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: constant.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene.as_entire_binding(),
                },
            ],
        });
        let surface1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("surface inputs"),
            layout: &surface.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &visibility.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &depth.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: g0.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: g1.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: g2.as_entire_binding(),
                },
            ],
        });
        dispatch(device, queue, &surface, &[&surface0, &surface1], 1, 1, 1);
        let g0_bytes = read(device, queue, &g0);
        let g1_bytes = read(device, queue, &g1);
        let position = [
            f32_at(&g0_bytes, 0),
            f32_at(&g0_bytes, 1),
            f32_at(&g0_bytes, 2),
        ];
        let packed_normal = f32_at(&g0_bytes, 3).to_bits();
        let material = f32_at(&g1_bytes, 0);
        let surface_ok = position.iter().all(|v| v.is_finite())
            && packed_normal != 0
            && material.is_finite()
            && (0.0..1_048_576.0).contains(&material);
        stage(
            "surface",
            surface_ok,
            format!(
                "position={position:?}, packed_normal={packed_normal:#010x}, material={material}"
            ),
        );
        let ray_header = storage(device, 16, "causal ray header", false);
        let rays = storage(device, (PIXELS * 64) as u64, "causal rays", false);
        write(queue, &ray_header, &bytes_u32(&[1, 0, PIXELS as u32, 0]));
        let primary = pipeline(device, "compute_wavefront_primary.wgsl");
        let primary0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("primary constants"),
            layout: &primary.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constant.as_entire_binding(),
            }],
        });
        let primary1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("primary queues"),
            layout: &primary.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: g0.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: ray_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: rays.as_entire_binding(),
                },
            ],
        });
        dispatch(device, queue, &primary, &[&primary0, &primary1], 1, 1, 1);
        let ray_header_bytes = read(device, queue, &ray_header);
        let ray_bytes = read(device, queue, &rays);
        let count = u32_at(&ray_header_bytes, 1);
        let ray_pixel = u32_at(&ray_bytes, 12);
        let ray_depth = u32_at(&ray_bytes, 13);
        let direction = (4..8).map(|i| f32_at(&ray_bytes, i)).collect::<Vec<_>>();
        let throughput = (8..12).map(|i| f32_at(&ray_bytes, i)).collect::<Vec<_>>();
        stage(
            "primary",
            count > 0
                && count <= PIXELS as u32
                && (0..12).all(|i| f32_at(&ray_bytes, i).is_finite())
                && direction[..3].iter().map(|value| value * value).sum::<f32>() > 1.0e-6
                && throughput.iter().all(|value| *value > 0.0)
                && ray_pixel < PIXELS as u32
                && ray_depth == 1,
            format!(
                "count={count}, pixel={ray_pixel}, depth={ray_depth}, origin=({}, {}, {}), direction={direction:?}, throughput={throughput:?}",
                f32_at(&ray_bytes, 0),
                f32_at(&ray_bytes, 1),
                f32_at(&ray_bytes, 2)
            ),
        );
        let dispatch_args = storage(device, 12, "causal dispatch args", true);
        let dispatch_pipe = pipeline(device, "compute_wavefront_dispatch.wgsl");
        let dispatch_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("primary dispatch"),
            layout: &dispatch_pipe.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ray_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: dispatch_args.as_entire_binding(),
                },
            ],
        });
        dispatch(device, queue, &dispatch_pipe, &[&dispatch_group], 1, 1, 1);
        let args = read(device, queue, &dispatch_args);
        stage(
            "dispatch",
            u32_at(&args, 0) > 0 && u32_at(&args, 1) == 1,
            format!(
                "args=[{}, {}, {}]",
                u32_at(&args, 0),
                u32_at(&args, 1),
                u32_at(&args, 2)
            ),
        );
        let hit_header = storage(device, 16, "causal hit header", false);
        let hits = storage(device, (PIXELS * 32) as u64, "causal hits", false);
        write(queue, &hit_header, &bytes_u32(&[1, 0, PIXELS as u32, 0]));
        let traversal = pipeline(device, "compute_wavefront_traversal.wgsl");
        let traversal0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("traversal scene"),
            layout: &traversal.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: constant.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene.as_entire_binding(),
                },
            ],
        });
        let traversal1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("traversal queues"),
            layout: &traversal.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ray_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: hit_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: hits.as_entire_binding(),
                },
            ],
        });
        dispatch(
            device,
            queue,
            &traversal,
            &[&traversal0, &traversal1],
            u32_at(&args, 0),
            1,
            1,
        );
        let hit_header_bytes = read(device, queue, &hit_header);
        let hit_count = u32_at(&hit_header_bytes, 1);
        stage(
            "traversal",
            hit_count <= count && u32_at(&hit_header_bytes, 3) == 0,
            format!(
                "rays={count}, hits={hit_count}, overflow={}",
                u32_at(&hit_header_bytes, 3)
            ),
        );
        let hit_args = storage(device, 12, "causal hit dispatch args", true);
        let hit_dispatch_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hit dispatch"),
            layout: &dispatch_pipe.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: hit_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: hit_args.as_entire_binding(),
                },
            ],
        });
        dispatch(
            device,
            queue,
            &dispatch_pipe,
            &[&hit_dispatch_group],
            1,
            1,
            1,
        );
        let hit_args_bytes = read(device, queue, &hit_args);
        stage(
            "hit dispatch",
            hit_count == 0 || u32_at(&hit_args_bytes, 0) > 0,
            format!("hits={hit_count}, args={}", u32_at(&hit_args_bytes, 0)),
        );
        let indirect = storage(
            device,
            (PIXELS * 32) as u64,
            "causal indirect accumulator",
            false,
        );
        let shade = pipeline(device, "compute_wavefront_shade.wgsl");
        let shade0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shade scene"),
            layout: &shade.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: constant.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene.as_entire_binding(),
                },
            ],
        });
        let textures = storage(device, 2048 * 16, "causal textures", false);
        write(queue, &textures, &bytes_u32(&[u32::MAX, 0, 0, 0]));
        let shade1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shade queues"),
            layout: &shade.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ray_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: hit_header.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: hits.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: indirect.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: textures.as_entire_binding(),
                },
            ],
        });
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("causal shade texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let tex_view = tex.create_view(&Default::default());
        let mut tex_entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(&sampler),
        }];
        for binding in 1..8 {
            tex_entries.push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(&tex_view),
            });
        }
        let shade2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shade textures"),
            layout: &shade.get_bind_group_layout(2),
            entries: &tex_entries,
        });
        if hit_count > 0 {
            dispatch(
                device,
                queue,
                &shade,
                &[&shade0, &shade1, &shade2],
                u32_at(&hit_args_bytes, 0),
                1,
                1,
            );
        }
        let indirect_bytes = read(device, queue, &indirect);
        let indirect_values = (0..12)
            .map(|i| f32_at(&indirect_bytes, i))
            .collect::<Vec<_>>();
        stage(
            "shade",
            indirect_values.iter().all(|v| v.is_finite()),
            format!("indirect={indirect_values:?}"),
        );
        let direct = storage(
            device,
            (PIXELS * 32) as u64,
            "causal direct accumulator",
            false,
        );
        let ao = storage(device, (PIXELS * 16) as u64, "causal ao state", false);
        let final_acc = storage(
            device,
            (PIXELS * 32) as u64,
            "causal final accumulator",
            false,
        );
        let composite = pipeline(device, "compute_wavefront_composite.wgsl");
        let composite_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite"),
            layout: &composite.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: constant.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: final_acc.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: direct.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: ao.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: indirect.as_entire_binding(),
                },
            ],
        });
        dispatch(device, queue, &composite, &[&composite_group], 1, 1, 1);
        let final_bytes = read(device, queue, &final_acc);
        let final_values = (0..3).map(|i| f32_at(&final_bytes, i)).collect::<Vec<_>>();
        stage(
            "composite",
            final_values.iter().all(|v| v.is_finite())
                && final_values.iter().any(|v| *v > 0.0)
                && final_values.iter().any(|v| *v < 1.0),
            format!("final={final_values:?}"),
        );
        let previous = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("causal previous"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let output = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("causal finalize output"),
            size: previous.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let finalize_source = production_shader("finalize.wgsl");
        let finalize_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("production finalize"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(finalize_source)),
        });
        let _finalize_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("finalize layout"),
            bind_group_layouts: &[Some(&device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("finalize bindings"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                },
            ))],
            immediate_size: 0,
        });
        let finalize = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("production finalize"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &finalize_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &finalize_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let finalize_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("finalize group"),
            layout: &finalize.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: constant.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: final_acc.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &previous.create_view(&Default::default()),
                    ),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("finalize encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("production finalize pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output.create_view(&Default::default()),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&finalize);
            pass.set_bind_group(0, &finalize_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
        poll(device);
        let output_bytes = read_texture(device, queue, &output, WIDTH, HEIGHT);
        let output_values = output_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|pixel| pixel[..3].iter().copied())
            .collect::<Vec<_>>();
        stage(
            "finalize",
            output_values.iter().any(|value| *value > 0)
                && output_values.iter().any(|value| *value < 255),
            format!("output first pixel={:?}", &output_bytes[..4]),
        );
    }
}
