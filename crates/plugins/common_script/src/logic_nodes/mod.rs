use sabi_nodes::LogicNodeRegistry;
use sabi_resources::SharedDataRc;

pub mod nodes;
pub use nodes::*;

pub fn register_nodes(shared_data: &SharedDataRc) {
    if let Some(registry) = shared_data.get_singleton_mut::<LogicNodeRegistry>() {
        registry.register_node::<RotateNode>();
    }
}
