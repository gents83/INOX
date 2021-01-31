extern crate nrg_app;
extern crate nrg_platform;

use std::path::Path;
use nrg_app::plugin_manager::IN_USE_PREFIX;
use nrg_platform::utils::*;

const LIB_NAME:&str = "nrg_game";
const EXTENSION:&str = ".pdb";

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

    copy_with_random_name(deps_path.clone(), in_use_build_path.clone(), LIB_NAME, EXTENSION);
    copy_with_random_name(build_path.clone(), in_use_build_path.clone(), LIB_NAME, EXTENSION);
}