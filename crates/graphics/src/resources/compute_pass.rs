use std::path::PathBuf;

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
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
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ComputePassId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ComputePassId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
        self.invalidate();
    }
}

impl DataTypeResource for ComputePass {
    type DataType = ComputePassData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            name: String::new(),
            pipelines: Vec::new(),
            is_initialized: false,
        }
    }
    fn invalidate(&mut self) -> &mut Self {
        self.is_initialized = false;
        self
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
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

    pub fn init(&mut self, render_context: &RenderContext, binding_data: &BindingData) {
        let mut is_initialized = false;
        self.pipelines.iter().for_each(|pipeline| {
            is_initialized |= pipeline.get_mut().init(render_context, binding_data);
        });
        self.is_initialized = is_initialized;
    }

    pub fn begin<'a>(
        &'a self,
        binding_data: &'a BindingData,
        command_buffer: &'a mut CommandBuffer,
    ) -> wgpu::ComputePass<'a> {
        let label = format!("ComputePass {}", self.name);
        let mut compute_pass =
            command_buffer
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(label.as_str()),
                });

        binding_data
            .bind_groups()
            .iter()
            .enumerate()
            .for_each(|(index, bind_group)| {
                compute_pass.set_bind_group(index as _, bind_group, &[]);
            });

        compute_pass
    }

    pub fn dispatch(&self, compute_pass: wgpu::ComputePass, x: u32, y: u32, z: u32) {
        let pipelines = self.pipelines().iter().map(|h| h.get()).collect::<Vec<_>>();
        {
            let mut is_ready = false;
            let mut compute_pass = compute_pass;
            pipelines.iter().for_each(|pipeline| {
                if pipeline.is_initialized() {
                    compute_pass.set_pipeline(pipeline.compute_pipeline());
                    is_ready = true;
                }
            });
            if is_ready {
                compute_pass.dispatch_workgroups(x, y, z);
            }
        }
    }
}
