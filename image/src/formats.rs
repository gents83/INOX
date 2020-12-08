use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ImageFormat {
    PNG,
    JPG,
    BMP,
}

impl ImageFormat {
    #[inline]
    pub fn get_extension(self) -> &'static str {
        ImageFormat::get_extension_from_format(self)
    }
    #[inline]
    pub const fn get_possible_extensions_from_format(format: ImageFormat) -> &'static [&'static str] {
        match format {
            ImageFormat::PNG => &["png"],
            ImageFormat::JPG => &["jpg", "jpeg"],
            ImageFormat::BMP => &["bmp"],
        }
    }
    #[inline]
    pub const fn get_extension_from_format(format: ImageFormat) -> &'static str {
        match format {
            ImageFormat::PNG => "png",
            ImageFormat::JPG => "jpg",
            ImageFormat::BMP => "bmp",
        }
    }
    #[inline]
    pub fn get_format_from_extension(extension: &str) -> Option<ImageFormat> {
        let res = Some(match extension {
                                        "jpg" | "jpeg" => ImageFormat::JPG,
                                        "png" => ImageFormat::PNG,
                                        "bmp" => ImageFormat::BMP,
                                        _ => return None,
                                    });
        res
    }
    #[inline]
    pub fn from_extension(filepath: &Path) -> Option<ImageFormat> {
        let ext = filepath.extension().unwrap().to_str().unwrap().to_ascii_lowercase();
        ImageFormat::get_format_from_extension(ext.as_str())
    }
}
