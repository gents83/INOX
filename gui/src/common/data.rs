use nrg_resources::SharedDataRw;
use nrg_serialize::{typetag, Deserialize, Serialize};

use crate::{WidgetGraphics, WidgetNode, WidgetState};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetData {
    pub node: WidgetNode,
    pub graphics: WidgetGraphics,
    pub state: WidgetState,
    initialized: bool,
    #[serde(skip)]
    shared_data: SharedDataRw,
}

impl WidgetData {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        Self {
            node: WidgetNode::default(),
            graphics: WidgetGraphics::new(shared_data),
            state: WidgetState::default(),
            initialized: false,
            shared_data: shared_data.clone(),
        }
    }
    pub fn get_shared_data(&self) -> &SharedDataRw {
        &self.shared_data
    }
}

#[typetag::serde(tag = "data")]
pub trait WidgetDataGetter {
    fn get_shared_data(&self) -> &SharedDataRw;
    fn get_data(&self) -> &WidgetData;
    fn get_data_mut(&mut self) -> &mut WidgetData;
    fn is_initialized(&self) -> bool {
        self.get_data().initialized
    }
    fn mark_as_initialized(&mut self) {
        self.get_data_mut().initialized = true;
    }
}
