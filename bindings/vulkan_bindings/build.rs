
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
  
    let bindings = bindgen::Builder::default()
      .header("data/Vulkan-Headers/include/vulkan/vulkan.h")
      // We don't care about the functions, we only really need the types + function pointers.
      .ignore_functions()
      // Pass some defines
      .clang_arg("-DVK_NO_PROTOTYPES")
      .generate()
      .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could nt write bindings due to some errors");
}