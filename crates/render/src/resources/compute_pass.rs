use std::{collections::HashMap, path::PathBuf};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedDataRc,
};

use crate::{
    BindingData, BufferId, BufferRef, CommandBuffer, ComputePassData, ComputePipeline,
    RenderContext,
};

pub type ComputePassId = ResourceId;

#[derive(Clone)]
pub struct ComputePass {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    name: String,
    label: String,
    pipelines: Vec<Resource<ComputePipeline>>,
    cached_pipelines: Vec<wgpu::ComputePipeline>,
    is_initialized: bool,
}

impl ResourceTrait for ComputePass {
    fn invalidate(&mut self) -> &mut Self {
        self.cached_pipelines.clear();
        self.is_initialized = false;
        self
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl DataTypeResource for ComputePass {
    type DataType = ComputePassData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            name: String::new(),
            label: "ComputePass".to_string(),
            pipelines: Vec::new(),
            cached_pipelines: Vec::new(),
            is_initialized: false,
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        _id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut pass = Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            name: data.name.clone(),
            label: format!("ComputePass {}", data.name),
            pipelines: Vec::new(),
            cached_pipelines: Vec::new(),
            is_initialized: false,
        };
        pass.set_pipelines(&data.pipelines);
        pass
    }
}

impl ComputePass {
    pub fn pipelines(&self) -> &[Resource<ComputePipeline>] {
        self.pipelines.as_slice()
    }
    pub fn set_pipelines(&mut self, pipelines: &[PathBuf]) -> &mut Self {
        self.pipelines.clear();
        self.cached_pipelines.clear();
        self.is_initialized = false;
        pipelines.iter().for_each(|path| {
            if !path.as_os_str().is_empty() {
                let pipeline = ComputePipeline::request_load(
                    &self.shared_data,
                    &self.message_hub,
                    path.as_path(),
                    None,
                );
                self.pipelines.push(pipeline);
            };
        });
        self
    }

    pub fn init(
        &mut self,
        render_context: &RenderContext,
        binding_data: &mut BindingData,
        entry_point_name: Option<&str>,
    ) {
        let previous_pipelines = self.cached_pipelines.clone();
        let had_valid_cache =
            self.is_initialized && previous_pipelines.len() == self.pipelines.len();
        let mut next_pipelines = Vec::with_capacity(self.pipelines.len());
        binding_data.set_bind_group_layout();
        let mut all_initialized = !self.pipelines.is_empty();
        for pipeline in &self.pipelines {
            let mut pipeline = pipeline.get_mut();
            if pipeline.init(render_context, binding_data, entry_point_name) {
                next_pipelines.push(pipeline.compute_pipeline().clone());
            } else {
                all_initialized = false;
            }
        }

        if all_initialized && next_pipelines.len() == self.pipelines.len() {
            self.cached_pipelines = next_pipelines;
            self.is_initialized = true;
        } else if !had_valid_cache {
            self.cached_pipelines.clear();
            self.is_initialized = false;
        }
    }

    pub fn dispatch<'a>(
        &'a self,
        _render_context: &RenderContext,
        binding_data: &'a mut BindingData,
        command_buffer: &'a mut CommandBuffer,
        x: u32,
        y: u32,
        z: u32,
    ) {
        if !self.is_initialized {
            return;
        }

        let mut compute_pass = {
            inox_profiler::gpu_scoped_profile!(
                &mut command_buffer.encoder,
                &_render_context.webgpu.device,
                "encoder::begin_compute_pass",
            );
            command_buffer
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(self.label.as_str()),
                    ..Default::default()
                })
        };

        for pipeline in &self.cached_pipelines {
            inox_profiler::gpu_scoped_profile!(
                &mut compute_pass,
                &_render_context.webgpu.device,
                "compute_pass::set_pipeline",
            );
            compute_pass.set_pipeline(pipeline);
        }

        binding_data.set_bind_groups();

        binding_data
            .bind_groups()
            .iter()
            .enumerate()
            .for_each(|(index, bind_group)| {
                inox_profiler::gpu_scoped_profile!(
                    &mut compute_pass,
                    &_render_context.webgpu.device,
                    "compute_pass::set_bind_group",
                );
                compute_pass.set_bind_group(index as _, Some(bind_group), &[]);
            });

        inox_profiler::gpu_scoped_profile!(
            &mut compute_pass,
            &_render_context.webgpu.device,
            "compute_pass::dispatch_workgroups",
        );
        compute_pass.dispatch_workgroups(x, y, z);
        drop(compute_pass);
    }

    pub fn dispatch_indirect<'a>(
        &'a self,
        _render_context: &RenderContext,
        binding_data: &'a mut BindingData,
        command_buffer: &'a mut CommandBuffer,
        buffers: &'a HashMap<BufferId, BufferRef>,
        indirect_args_id: BufferId,
    ) {
        if !self.is_initialized {
            return;
        }

        let mut compute_pass = {
            inox_profiler::gpu_scoped_profile!(
                &mut command_buffer.encoder,
                &_render_context.webgpu.device,
                "encoder::begin_compute_pass",
            );
            command_buffer
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(self.label.as_str()),
                    ..Default::default()
                })
        };

        for pipeline in &self.cached_pipelines {
            inox_profiler::gpu_scoped_profile!(
                &mut compute_pass,
                &_render_context.webgpu.device,
                "compute_pass::set_pipeline",
            );
            compute_pass.set_pipeline(pipeline);
        }

        binding_data.set_bind_groups();

        binding_data
            .bind_groups()
            .iter()
            .enumerate()
            .for_each(|(index, bind_group)| {
                inox_profiler::gpu_scoped_profile!(
                    &mut compute_pass,
                    &_render_context.webgpu.device,
                    "compute_pass::set_bind_group",
                );
                compute_pass.set_bind_group(index as _, Some(bind_group), &[]);
            });

        inox_profiler::gpu_scoped_profile!(
            &mut compute_pass,
            &_render_context.webgpu.device,
            "compute_pass::dispatch_workgroups",
        );
        compute_pass.dispatch_workgroups_indirect(
            buffers
                .get(&indirect_args_id)
                .unwrap()
                .gpu_buffer()
                .unwrap(),
            0,
        );
        drop(compute_pass);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use futures::executor::block_on;
    use inox_messenger::MessageHub;
    use inox_resources::{
        DataTypeResource, ResourceHandle, ResourceTrait, SerializableResource, SharedData,
    };
    use inox_uid::generate_random_uid;

    use crate::{ComputePipeline, Shader, ShaderData};

    use super::ComputePass;

    const VALID_SHADER: &str = "@compute @workgroup_size(1) fn main() {}";

    #[test]
    fn dispatch_retains_complete_cache_when_reload_fails() {
        let shared_data = Arc::new(SharedData::default());
        let message_hub = Arc::new(MessageHub::default());
        let context = Arc::new(
            block_on(crate::RenderContext::create_headless(
                shared_data.clone(),
                message_hub.clone(),
            ))
            .expect("headless WebGPU adapter/device required for compute pass tests"),
        );

        let shader_path = Path::new("compute_pass_reload.wgsl");
        let mut shader = Shader::create_from_data(
            &shared_data,
            &message_hub,
            generate_random_uid(),
            &ShaderData {
                spirv_code: Vec::new(),
                wgsl_code: VALID_SHADER.to_string(),
            },
        );
        shader.set_path(shader_path);
        let shader = Arc::new(ResourceHandle::new(generate_random_uid(), shader));
        assert!(shader.get_mut().init(&context).is_ok());

        let pipeline = ComputePipeline::from_shader(
            "compute_pass_reload.compute_pipeline".into(),
            &shared_data,
            &message_hub,
            shader.clone(),
        );
        let pipeline = Arc::new(ResourceHandle::new(generate_random_uid(), pipeline));
        let mut pass = <ComputePass as DataTypeResource>::new(
            generate_random_uid(),
            &shared_data,
            &message_hub,
        );
        pass.pipelines.push(pipeline);

        let mut binding_data = crate::BindingData::new(&context, "compute-pass-reload");
        pass.init(&context, &mut binding_data, None);
        assert!(pass.is_initialized());

        let invalid = Shader::create_from_data(
            &shared_data,
            &message_hub,
            generate_random_uid(),
            &ShaderData {
                spirv_code: Vec::new(),
                wgsl_code: "@compute @workgroup_size(1) fn other() {}".to_string(),
            },
        );
        let invalid = ResourceHandle::new(generate_random_uid(), invalid);
        shader.get_mut().replace_from(&invalid.get());
        pass.pipelines[0]
            .get_mut()
            .check_shaders_to_reload(shader_path.to_string_lossy().into_owned());

        let mut binding_data = crate::BindingData::new(&context, "compute-pass-reload");
        pass.init(&context, &mut binding_data, None);
        assert!(pass.is_initialized());

        let device = &context.webgpu.device;
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut command_buffer = crate::CommandBuffer {
            encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Pass Reload Test Encoder"),
            }),
        };
        pass.dispatch(&context, &mut binding_data, &mut command_buffer, 1, 1, 1);
        context
            .webgpu
            .queue
            .submit(Some(command_buffer.encoder.finish()));
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(10)),
            })
            .expect("compute submission must complete");
        assert!(block_on(error_scope.pop()).is_none());
    }
}
