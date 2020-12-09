use std::io::Read;
use super::colors::*;


/// The trait that all decoders implement
pub trait ImageDecoder<'a>: Sized {
    type Reader: Read + 'a;

    fn dimensions(&self) -> (u32, u32);
    fn color_type(&self) -> ColorType;

    fn original_color_type(&self) -> ExtendedColorType {
        self.color_type().into()
    }

    fn total_bytes(&self) -> u64 {
        let dimensions = self.dimensions();
        u64::from(dimensions.0) * u64::from(dimensions.1) * u64::from(self.color_type().bytes_per_pixel())
    }

    fn scanline_bytes(&self) -> u64 {
        self.total_bytes()
    }

    fn read_image(&mut self, buf: &mut [u8]);
}