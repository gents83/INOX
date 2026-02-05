fn main() {
    let mut build = cxx_build::bridge("src/cxx_ffi.rs");
    build
        .file("ffi/src/tinybvh.cpp")
        .std("c++20");

    if std::env::var("TARGET").is_ok_and(|t| (t.contains("x86_64") || t.contains("x86")) && !t.contains("msvc")) {
        build.flag("-march=native"); // SIMD for x86/x64 host optimization
    }

    build.compile("tinybvh");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=tinybvh/tiny_bvh.h");
    println!("cargo:rerun-if-changed=src/tiny_bvh.h");
    println!("cargo:rerun-if-changed=src/tiny_bvh.cpp");
}
