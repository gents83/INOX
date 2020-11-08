//#![warn(missing_doc)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(ellipsis_inclusive_range_patterns)]
#![forbid(non_camel_case_types)]
#![forbid(unsafe_code)]

pub use reader::EventReader;
pub use reader::ParserConfig;
pub use writer::EventWriter;
pub use writer::EmitterConfig;

pub mod macros;
pub mod name;
pub mod attribute;
pub mod common;
pub mod escape;
pub mod namespace;
pub mod reader;
pub mod writer;
mod util;
