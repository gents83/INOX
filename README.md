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
- [x] Multi-thread support with different Phases and Job system
- [x] CPU Profiler using Chrome Trace Event format and usable through chrome://tracing/
- [x] GUI library
- [x] Indirect draw support
- [x] Texture array and atlas support
- [x] Hot reload of code
- [x] Hot reload of data



[External crates dependencies](#dependencies)

Focus is to have all of them with MIT license.

- FFI bindings from C\C++ - used for vulkan_bindings: https://github.com/rust-lang/rust-bindgen
- XML parser - used for Vulkan xml specification: https://github.com/netvl/xml-rs 
- Image processing library: https://github.com/image-rs/image
- Trait casting: https://github.com/marcianx/downcast-rs
- Serialization - serde, serde_derive & serde_json: https://github.com/serde-rs/serde
- CG Math library: https://github.com/rustgd/cgmath


[Hotkeys](#hotkeys)

Useful hotkeys to know:
- F5: Launch Game from Editor app
- F9: Start\Stop Profiler and generate profile file on stop



[Screenshots](#screenshot)


![Hot code reload test](https://pbs.twimg.com/media/ErY_fFnW4AAIN5Q?format=jpg)

![GUI test](https://user-images.githubusercontent.com/62186646/116011134-c428b380-a623-11eb-8979-34d23f0532fd.jpg)

![Profiler example](![Profiler](https://user-images.githubusercontent.com/62186646/120451742-f9968e80-c391-11eb-962e-13d132e09847.jpg))
