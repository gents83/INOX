## Features

Here you can find a list of features that are currently supported by the library.

- [x] Multi platform architecture \
      &emsp;&emsp;**Windows** only implemented right now \
      &emsp;&emsp;_rawwindowhandle_ integrated and working on WASM intergration
- [x] Multi graphics api support \
      &emsp;&emsp;_wgpu_ integration done
- [x] Multi threading architecture with different Phases > Systems > Jobs \
      &emsp;&emsp;See more info in _sabicore_ crate section related to scheduling
- [x] CPU profiling using _Chrome Trace Event_ format and usable through chrome://tracing/ \
      &emsp;&emsp;See more info in _sabiprofiler_ crate section
- [x] GPU profiling using [RenderDoc](https://renderdoc.org/) by [Baldurk Karlsson](https://twitter.com/baldurk)
- [x] _Blender_ addon written in _Python_ and _Rust_ built and copied into right folder to be used right away \
      &emsp;&emsp;See more info in _sabiblender_ crate section)
- [x] Launch and execute _SABI_ directly from _Blender_ \
      &emsp;&emsp;Exporting scene as Khronos [GLTF](https://www.khronos.org/gltf/), binarizing and loading it into _SABI_
- [x] Data binarization, shader compilation, etc as background task \
      &emsp;&emsp;See more info in _sabibinarizer_ crate section
- [x] Hot reload of code while INOX engine is running 
- [x] Hot reload of data reloading on the fly while INOX engine is running  
- [x] In-game GUI integration using [egui](https://github.com/emilk/egui) by [emilk](https://twitter.com/ernerfeldt)
- [x] Documentations using [mdBook](https://rust-lang.github.io/mdBook/)
- [x] Continous integration and build support using Github Actions 
- [x] _**INOX**_ <-> _**Blender**_ communication through TCP connection
- [x] Create custom Logic Nodes editor in _**Blender**_
- [x] Possibility to define and use custom nodes in external dlls
- [x] Integrated wasm as target
- [x] Added meshoptimizer library by [Arseny Kapoulkine](https://twitter.com/zeuxcg)
- [x] Possibility to run Phases in parallel when not dependent
- [x] Plugin indipendent and ability to enable\disable them at runtime
- [x] Dynamic vertex layout and storage buffers per-shader 

TODO:
- [ ] Integrate Android as target
- [ ] Add multithreading workers for WASM target on web
- [ ] Integrate Meshlet generation and gpu culling
- [ ] Integrate PBR rendering
- [ ] Raytracing on GPU
- [ ] Quick example of game 
- [ ] Integrate KTX universal texture compression
