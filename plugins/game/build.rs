extern crate nrg_core;
extern crate nrg_platform;

use nrg_core::plugins::plugin_manager::IN_USE_PREFIX;
use nrg_platform::utils::*;
use std::path::Path;

const LIB_NAME: &str = "nrg_game";
const EXTENSION: &str = ".pdb";

fn main() {
    let out_dir = std::env::current_dir().unwrap();
    let mut build_path = Path::new(&out_dir).join("..\\..\\target\\debug");
    if build_path.exists() {
        build_path = build_path.canonicalize().unwrap();
    }
    let mut deps_path = Path::new(&out_dir).join("..\\..\\target\\debug\\deps");
    if deps_path.exists() {
        deps_path = deps_path.canonicalize().unwrap();
    }

    let in_use_build_path = build_path.join(IN_USE_PREFIX);

    remove_files_containing_with_ext(deps_path.clone(), LIB_NAME, EXTENSION);
    remove_files_containing_with_ext(build_path.clone(), LIB_NAME, EXTENSION);

    copy_with_random_name(deps_path, in_use_build_path.clone(), LIB_NAME, EXTENSION);
    copy_with_random_name(build_path, in_use_build_path, LIB_NAME, EXTENSION);
}
