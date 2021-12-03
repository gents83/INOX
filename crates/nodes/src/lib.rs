pub mod link;
pub mod logic_nodes;
pub mod macros;
pub mod node;
pub mod node_registry;
pub mod node_tree;
pub mod pin;

pub use link::*;
pub use logic_nodes::*;
pub use macros::*;
pub use node::*;
pub use node_registry::*;
pub use node_tree::*;
pub use pin::*;

pub use sabi_serialize::typetag;

implement_pin!(f32);
implement_pin!(f64);
implement_pin!(u8);
implement_pin!(i8);
implement_pin!(u16);
implement_pin!(i16);
implement_pin!(u32);
implement_pin!(i32);
implement_pin!(bool);
implement_pin!(String);
