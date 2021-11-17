## External Dependencies



In this page you'll find a list of external dependencies that are required from the _**SABI**_ engine
and that are referenced as git submodules inside the `/extern/` folder. 

- [**cgmath**](https://github.com/rustgd/cgmath): \
  A linear algebra and mathematics library for computer graphics \
  Used in _**SABI**_ as math library for vectors, matrices, quaternions, etc.
- [**downcast-rs**](https://github.com/marcianx/downcast-rs): \
  Adds downcasting support to trait objects using only safe Rust \
  Used in _**SABI**_ for trait downcasting.
- [**uuid**](https://github.com/uuid-rs/uuid): \
  Generate and parse UUIDs. \
  Used in _**SABI**_ for all resource IDs and for every UUID generated.
- [**serde**](https://github.com/serde-rs/serde): \
  Serialization framework for Rust \
  Used in _**SABI**_ for serialization and deserialization purposes.
- [**serde-json**](https://github.com/serde-rs/json): \
  Strongly typed JSON library for Rust \
  Used in _**SABI**_ for serialization and deserialization purposes of data.
- [**typetag**](https://github.com/dtolnay/typetag): \
  Serde serializable and deserializable trait objects \
  Used in _**SABI**_ for serialization and deserialization purposes.
- [**image-rs**](https://github.com/image-rs/image): \
  Encoding and decoding images in Rust \
  Used in _**SABI**_ for images loading and saving.
- [**ttf-parser**](https://github.com/RazrFalcon/ttf-parser): \
  A high-level, safe, zero-allocation TrueType font parser \
  Used in _**SABI**_ for TrueType fonts loading and parsing.
- [**xml-rs**](https://github.com/netvl/xml-rs): \
  An XML library in Rust \
  Used in _**SABI**_ for Vulkan docs parsing.
- [**gltf-rs**](https://github.com/gltf-rs/gltf): \
  A crate for loading glTF 2.0 \
  Used in _**SABI**_ for GLTF file loading and parsing.
- [**egui**](https://github.com/emilk/egui): \
  An easy-to-use immediate mode GUI in pure Rust \
  Used in _**SABI**_ for in-game tooling and debug UI.
- [**cpython**](https://github.com/dgrunwald/rust-cpython): \
  Rust <-> Python bindings \
  Used in _**SABI**_ for Blender add-on interaction.
- [**rand**](https://github.com/rust-random/rand): \
  A Rust library for random number generation. \
  Used in _**SABI**_ for random number generation.
- [**rust-bindgen**](https://github.com/rust-lang/rust-bindgen): \
  Automatically generates Rust FFI bindings to C (and some C++) libraries. \
  Used in _**SABI**_ to generate Rust bindings of Vulkan headers.