pub mod link;
pub mod logic_context;
pub mod logic_data;
pub mod logic_nodes;
pub mod macros;
pub mod node;
pub mod node_registry;
pub mod node_tree;
pub mod pin;

pub use link::*;
pub use logic_context::*;
pub use logic_data::*;
pub use logic_nodes::*;
pub use macros::*;
pub use node::*;
pub use node_registry::*;
pub use node_tree::*;
pub use pin::*;

use sabi_resources::SharedDataRc;
use sabi_resources::Singleton;
use sabi_serialize::sabi_serializable;

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

pub fn register_nodes(shared_data: &SharedDataRc) {
    shared_data.register_singleton(LogicNodeRegistry::default());

    let registry = LogicNodeRegistry::get(shared_data);
    //Registering basic types
    registry.register_pin_type::<f32>();
    registry.register_pin_type::<f64>();
    registry.register_pin_type::<u8>();
    registry.register_pin_type::<i8>();
    registry.register_pin_type::<u16>();
    registry.register_pin_type::<i16>();
    registry.register_pin_type::<u32>();
    registry.register_pin_type::<i32>();
    registry.register_pin_type::<bool>();
    registry.register_pin_type::<String>();
    registry.register_pin_type::<LogicExecution>();

    //Registering default nodes
    registry.register_node::<RustExampleNode>();
    registry.register_node::<ScriptInitNode>();
}

pub fn unregister_nodes(shared_data: &SharedDataRc) {
    let registry = LogicNodeRegistry::get(shared_data);
    //Unregistering default nodes
    registry.unregister_node::<ScriptInitNode>();
    registry.unregister_node::<RustExampleNode>();

    //Unregistering basic types
    registry.unregister_pin_type::<LogicExecution>();
    registry.unregister_pin_type::<String>();
    registry.unregister_pin_type::<bool>();
    registry.unregister_pin_type::<i32>();
    registry.unregister_pin_type::<u32>();
    registry.unregister_pin_type::<i16>();
    registry.unregister_pin_type::<u16>();
    registry.unregister_pin_type::<i8>();
    registry.unregister_pin_type::<u8>();
    registry.unregister_pin_type::<f64>();
    registry.unregister_pin_type::<f32>();

    shared_data.unregister_singleton::<LogicNodeRegistry>();
}
