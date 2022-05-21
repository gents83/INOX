use inox_core::{implement_unique_system_uid, ContextRc, System};

use inox_resources::{Handle, Resource};

use crate::{ComputePass, GraphicsData, VertexFormatBits, GRAPHICS_DATA_UID};

pub const CULLING_UPDATE: &str = "CULLING_UPDATE";

pub struct CullingSystem {
    _culling_compute_pass: Resource<ComputePass>,
    _graphics_data: Handle<GraphicsData>,
    _vertex_format: VertexFormatBits,
}

impl CullingSystem {
    pub fn new(
        context: &ContextRc,
        culling_compute_pass: &Resource<ComputePass>,
        vertex_format: VertexFormatBits,
    ) -> Self {
        Self {
            _culling_compute_pass: culling_compute_pass.clone(),
            _graphics_data: context
                .shared_data()
                .get_resource::<GraphicsData>(&GRAPHICS_DATA_UID),
            _vertex_format: vertex_format,
        }
    }
}

unsafe impl Send for CullingSystem {}
unsafe impl Sync for CullingSystem {}

implement_unique_system_uid!(CullingSystem);

impl System for CullingSystem {
    fn read_config(&mut self, _plugin_name: &str) {}

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        /*
        if let Some(pipeline) = self.culling_compute_pass.get().pipeline(0) {
            if let Some(graphics_data) = self.graphics_data.as_ref() {
                if let Some(meshlets) = graphics_data.get().get_meshlets(&self.vertex_format) {
                    pipeline
                    .get_mut()
                    .binding_data_mut()
                    .set_custom_data_array::<MeshletData>(meshlets, true);
                }
            }
        }
        */
        true
    }
    fn uninit(&mut self) {}
}
