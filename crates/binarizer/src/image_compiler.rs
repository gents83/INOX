use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use nrg_messenger::MessengerRw;
use nrg_profiler::debug_log;

const IMAGE_PNG_EXTENSION: &str = "png";
const IMAGE_JPG_EXTENSION: &str = "jpg";
const IMAGE_JPEG_EXTENSION: &str = "jpeg";
const IMAGE_BMP_EXTENSION: &str = "bmp";
const IMAGE_TGA_EXTENSION: &str = "tga";
const IMAGE_DDS_EXTENSION: &str = "dds";
const IMAGE_TIFF_EXTENSION: &str = "tiff";
const IMAGE_GIF_EXTENSION: &str = "bmp";
const IMAGE_ICO_EXTENSION: &str = "ico";

pub struct ImageCompiler {
    global_messenger: MessengerRw,
}

impl ImageCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }
}

impl ExtensionHandler for ImageCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let extension = ext.to_str().unwrap().to_string();
            if (extension.as_str() == IMAGE_PNG_EXTENSION
                || extension.as_str() == IMAGE_JPG_EXTENSION
                || extension.as_str() == IMAGE_JPEG_EXTENSION
                || extension.as_str() == IMAGE_BMP_EXTENSION
                || extension.as_str() == IMAGE_TGA_EXTENSION
                || extension.as_str() == IMAGE_TIFF_EXTENSION
                || extension.as_str() == IMAGE_GIF_EXTENSION
                || extension.as_str() == IMAGE_ICO_EXTENSION
                || extension.as_str() == IMAGE_DDS_EXTENSION)
                && copy_into_data_folder(&self.global_messenger, path)
            {
                debug_log(format!("Serializing {:?}", path).as_str());
            }
        }
    }
}
