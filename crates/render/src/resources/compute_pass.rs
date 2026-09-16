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
        self.cached_pipelines.clear();
        binding_data.set_bind_group_layout();
        let mut is_initialized = !self.pipelines.is_empty();
        for pipeline in &self.pipelines {
            let mut pipeline = pipeline.get_mut();
            if pipeline.init(render_context, binding_data, entry_point_name) {
                self.cached_pipelines
                    .push(pipeline.compute_pipeline().clone());
            } else {
                is_initialized = false;
            }
        }
        if !is_initialized {
            self.cached_pipelines.clear();
        }
        self.is_initialized = is_initialized;
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
