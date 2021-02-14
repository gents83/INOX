extern crate nrg_core;
extern crate nrg_platform;

use nrg_core::plugins::plugin_manager::IN_USE_PREFIX;
use nrg_platform::utils::*;
use std::{
    env::{self, consts::*},
    path::Path,
};

fn main() {
    let out_dir = env::current_dir().unwrap();
    let build_path = Path::new(&out_dir)
        .join("..\\target\\debug")
        .canonicalize()
        .unwrap();

    let in_use_build_path = build_path.join(IN_USE_PREFIX);
    remove_files_containing_with_ext(in_use_build_path, IN_USE_PREFIX, DLL_SUFFIX);
}
