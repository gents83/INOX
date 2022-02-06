#![warn(clippy::all)]
pub extern crate inox_serializable;

pub use serde::*;
pub use serde_derive::*;

pub use self::serialize::*;

pub mod serialize;
