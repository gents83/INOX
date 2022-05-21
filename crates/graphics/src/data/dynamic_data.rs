use std::mem::size_of;

use inox_resources::HashBuffer;

use crate::{
    AsBufferBinding, DataBuffer, LightData, LightId, Material, MaterialId, RenderContext,
    ShaderMaterialData, TextureData, TextureId, TextureType, INVALID_INDEX, MAX_NUM_LIGHTS,
    MAX_NUM_MATERIALS, MAX_NUM_TEXTURES,
};

#[derive(Default)]
pub struct DynamicData {
    pub textures_data: HashBuffer<TextureId, TextureData, MAX_NUM_TEXTURES>,
    pub materials_data: HashBuffer<MaterialId, ShaderMaterialData, MAX_NUM_MATERIALS>,
    pub lights_data: HashBuffer<LightId, LightData, MAX_NUM_LIGHTS>,
}
impl DynamicData {
    pub fn set_material_data(&mut self, material_id: &MaterialId, material: &Material) -> usize {
        let mut textures_indices = [INVALID_INDEX; TextureType::Count as _];
        material
            .textures()
            .iter()
            .enumerate()
            .for_each(|(i, handle_texture)| {
                if let Some(texture) = handle_texture {
                    textures_indices[i] = texture.get().uniform_index() as _;
                }
            });
        let shader_material_data = ShaderMaterialData {
            textures_indices,
            textures_coord_set: *material.textures_coords_set(),
            roughness_factor: material.roughness_factor(),
            metallic_factor: material.metallic_factor(),
            alpha_cutoff: material.alpha_cutoff(),
            alpha_mode: material.alpha_mode() as _,
            base_color: material.base_color().into(),
            emissive_color: material.emissive_color().into(),
            occlusion_strength: material.occlusion_strength(),
            diffuse_color: material.diffuse_color().into(),
            specular_color: material.specular_color().into(),
        };
        self.materials_data
            .insert(material_id, shader_material_data)
    }
}

impl AsBufferBinding for DynamicData {
    fn size(&self) -> u64 {
        let total_size = size_of::<TextureData>() * MAX_NUM_TEXTURES
            + size_of::<ShaderMaterialData>() * MAX_NUM_MATERIALS
            + size_of::<LightData>() * MAX_NUM_LIGHTS;
        total_size as _
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_context, self.textures_data.data());
        buffer.add_to_gpu_buffer(render_context, self.materials_data.data());
        buffer.add_to_gpu_buffer(render_context, self.lights_data.data());
    }
}
