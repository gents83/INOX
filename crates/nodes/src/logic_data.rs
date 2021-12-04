use sabi_serialize::{Deserialize, Serialize, SerializeFile};

use crate::NodeTree;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct LogicData {
    #[serde(flatten)]
    tree: NodeTree,
}

impl SerializeFile for LogicData {
    fn extension() -> &'static str {
        "logic"
    }
}

impl From<NodeTree> for LogicData {
    fn from(tree: NodeTree) -> Self {
        Self { tree }
    }
}

impl LogicData {
    pub fn tree(&self) -> &NodeTree {
        &self.tree
    }
}
