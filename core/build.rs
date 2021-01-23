
use std::{env, path::Path};

fn main() {
    // Write the library to the $OUT_DIR/lib_nrg_core file.  

    let out_dir = env::var("OUT_DIR").unwrap();
    let src_path = Path::new(&out_dir).join("nrg_core.dll");
    let dest_path = Path::new(&out_dir).join("lib_nrg_core");
    
    println!("SRC: {}", src_path.to_str().unwrap());
    println!("DST: {}", dest_path.to_str().unwrap());
    let res = ::std::fs::copy(src_path, dest_path);
    if res.is_ok() {
        println!("Copy succeded");
    }
    else {
        println!("Copy failed")
    }
}