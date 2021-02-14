# NRG

New Rust GENTS Game Engine


It's a prototyping Game Engine written entirely in Rust Language with some bindings for external libs.


[Philosophy](#philosopy)

NRG Engine is based on a plugin architecture.



[Notes](#notes)

Not ready yet for production.



[Features](#features)

- [x] Multi-platform support (Windows-only implemented right now)
- [x] Multi-GFX api support (Vulkan-only implemented right now)
- [x] Hot reload of code
- [ ] Hot reload of data



[Dependencies](#dependencies)

External crate dependencies - all with MIT license:

- https://github.com/rust-lang/rust-bindgen (FFI bindings from C\C++ - used for vulkan_bindings)
- https://github.com/netvl/xml-rs (xml parser - used for Vulkan xml specification)
- https://github.com/image-rs/image (image processing library)
- https://github.com/marcianx/downcast-rs (trait casting)


