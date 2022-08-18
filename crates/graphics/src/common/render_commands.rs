use std::collections::HashMap;

use crate::{DrawCommandType, DrawIndexedCommand};

#[derive(Default, Clone)]
pub struct RenderCommandsPerType {
    pub map: HashMap<DrawCommandType, RenderCommands>,
}

#[derive(Default, Clone)]
pub struct RenderCommands {
    pub count: usize,
    pub commands: Vec<DrawIndexedCommand>,
}
