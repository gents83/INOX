use std::path::PathBuf;

use image::{DynamicImage, GenericImage, GenericImageView, Pixel};

use crate::api::backend;

use super::device::*;

pub struct Texture {
    pub inner: backend::Texture,
}

impl Texture {
    pub fn create(device: &Device, filepath: PathBuf) -> Self {
        Self {
            inner: backend::Texture::create_from(&device.inner, filepath.to_str().unwrap()),
        }
    }
    pub fn empty(device: &Device) -> Self {
        let mut image_data = DynamicImage::new_rgba8(1, 1);
        let (width, height) = image_data.dimensions();
        for x in 0..width {
            for y in 0..height {
                image_data.put_pixel(x, y, Pixel::from_channels(255, 255, 255, 255))
            }
        }
        Self {
            inner: backend::Texture::create(&device.inner, &image_data, 1),
        }
    }
}
