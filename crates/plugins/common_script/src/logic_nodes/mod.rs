use sabi_nodes::LogicNodeRegistry;
use sabi_resources::SharedDataRc;
use sabi_resources::Singleton;

pub mod nodes;
pub use nodes::*;

pub fn register_nodes(shared_data: &SharedDataRc) {
    let registry = LogicNodeRegistry::get(shared_data);
    registry.register_node::<RotateNode>(shared_data);
}
