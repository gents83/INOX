use std::collections::HashSet;

use super::stage::*;

pub struct Scheduler {
    stages: HashSet<&'static str, Stage>, 
}