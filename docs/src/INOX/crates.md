## INOX Crates


In this page you'll find a list of crates and their description and main concepts of the _**INOX**_ project:


- [**app**](app): \
  It's the entry point crate of every _**INOX**_ application. \
  It's basically a launcher that load different plugins dinamically or statically depending on platforms. \
  It also contains the basic window system and the main loop.

- [**binarizer**](binarizer): \
  This crate is used on pc platforms to binarize raw data into _**INOX**_ data format. 
  It supports simple copy, font texture generation, gltf conversion, textures and shaders.
  
- [**blender**](blender): \
  This crate contains the interoperability between _**Blender**_ and _**INOX**_ engine.
  
- [**commands**](comands): \
  It's a useful crate to handle command line parsing easily.
  
- [**core**](core): \
  This crate is the real core of _**INOX**_ engine. \
  It contains the real application loop, the scheduler, phases and job system and the handling of plugins.
  
- [**filesystem**](filesystem): \
  File handling for different platforms, dinamic library loading and file watchers.
  
- [**graphics**](graphics): \
  All graphics related stuff of _**INOX**_ engine. \
  From renderer to shaders, textures, meshes, materials, fonts and all other resources handling. \
  It lies on top of wgpu
  
- [**log**](log): \
  It's a useful crate to handle logging on different platforms.
  
- [**math**](math): \
  This crate contains math classes and utilities. \
  It wraps cgmath crate. 
  
- [**messenger**](messenger): \
  Message handlers and message communication crate. \
  It's at the core of _**INOX**_ engine and it's used to handle messages between different parts of the engine.
  
- [**nodes**](nodes): \
  It's a useful crate to handle node graphs, input and output pins and connections.
  
- [**platform**](platform): \
  Basic platform code for windowing and input handling.
  
- [**plugins**](plugins): \
  In this folder you can find several plugins that you can use for your application. \
  They can be used alone or combined together with other plugins easily.
  
- [**profiler**](profiler): \
  It's a useful crate to handle profiling. \
  It stores data to be used with chrome:://tracing but it can very easily extended if needed.
  
- [**resources**](resources): \
  It contains resource concept and shared data that is at the core of ECS paradigm of _**INOX**_ engine.
  
- [**scene**](scene): \
  This crate has scene, object and its components (like camera, script, etc).
  
- [**serialize**](serialize): \
  _**INOX**_ engine serialization and deserialization library.
  
- [**time**]()time: \
  It's a useful crate to handle timer and fps computation.
  
- [**ui**](ui): \
  This crate contains ui system, widget handling and wraps egui.
  
- [**uid**](uid): \
  It's a useful crate to handle unique identifiers generation and handling.
