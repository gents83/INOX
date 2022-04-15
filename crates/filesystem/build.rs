use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
fn move_all_files_with_extension(src_path: PathBuf, target_path: PathBuf, extension: &str) {
    let files = fs::read_dir(src_path).unwrap();
    files
        .filter_map(Result::ok)
        .filter(|f| match f.path().extension() {
            Some(file) => file == extension,
            _ => false,
        })
        .for_each(|f| {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros();
            let new_name = format!(
                "{}_{}",
                secs,
                f.path().file_name().unwrap().to_str().unwrap()
            );
            let new_path = target_path.join(new_name);
            let _res = fs::rename(f.path(), new_path);
            let _res = std::fs::remove_file(f.path());
        });
}
fn link_library(name: &str) {
    println!("cargo:rustc-link-lib=dylib={}", name);
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir)
        .join("..\\..\\..\\")
        .canonicalize()
        .unwrap();
    let mut deps_path = Path::new(&out_dir).join("deps");
    if deps_path.exists() {
        deps_path = deps_path.canonicalize().unwrap();
    }

    let deps_build_path = out_dir.join("in_use");
    let in_use_build_path = deps_build_path.join("deps");

    move_all_files_with_extension(deps_path, deps_build_path, "pdb");
    move_all_files_with_extension(out_dir, in_use_build_path, "pdb");

    // Deterimine build platform
    let target_os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let is_windows_platform = target_os == "windows";
    let is_android_platform = target_os == "android";
    let is_web_platform = target_arch == "wasm32";

    if is_windows_platform {
        link_library("user32");
        link_library("kernel32");
        link_library("shcore");
    } else if is_android_platform || is_web_platform {
    } else {
        panic!("Platform {} not yet supported - Check build.rs to setup this platform to build from source", target_os);
    }
}
