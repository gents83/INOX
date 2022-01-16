use std::path::Path;

use sabi_messenger::MessengerRw;
use sabi_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use sabi_serialize::read_from_file;

use crate::{GraphicsMesh, LoadOperation, Pipeline, RenderPassData, StoreOperation};

pub type RenderPassId = ResourceId;

#[derive(Default, Clone)]
pub struct RenderPass {
    data: RenderPassData,
    pipelines: Vec<Resource<Pipeline>>,
}

impl DataTypeResource for RenderPass {
    type DataType = RenderPassData;
    type OnCreateData = ();

    fn invalidate(&mut self) {}
    fn is_initialized(&self) -> bool {
        true
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _id: &RenderPassId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _id: &RenderPassId) {}

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut pipelines = Vec::new();
        data.pipelines.iter().for_each(|path| {
            if !path.as_os_str().is_empty() {
                let pipeline =
                    Pipeline::request_load(shared_data, global_messenger, path.as_path(), None);
                pipelines.push(pipeline);
            };
        });

        Self { data, pipelines }
    }
}

impl RenderPass {
    pub fn data(&self) -> &RenderPassData {
        &self.data
    }
    fn color_operations(&self, c: wgpu::Color) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.data.load_color {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(c),
            },
            store: matches!(&self.data.store_color, StoreOperation::Store),
        }
    }

    fn depth_operations(&self, c: wgpu::Color) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.data.load_depth {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(c),
            },
            store: matches!(&self.data.store_depth, StoreOperation::Store),
        }
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        graphics_mesh: &GraphicsMesh,
        constant_data_bind_group: &wgpu::BindGroup,
    ) {
        if graphics_mesh.vertex_count() == 0 {
            return;
        }
        let pipelines = self
            .pipelines
            .iter()
            .map(|h| h.get_mut())
            .collect::<Vec<_>>();
        let color_operations = self.color_operations(wgpu::Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: texture_view,
                resolve_target: None,
                ops: color_operations,
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_bind_group(0, constant_data_bind_group, &[]);

        if let Some(buffer_slice) = graphics_mesh.vertex_buffer() {
            render_pass.set_vertex_buffer(0, buffer_slice);
        }
        if let Some(buffer_slice) = graphics_mesh.index_buffer() {
            render_pass.set_index_buffer(buffer_slice, wgpu::IndexFormat::Uint32);
        }

        pipelines.iter().for_each(|pipeline| {
            let instance_count = pipeline.instance_count();
            if pipeline.is_initialized() && instance_count > 0 {
                if let Some(instance_buffer) = pipeline.instance_buffer() {
                    render_pass.set_vertex_buffer(1, instance_buffer);
                }
                render_pass.set_pipeline(pipeline.render_pipeline());

                if graphics_mesh.index_count() > 0 {
                    if let Some(indirect_buffer) = pipeline.indirect_buffer() {
                        render_pass.multi_draw_indexed_indirect(
                            indirect_buffer,
                            0,
                            instance_count as u32,
                        );
                    }
                } else {
                    render_pass.draw(
                        0..graphics_mesh.vertex_count() as u32,
                        0..instance_count as u32,
                    );
                }
            }
        });
    }
}
