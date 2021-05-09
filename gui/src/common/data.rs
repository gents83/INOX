use nrg_events::EventsRw;
use nrg_resources::SharedDataRw;
use nrg_serialize::{typetag, Deserialize, Serialize};

use crate::{WidgetGraphics, WidgetNode, WidgetState};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetData {
    node: WidgetNode,
    graphics: WidgetGraphics,
    state: WidgetState,
    initialized: bool,
    #[serde(skip)]
    shared_data: SharedDataRw,
    #[serde(skip)]
    events_rw: EventsRw,
}

impl WidgetData {
    pub fn new(shared_data: &SharedDataRw, events_rw: &EventsRw) -> Self {
        Self {
            node: WidgetNode::default(),
            graphics: WidgetGraphics::new(shared_data),
            state: WidgetState::default(),
            initialized: false,
            shared_data: shared_data.clone(),
            events_rw: events_rw.clone(),
        }
    }
    #[inline]
    pub fn get_shared_data(&self) -> &SharedDataRw {
        &self.shared_data
    }
    #[inline]
    pub fn get_events(&self) -> &EventsRw {
        &self.events_rw
    }
    #[inline]
    pub fn node(&self) -> &WidgetNode {
        &self.node
    }
    #[inline]
    pub fn node_mut(&mut self) -> &mut WidgetNode {
        &mut self.node
    }
    #[inline]
    pub fn graphics(&self) -> &WidgetGraphics {
        &self.graphics
    }
    #[inline]
    pub fn graphics_mut(&mut self) -> &mut WidgetGraphics {
        &mut self.graphics
    }
    #[inline]
    pub fn state(&self) -> &WidgetState {
        &self.state
    }
    #[inline]
    pub fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    #[inline]
    pub fn mark_as_initialized(&mut self) {
        self.initialized = true;
    }
}

#[typetag::serde(tag = "data")]
pub trait WidgetDataGetter {
    fn get_shared_data(&self) -> &SharedDataRw;
    fn get_events(&self) -> &EventsRw;
    fn node(&self) -> &WidgetNode;
    fn node_mut(&mut self) -> &mut WidgetNode;
    fn graphics(&self) -> &WidgetGraphics;
    fn graphics_mut(&mut self) -> &mut WidgetGraphics;
    fn state(&self) -> &WidgetState;
    fn state_mut(&mut self) -> &mut WidgetState;
    fn is_initialized(&self) -> bool;
    fn mark_as_initialized(&mut self);
}
