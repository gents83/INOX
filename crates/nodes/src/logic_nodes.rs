use std::any::type_name;

use sabi_serialize::{generate_uid_from_string, Deserialize, Serialize, Uid};

use crate::{implement_node, implement_output_pin};

pub type NodeId = Uid;

implement_output_pin!(ScriptExecution);

pub trait Node: Send + Sync + 'static {
    fn type_id() -> NodeId
    where
        Self: Sized,
    {
        generate_uid_from_string(Self::node_type())
    }
    fn node_type() -> &'static str
    where
        Self: Sized,
    {
        type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    }
    fn base_type() -> &'static str
    where
        Self: Sized;
    fn description() -> &'static str
    where
        Self: Sized;
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct RustExampleNode {
    in_int_value: u32,
    in_float_value: f32,
    in_string_value: String,
    in_bool_value: bool,
    out_execute: ScriptExecution,
    out_int_value: u32,
    out_float_value: f32,
    out_string_value: String,
    out_bool_value: bool,
}
implement_node!(
    RustExampleNode,
    "LogicNodeBase",
    "Example node created from Rust"
);

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct ScriptInitNode {
    out_execute: ScriptExecution,
}
implement_node!(
    ScriptInitNode,
    "LogicNodeBase",
    "Node will be called when starting the script"
);
