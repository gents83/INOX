#![warn(clippy::all)]
pub extern crate inox_serializable;

pub use serde::*;
pub use serde_derive::*;

pub use self::serialize::*;
pub use self::uuid::*;

pub mod serialize;
pub mod uuid;
