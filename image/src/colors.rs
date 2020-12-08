

#[derive(Copy, PartialEq, Eq, Debug, Clone, Hash)]
pub enum ColorType {
    /// Pixel is 8-bit luminance
    L8,
    /// Pixel is 8-bit luminance with an alpha channel
    La8,
    /// Pixel contains 8-bit R, G and B channels
    Rgb8,
    /// Pixel is 8-bit RGB with an alpha channel
    Rgba8,
    /// Pixel is 16-bit luminance
    L16,
    /// Pixel is 16-bit luminance with an alpha channel
    La16,
    /// Pixel is 16-bit RGB
    Rgb16,
    /// Pixel is 16-bit RGBA
    Rgba16,
    /// Pixel contains 8-bit B, G and R channels
    Bgr8,
    /// Pixel is 8-bit BGR with an alpha channel
    Bgra8,
}

impl ColorType {
    pub fn bytes_per_pixel(self) -> u8 {
        match self {
            ColorType::L8 => 1,
            ColorType::L16 | ColorType::La8 => 2,
            ColorType::Rgb8 | ColorType::Bgr8 => 3,
            ColorType::Rgba8 | ColorType::Bgra8 | ColorType::La16 => 4,
            ColorType::Rgb16 => 6,
            ColorType::Rgba16 => 8,
        }
    }

    pub fn has_alpha_channel(self) -> bool {
        use ColorType::*;
        match self {
            L8 | L16 | Rgb8 | Bgr8 | Rgb16 => false,
            La8 | Rgba8 | Bgra8 | La16 | Rgba16 => true,
        }
    }

    pub fn has_color(self) -> bool {
        !self.is_grayscale()
    }

    pub fn is_grayscale(self) -> bool {
        use ColorType::*;
        match self {
            L8 | L16 | La8 | La16 => true,
            Rgb8 | Bgr8 | Rgb16 | Rgba8 | Bgra8 | Rgba16 => false,
        }
    }

    pub fn bits_per_pixel(self) -> u16 {
        <u16 as From<u8>>::from(self.bytes_per_pixel()) * 8
    }

    pub fn channel_count(self) -> u8 {
        let e: ExtendedColorType = self.into();
        e.channel_count()
    }
}


#[derive(Copy, PartialEq, Eq, Debug, Clone, Hash)]
pub enum ExtendedColorType {
    /// Pixel is 1-bit luminance
    L1,
    /// Pixel is 1-bit luminance with an alpha channel
    La1,
    /// Pixel contains 1-bit R, G and B channels
    Rgb1,
    /// Pixel is 1-bit RGB with an alpha channel
    Rgba1,
    /// Pixel is 2-bit luminance
    L2,
    /// Pixel is 2-bit luminance with an alpha channel
    La2,
    /// Pixel contains 2-bit R, G and B channels
    Rgb2,
    /// Pixel is 2-bit RGB with an alpha channel
    Rgba2,
    /// Pixel is 4-bit luminance
    L4,
    /// Pixel is 4-bit luminance with an alpha channel
    La4,
    /// Pixel contains 4-bit R, G and B channels
    Rgb4,
    /// Pixel is 4-bit RGB with an alpha channel
    Rgba4,
    /// Pixel is 8-bit luminance
    L8,
    /// Pixel is 8-bit luminance with an alpha channel
    La8,
    /// Pixel contains 8-bit R, G and B channels
    Rgb8,
    /// Pixel is 8-bit RGB with an alpha channel
    Rgba8,
    /// Pixel is 16-bit luminance
    L16,
    /// Pixel is 16-bit luminance with an alpha channel
    La16,
    /// Pixel contains 16-bit R, G and B channels
    Rgb16,
    /// Pixel is 16-bit RGB with an alpha channel
    Rgba16,
    /// Pixel contains 8-bit B, G and R channels
    Bgr8,
    /// Pixel is 8-bit BGR with an alpha channel
    Bgra8,
    Unknown(u8),
}

impl ExtendedColorType {
    
    pub fn channel_count(self) -> u8 {
        match self {
            ExtendedColorType::L1 |
            ExtendedColorType::L2 |
            ExtendedColorType::L4 |
            ExtendedColorType::L8 |
            ExtendedColorType::L16 |
            ExtendedColorType::Unknown(_) => 1,
            ExtendedColorType::La1 |
            ExtendedColorType::La2 |
            ExtendedColorType::La4 |
            ExtendedColorType::La8 |
            ExtendedColorType::La16 => 2,
            ExtendedColorType::Rgb1 |
            ExtendedColorType::Rgb2 |
            ExtendedColorType::Rgb4 |
            ExtendedColorType::Rgb8 |
            ExtendedColorType::Rgb16 |
            ExtendedColorType::Bgr8 => 3,
            ExtendedColorType::Rgba1 |
            ExtendedColorType::Rgba2 |
            ExtendedColorType::Rgba4 |
            ExtendedColorType::Rgba8 |
            ExtendedColorType::Rgba16 |
            ExtendedColorType::Bgra8 => 4,
        }
    }
}

impl From<ColorType> for ExtendedColorType {
    fn from(c: ColorType) -> Self {
        match c {
            ColorType::L8 => ExtendedColorType::L8,
            ColorType::La8 => ExtendedColorType::La8,
            ColorType::Rgb8 => ExtendedColorType::Rgb8,
            ColorType::Rgba8 => ExtendedColorType::Rgba8,
            ColorType::L16 => ExtendedColorType::L16,
            ColorType::La16 => ExtendedColorType::La16,
            ColorType::Rgb16 => ExtendedColorType::Rgb16,
            ColorType::Rgba16 => ExtendedColorType::Rgba16,
            ColorType::Bgr8 => ExtendedColorType::Bgr8,
            ColorType::Bgra8 => ExtendedColorType::Bgra8,
        }
    }
}
