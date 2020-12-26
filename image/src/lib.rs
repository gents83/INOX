
pub use crate::colors::*;
pub use crate::formats::*;
pub use crate::utils::*;
pub use crate::bmp::reader::Reader as BmpReader;
pub use crate::jpg::reader::Reader as JpgReader;
pub use crate::png::reader::Reader as PngReader;

mod formats;
mod macros;

pub mod colors;
pub mod decoder;
pub mod image;
pub mod reader;
pub mod utils;

mod bmp {
    pub mod reader;
    pub mod decoder;
}
mod jpg {
    pub mod reader;
}
mod png {
    pub mod reader;
    pub mod decoder;
}