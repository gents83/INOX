
fn link_library(name: &str) 
{
    println!("cargo:rustc-link-lib=dylib={}", name);
}

fn main() 
{
    // Deterimine build platform
    let target_os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_windows_platform = target_os == "windows";

    if is_windows_platform 
    {
        link_library("user32");
        link_library("kernel32");
    } 
    else 
    {
        panic!("Platform not yet supported - Check build.rs to setup this platform to build from source");
    }
}

