#![warn(clippy::all)]

pub use erased_serde;
pub use inox_serialize_attribute::*;
pub use macros::*;
pub use registry::*;
pub use serde;

pub mod adjacently;
pub mod externally;
pub mod internally;
pub mod macros;
pub mod registry;

mod content;
mod de;
mod ser;

// Object-safe trait bound inserted by inox_serialize serialization. We want this just
// so the serialization requirement appears on rustdoc's view of your trait.
// Otherwise not public API.
#[doc(hidden)]
pub trait Serialize: erased_serde::Serialize {}

impl<T: ?Sized> Serialize for T where T: erased_serde::Serialize {}

// Object-safe trait bound inserted by inox_serialize deserialization. We want this
// just so the serialization requirement appears on rustdoc's view of your
// trait. Otherwise not public API.
#[doc(hidden)]
pub trait Deserialize {}

impl<T> Deserialize for T {}

// Not public API.
#[doc(hidden)]
pub trait InheritTrait {
    type Object: ?Sized;
}
