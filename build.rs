pub mod src {
    pub mod core;
}

use src::core::*;





const FOLDER_SEPARATOR: &str = "/";

#[cfg(debug_assertions)]
const TARGET_FOLDER: &str = "/target/debug/";
#[cfg(not(debug_assertions))]
const TARGET_FOLDER: &str = "/target/release/";


fn use_project_library(project_folder: String, library_name: String) {
    let out_dir = ::std::env::var("CARGO_MANIFEST_DIR").unwrap_or(String::from(""));
    println!("CARGO_MANIFEST_DIR: {}", out_dir);

    let mut src_path = out_dir.clone();
    src_path.push_str(FOLDER_SEPARATOR);
    src_path.push_str(project_folder.as_str());
    src_path.push_str(TARGET_FOLDER);
    src_path.push_str(library_name.as_str());

    let mut dest_path = out_dir.clone();
    dest_path.push_str(TARGET_FOLDER); 
    dest_path.push_str(library_name.as_str());
    
    let res = ::std::fs::copy(src_path, dest_path);
    if res.is_ok() {
        println!("Copy succeded");
    }
    else {
        println!("Copy failed")
    }
}

fn main() {
    use_project_library( get_core_project_folder_name(), get_core_lib_path());
}