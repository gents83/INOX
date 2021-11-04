# NRG :zap: New Rust Game engine - with Blender as Editor

[<img alt="github repository" src="https://img.shields.io/badge/github-gents83/NRG-8da0cb?logo=github" height="20">](https://github.com/gents83/NRG)
[<img alt="github pages" src="https://img.shields.io/badge/Docs-github-brightgreen" height="20">](https://gents83.github.io/NRG/)
[<img alt="github workflow sattus" src="https://img.shields.io/github/workflow/status/gents83/NRG/Deploy%20on%20Github%20Pages?style=plastic" height="20">](https://github.com/gents83/NRG/actions)
[<img alt="github sponsor" src="https://img.shields.io/github/sponsors/gents83?style=plastic" height="20">](https://github.com/sponsors/gents83)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

## NRG

It's a Game Engine written in _Rust_ :crab: with some bindings for external libs and with _Blender_ as editor.
NRG is a game engine written in _Rust_ and developed by [GENTS](https://twitter.com/gents83). 
The main idea behind NRG is to use [Blender](https://www.blender.org/) as external editor, even being able to create visual logic scripting nodes in it, and then have a button to launch the _Rust_ engine, that should be scalable to create games of any scale and for users with every kind of experience.
_Rust_ will give to NRG high performance, reliability and safe code, while _Blender_ will bring easy customization and powerful editor capabilities.

Summary:

* [Vision](#vision)
* [Documentation](#documentation)
* [Notes](#notes)
* [Hotkeys](#hotkeys)
* [Screenshots](#screenshots)


## Vision

The engine is developed with following pillars:
- [x] The game engine should be written in _Rust_
- [x] Multi-platform support (Windows-only implemented right now)
- [x] Multi-GFX api support (Vulkan-only implemented right now)
- [x] Multi-threading support with different Phases, Systems and Jobs
- [x] Easy to use profiling tools of CPU through custom NRG Profiler using _Chrome Trace Event_ format and usable through chrome://tracing/
- [x] Multi-threading Rendering support with IndirectDraw, Render-to-Texture, Multiple passes, Bindless descriptors, etc
- [ ] Should support high-end performance rendering features like PBR, Raytracing, etc
- [x] Easy to use profiling of GPU through [RenderDoc](https://renderdoc.org/) by [Baldurk Karlsson](https://twitter.com/baldurk)
- [x] _Blender_ should be used as external 3D scene editor in order to press a button and launch the external NRG window 
- [x] _Blender_ files are converted through custom NRG Blender add-on written in Python in Khronos [GLTF](https://www.khronos.org/gltf/) files 
- [x] File binarization in background as a continuos thread to convert from raw data to binarized one used by the engine 
- [x] Resources should be loaded at runtime while game is running as a background task
- [x] Hot reload of code to be able to change Rust code with the engine still running 
- [x] Hot reload of data to enable content creators to reload their data changed on the fly  
- [x] In-game GUI library for debugging purposes or in-game tooling through [egui](https://github.com/emilk/egui) by [emilk](https://twitter.com/ernerfeldt)
- [ ] Documentations should be written trough [mdBook](https://rust-lang.github.io/mdBook/)
- [ ] Continous integration and build support should be granted by Github Actions 

## Documentation 

You can find documentation [here](https://gents83.github.io/NRG/)

## Notes

Not ready yet for production.
NRG is in active development, it still lacks many features code architecture or interfaces could still change. 
New releases could still have breaking changes.

## Hotkeys

Useful hotkeys to know:
- in **_Blender_**:
  - F5: Launch Game Engine with current scene or you can use the panel inside Render properties
- in **_NRG_**:
  - F1: Open\Close Info window
  - F2: Open\Close Hierarchy window
  - F9: Start\Stop Profiler and generate profile file on stop


## Screenshots

Following screenshots are just related about NRG engine capabilities:

![Editor test](https://user-images.githubusercontent.com/62186646/130697761-056e6de4-fccb-42fc-8271-ccfa9ab0544f.gif)

![In-game_ui_tests](https://user-images.githubusercontent.com/62186646/127272503-6ff30eba-ea2a-46a0-bdc7-9be6cc32aee1.gif)

![Hot-code-reload-test](https://user-images.githubusercontent.com/62186646/130698279-9daa7b9a-1f3c-4556-be0c-37f8a1c4431e.gif)

![Profiler example](https://user-images.githubusercontent.com/62186646/120451742-f9968e80-c391-11eb-962e-13d132e09847.jpg)
