## Features

Here you can find a list of features that are currently supported by the library.

- [x] Multi platform architecture \
      &emsp;&emsp;**Windows** only implemented right now \
      &emsp;&emsp;_rawwindowhandle_ integration could be an option in the future
- [x] Multi graphics api support \
      &emsp;&emsp;**Vulkan** only implemented right now, \
      &emsp;&emsp;_wgpu_ integration could be an option in the future
- [x] Multi threading architecture with different Phases > Systems > Jobs \
      &emsp;&emsp;See more info in _nrgcore_ crate section related to scheduling
- [x] CPU profiling using _Chrome Trace Event_ format and usable through chrome://tracing/ \
      &emsp;&emsp;See more info in _nrgprofiler_ crate section
- [x] GPU profiling using [RenderDoc](https://renderdoc.org/) by [Baldurk Karlsson](https://twitter.com/baldurk)
- [x] _Blender_ addon written in _Python_ and _Rust_ built and copied into right folder to be used right away \
      &emsp;&emsp;See more info in _nrgblender_ crate section)
- [x] Launch and execute _NRG_ directly from _Blender_ \
      &emsp;&emsp;Exporting scene as Khronos [GLTF](https://www.khronos.org/gltf/), binarizing and loading it into _NRG_
- [x] Data binarization, shader compilation, etc as background task \
      &emsp;&emsp;See more info in _nrgbinarizer_ crate section
- [x] Hot reload of code while NRG engine is running 
- [x] Hot reload of data reloading on the fly while NRG engine is running  
- [x] In-game GUI integration using [egui](https://github.com/emilk/egui) by [emilk](https://twitter.com/ernerfeldt)
- [x] Documentations using [mdBook](https://rust-lang.github.io/mdBook/)
- [x] Continous integration and build support using Github Actions 

TODO:
- [ ] Create custom Logic Nodes editor in _**Blender**_ (in progress)
- [ ] Integrate PBR rendering
- [ ] Raytracing on GPU compute
- [ ] Integrate wgpu as optional feature
- [ ] Dynamic vertex data per-shader
- [ ] Integrate wasm as target
