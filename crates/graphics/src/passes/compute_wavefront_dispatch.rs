use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, GPUVector,
    Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{wavefront_primary_bindings, DispatchArgs, QueueHeader};

pub const COMPUTE_WAVEFRONT_DISPATCH_PIPELINE: &str =
    "pipelines/ComputeWavefrontDispatch.compute_pipeline";
pub const COMPUTE_WAVEFRONT_DISPATCH_NAME: &str = "ComputeWavefrontDispatchPass";
pub const WAVEFRONT_DISPATCH_ARGS_ID: Uid = generate_static_uid_from_string("WF_DISPATCH_ARGS");
pub const WAVEFRONT_HIT_DISPATCH_ARGS_ID: Uid =
    generate_static_uid_from_string("WF_HIT_DISPATCH_ARGS");

pub struct ComputeWavefrontDispatchPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    ray_queue_header: GPUVector<QueueHeader>,
    dispatch_args: GPUVector<DispatchArgs>,
    pass_name: String,
}

unsafe impl Send for ComputeWavefrontDispatchPass {}
unsafe impl Sync for ComputeWavefrontDispatchPass {}

impl Pass for ComputeWavefrontDispatchPass {
    fn name(&self) -> &str {
        &self.pass_name
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_DISPATCH_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        // Queue counts are written by an earlier GPU pass in the same command
        // buffer.  They are not available to the CPU until readback, so the
        // indirect dispatch shader must decide whether to launch workgroups.
        !self.ray_queue_header.read().unwrap().is_empty()
            && !self.dispatch_args.read().unwrap().is_empty()
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        Self::create_for_header(
            context,
            render_context,
            wavefront_primary_bindings().ray_header,
            wavefront_primary_bindings().dispatch_args,
            COMPUTE_WAVEFRONT_DISPATCH_NAME,
        )
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.ray_queue_header.read().unwrap().is_empty() {
            return;
        }
        self.binding_data.add_buffer(
            &mut *self.ray_queue_header.write().unwrap(),
            Some("Wavefront Ray Queue Header"),
            BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.dispatch_args.write().unwrap(),
            Some("Wavefront Dispatch Args"),
            BindingInfo {
                group_index: 0,
                binding_index: 1,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite | BindingFlags::Indirect,
                ..Default::default()
            },
        );
        self.compute_pass
            .get_mut()
            .init(render_context, &mut self.binding_data, None);
    }

    fn reset(&mut self, render_context: &RenderContext) {
        let mut args = self.dispatch_args.write().unwrap();
        if let Some(args) = args.first_mut() {
            *args = DispatchArgs::default();
        }
        render_context.update_buffer(&mut *args);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        {
            let mut args = self.dispatch_args.write().unwrap();
            if let Some(args) = args.first_mut() {
                *args = DispatchArgs::default();
            }
            render_context.update_buffer(&mut *args);
        }
        if !self.is_active(render_context) {
            return;
        }
        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            1,
            1,
            1,
        );
    }
}

impl ComputeWavefrontDispatchPass {
    pub fn create_for_header(
        context: &ContextRc,
        render_context: &RenderContextRc,
        header_id: Uid,
        dispatch_args_id: Uid,
        pass_name: &str,
    ) -> Self {
        let data = ComputePassData {
            name: pass_name.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_DISPATCH_PIPELINE)],
        };
        let ray_queue_header = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(header_id);
        let dispatch_args = render_context
            .global_buffers()
            .vector_with_id::<DispatchArgs>(dispatch_args_id);
        dispatch_args
            .write()
            .unwrap()
            .resize(1, DispatchArgs::default());

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, pass_name),
            ray_queue_header,
            dispatch_args,
            pass_name: pass_name.to_string(),
        }
    }
}
