extern crate nrg_app;
extern crate nrg_platform;

use std::{env::{self, consts::*}, path::Path};
use nrg_app::plugin_manager::IN_USE_PREFIX;
use nrg_platform::utils::*;

fn main() {
    let out_dir = env::current_dir().unwrap();
    let build_path = Path::new(&out_dir).join("..\\target\\debug").canonicalize().unwrap(); 

    let in_use_build_path = build_path.join(IN_USE_PREFIX);    
    remove_files_containing_with_ext(in_use_build_path.clone(), IN_USE_PREFIX, DLL_SUFFIX);
}