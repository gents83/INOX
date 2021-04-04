use nrg_serialize::{typetag, Deserialize, Serialize};

use crate::{WidgetGraphics, WidgetNode, WidgetState};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetData {
    pub node: WidgetNode,
    pub graphics: WidgetGraphics,
    pub state: WidgetState,
    initialized: bool,
}

impl Default for WidgetData {
    fn default() -> Self {
        Self {
            node: WidgetNode::default(),
            graphics: WidgetGraphics::default(),
            state: WidgetState::default(),
            initialized: false,
        }
    }
}

#[typetag::serde(tag = "data")]
pub trait WidgetDataGetter {
    fn get_data(&self) -> &WidgetData;
    fn get_data_mut(&mut self) -> &mut WidgetData;
    fn is_initialized(&self) -> bool {
        self.get_data().initialized
    }
    fn mark_as_initialized(&mut self) {
        self.get_data_mut().initialized = true;
    }
}
