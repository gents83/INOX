#![warn(clippy::all)]

#[macro_use]
pub extern crate serde;

#[macro_use]
pub extern crate serde_derive;

pub use serde::*;
pub use serde_derive::*;

pub use crate::serialize::*;

pub mod serialize;
