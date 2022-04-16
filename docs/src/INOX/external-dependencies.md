## External Dependencies



In this page you'll find a list of external dependencies that are required from the _**INOX**_ engine
and that are referenced as git submodules inside the `/extern/` folder. 

- [**cgmath**](https://github.com/rustgd/cgmath): \
  A linear algebra and mathematics library for computer graphics \
  Used in _**INOX**_ as math library for vectors, matrices, quaternions, etc.
- [**downcast-rs**](https://github.com/marcianx/downcast-rs): \
  Adds downcasting support to trait objects using only safe Rust \
  Used in _**INOX**_ for trait downcasting.
- [**egui**](https://github.com/emilk/egui): \
  An easy-to-use immediate mode GUI in pure Rust \
  Used in _**INOX**_ for in-game tooling and debug UI.
- [**erased-serde**](https://github.com/dtolnay/erased-serde): \
  Trait serialization and deserialization for Rust \
  Used in _**INOX**_ for trait serialization and deserialization purposes.
- [**gltf-rs**](https://github.com/gltf-rs/gltf): \
  A crate for loading glTF 2.0 \
  Used in _**INOX**_ for GLTF file loading and parsing.
- [**image-rs**](https://github.com/image-rs/image): \
  Encoding and decoding images and textures in Rust \
  Used in _**INOX**_ for images loading and saving.
- [**serde-json**](https://github.com/serde-rs/json): \
  Strongly typed JSON library for Rust \
  Used in _**INOX**_ for serialization and deserialization purposes of data.
- [**meshopt-rs**](https://github.com/gwihlidal/meshopt-rs): \
  Rust ffi and idiomatic wrapper for zeux/meshoptimizer, a mesh optimization library that makes indexed meshes more GPU-friendly. \
  Used in _**INOX**_ for mesh optimization and clustering.
- [**pyo3**](https://github.com/PyO3/pyo3): \
  Rust bindings for the Python interpreter \
  Used in _**INOX**_ for Blender add-on interactions.
- [**rand**](https://github.com/rust-random/rand): \
  A Rust library for random number generation. \
  Used in _**INOX**_ for random number generation.
- [**raw-window-handle**](https://github.com/rust-windowing/raw-window-handle): \
  A common windowing interoperability library for Rust \
  Used in _**INOX**_ for shared window and handle concepts needed by wpgu.
- [**serde**](https://github.com/serde-rs/serde): \
  Serialization framework for Rust \
  Used in _**INOX**_ for serialization and deserialization purposes.
- [**ttf-parser**](https://github.com/RazrFalcon/ttf-parser): \
  A high-level, safe, zero-allocation TrueType font parser \
  Used in _**INOX**_ for TrueType fonts loading and parsing.
- [**uuid**](https://github.com/uuid-rs/uuid): \
  Generate and parse UUIDs. \
  Used in _**INOX**_ for all resource IDs and for every UUID generated.
- [**wasm-bindgen**](https://github.com/rustwasm/wasm-bindgen): \
  Facilitating high-level interactions between Wasm modules and JavaScript \
  Used in _**INOX**_ for wasm and javascript bindings.
- [**wgpu**](https://github.com/gfx-rs/wgpu): \
  Safe and portable GPU abstraction in Rust, implementing WebGPU API \
  Used in _**INOX**_ for cross-platform graphics shared api