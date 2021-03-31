#![warn(clippy::all)]

pub extern crate typetag;

pub use serde::*;
pub use serde_derive::*;

pub use crate::serialize::*;
pub use crate::uuid::*;

pub mod serialize;
pub mod uuid;
