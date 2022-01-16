use std::path::Path;

use sabi_profiler::debug_log;

use crate::{RenderContext, ShaderTextureData, TextureId};

use super::texture_atlas::TextureAtlas;

pub struct TextureHandler {
    texture_atlas: Vec<TextureAtlas>,
}

impl TextureHandler {
    pub fn create(context: &RenderContext) -> Self {
        Self {
            texture_atlas: vec![TextureAtlas::create_default(context)],
        }
    }

    pub fn add_render_target(
        &mut self,
        context: &RenderContext,
        id: &TextureId,
        width: u32,
        height: u32,
    ) {
        self.texture_atlas
            .push(TextureAtlas::create_texture(context, id, width, height, 1));
    }

    pub fn get_textures_atlas(&self) -> &[TextureAtlas] {
        self.texture_atlas.as_slice()
    }

    pub fn get_texture_atlas(&self, id: &TextureId) -> Option<&TextureAtlas> {
        if let Some(index) = self.texture_atlas.iter().position(|t| t.texture_id() == id) {
            return Some(&self.texture_atlas[index]);
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.texture_atlas.is_empty()
    }

    pub fn copy(&self, context: &RenderContext, id: &TextureId, _image_data: &mut [u8]) {
        sabi_profiler::scoped_profile!("texture::copy");

        self.texture_atlas.iter().for_each(|atlas| {
            if atlas.read_from_gpu(context, id) {
                todo!();
            }
        });
    }

    pub fn remove(&mut self, id: &TextureId) {
        let mut texture_to_remove = Vec::new();
        self.texture_atlas
            .iter_mut()
            .enumerate()
            .for_each(|(texture_index, atlas)| {
                if atlas.remove(texture_index as _, id) {
                    texture_to_remove.push(texture_index);
                }
            });
        texture_to_remove.iter().rev().for_each(|i| {
            let mut texture_atlas = self.texture_atlas.remove(*i);
            texture_atlas.destroy();
            debug_log(format!("Removing texture atlas {:?}", i).as_str());
        });
    }

    pub fn add_from_path(
        &mut self,
        context: &RenderContext,
        id: &TextureId,
        filepath: &Path,
    ) -> ShaderTextureData {
        let image = image::open(filepath).unwrap();
        self.add_image(
            context,
            id,
            image.width(),
            image.height(),
            image.to_rgba8().as_raw().as_slice(),
        )
    }

    pub fn add_image(
        &mut self,
        context: &RenderContext,
        id: &TextureId,
        width: u32,
        height: u32,
        image_data: &[u8],
    ) -> ShaderTextureData {
        for (texture_index, texture_atlas) in self.texture_atlas.iter_mut().enumerate() {
            if let Some(texture_data) =
                texture_atlas.allocate(context, id, texture_index as _, width, height, image_data)
            {
                return texture_data;
            }
        }
        panic!("Unable to allocate texture")
    }

    pub fn get_texture_data(&self, id: &TextureId) -> Option<ShaderTextureData> {
        for (texture_index, texture_atlas) in self.texture_atlas.iter().enumerate() {
            if let Some(texture_data) = texture_atlas.get_texture_data(texture_index as _, id) {
                return Some(texture_data);
            }
        }
        None
    }
}
