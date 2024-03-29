use inox_nodes::LogicNodeRegistry;
use inox_resources::SharedDataRc;
use inox_resources::Singleton;

pub mod nodes;
pub use nodes::*;

pub fn register_nodes(shared_data: &SharedDataRc) {
    let registry = LogicNodeRegistry::get(shared_data);
    registry.register_node::<RotateNode>();
}

pub fn unregister_nodes(shared_data: &SharedDataRc) {
    let registry = LogicNodeRegistry::get(shared_data);
    registry.unregister_node::<RotateNode>();
}
