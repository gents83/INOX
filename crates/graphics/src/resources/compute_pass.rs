use std::path::PathBuf;

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedDataRc,
};

use crate::{BindingData, CommandBuffer, ComputePassData, ComputePipeline, RenderContext};

pub type ComputePassId = ResourceId;

#[derive(Clone)]
pub struct ComputePass {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    name: String,
    pipelines: Vec<Resource<ComputePipeline>>,
    is_initialized: bool,
}

impl ResourceTrait for ComputePass {
    fn invalidate(&mut self) -> &mut Self {
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
            pipelines: Vec::new(),
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
            pipelines: Vec::new(),
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

    pub fn init(&mut self, render_context: &RenderContext, binding_data: &mut BindingData) {
        let mut is_initialized = false;
        binding_data.set_bind_group_layout();
        self.pipelines.iter().for_each(|pipeline| {
            is_initialized |= pipeline.get_mut().init(render_context, binding_data);
        });
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
        let mut is_ready = true;

        let label = format!("ComputePass {}", self.name);
        let pipelines = self.pipelines().iter().map(|h| h.get()).collect::<Vec<_>>();
        let mut compute_pass = {
            inox_profiler::gpu_scoped_profile!(
                &mut command_buffer.encoder,
                &_render_context.core.device,
                "encoder::begin_compute_pass",
            );
            command_buffer
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(label.as_str()),
                    ..Default::default()
                })
        };

        pipelines.iter().for_each(|pipeline| {
            is_ready &= pipeline.is_initialized();
            if is_ready {
                inox_profiler::gpu_scoped_profile!(
                    &mut compute_pass,
                    &_render_context.core.device,
                    "compute_pass::set_pipeline",
                );
                compute_pass.set_pipeline(pipeline.compute_pipeline());
            }
        });

        if !is_ready {
            return;
        }

        binding_data.set_bind_groups();

        binding_data
            .bind_groups()
            .iter()
            .enumerate()
            .for_each(|(index, bind_group)| {
                inox_profiler::gpu_scoped_profile!(
                    &mut compute_pass,
                    &_render_context.core.device,
                    "compute_pass::set_bind_group",
                );
                compute_pass.set_bind_group(index as _, bind_group, &[]);
            });

        inox_profiler::gpu_scoped_profile!(
            &mut compute_pass,
            &_render_context.core.device,
            "compute_pass::dispatch_workgroups",
        );
        compute_pass.dispatch_workgroups(x, y, z);
        drop(compute_pass);
    }
}
