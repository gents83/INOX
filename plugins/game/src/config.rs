use std::path::PathBuf;

use nrg_core::*;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub fonts: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fonts: Vec::new(),
        }
    }
}

impl Data for Config {}
impl ConfigBase for Config {}
