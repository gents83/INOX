#![warn(clippy::all)]
pub extern crate inox_serializable;

pub use serde::*;

pub use self::serialize::*;

pub mod serialize;
