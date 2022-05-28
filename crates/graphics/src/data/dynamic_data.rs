use std::mem::size_of;

use inox_resources::HashBuffer;

use crate::{
    AsBufferBinding, DataBuffer, LightData, LightId, Material, MaterialId, RenderCoreContext,
    ShaderMaterialData, TextureData, TextureId, TextureType, INVALID_INDEX, MAX_NUM_LIGHTS,
    MAX_NUM_MATERIALS, MAX_NUM_TEXTURES,
};

#[derive(Default)]
pub struct DynamicData {
    textures_data: HashBuffer<TextureId, TextureData, MAX_NUM_TEXTURES>,
    materials_data: HashBuffer<MaterialId, ShaderMaterialData, MAX_NUM_MATERIALS>,
    lights_data: HashBuffer<LightId, LightData, MAX_NUM_LIGHTS>,
    is_dirty: bool,
}

impl DynamicData {
    pub fn add_texture_data(&mut self, id: &TextureId, data: TextureData) -> usize {
        self.set_dirty(true);
        self.textures_data.insert(id, data)
    }
    pub fn add_light_data(&mut self, id: &LightId, data: LightData) -> usize {
        self.set_dirty(true);
        self.lights_data.insert(id, data)
    }
    pub fn add_material_data(&mut self, material_id: &MaterialId, material: &Material) -> usize {
        self.set_dirty(true);
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
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        let total_size = size_of::<TextureData>() * MAX_NUM_TEXTURES
            + size_of::<ShaderMaterialData>() * MAX_NUM_MATERIALS
            + size_of::<LightData>() * MAX_NUM_LIGHTS;
        total_size as _
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, self.textures_data.data());
        buffer.add_to_gpu_buffer(render_core_context, self.materials_data.data());
        buffer.add_to_gpu_buffer(render_core_context, self.lights_data.data());
    }
}
