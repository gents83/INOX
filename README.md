# **INOX** - Rust Game engine integrated in Blender

[<img alt="github repository" src="https://img.shields.io/badge/github-gents83/INOX-8da0cb?logo=github" height="20">](https://github.com/gents83/INOX)
[<img alt="github pages" src="https://img.shields.io/badge/Docs-github-brightgreen" height="20">](https://gents83.github.io/INOX/)
[<img alt="github workflow sattus" src="https://img.shields.io/github/workflow/status/gents83/INOX/Deploy%20on%20Github%20Pages?style=plastic" height="20">](https://github.com/gents83/INOX/actions)
[<img alt="github sponsor" src="https://img.shields.io/github/sponsors/gents83?style=plastic" height="20">](https://github.com/sponsors/gents83)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)


Summary:
  - [INOX](#sabi)
  - [Why INOX?](#why-inox)
  - [Vision](#vision)
  - [Documentation](#documentation)
  - [Notes](#notes)
  - [Hotkeys](#hotkeys)
  - [Screenshots](#screenshots)


## INOX

It's a Game Engine written in _**Rust**_ with some bindings for external libs and with _**Blender**_ as editor.

INOX is a game engine written in _**Rust**_ and developed by [GENTS](https://twitter.com/gents83). 

The main idea behind INOX is to use [Blender](https://www.blender.org/) as external editor, even being able to create visual logic scripting nodes in it, and then have a button to launch the _**Rust**_ engine, that should be scalable to create games of any scale and for users with every kind of experience.

_**Rust**_ will give to INOX high performance, reliability and safe code, while _**Blender**_ will bring easy customization and powerful editor capabilities.


## Why INOX?

Well... because even if made in _Rust_ it should become an inoxidable game development engine :)


## Vision

The engine is developed with following pillars:
- The game engine should be obviously written in _**Rust**_
- The engine should support multiple platforms (PC, Mobile, Sony Playstation, Microsoft XBox, Nintendo Switch, etc)
- The rendering engine should support different GFX API as well (like Vulkan, DirectX, Metal, etc)
- The engine should be multi-threaded both on CPU and GPU to reach high-end performances
- The engine should support streaming, quick background loading and hot-reload of Code and Data
- _**Blender**_ should be used as external 3D scene, Material, Animation and other content edition 
- INOX should generate a _**Blender**_ addon that could be installed in order to launch and communicate with it
- _**Blender**_ should be used as external editor with new custom INOX editors (like Logic Node Visual Scripting, etc) 
- From _**Blender**_ the user should be able to see the same scene rendered in INOX just pressing a button
- INOX Engine could be used just as a high quality rendering engine
- INOX Engine could be used to run game logic  


## Documentation 

You can find documentation [here](https://gents83.github.io/INOX/)

## Notes

Not ready yet for production.
INOX is in active development, it still lacks many features code architecture or interfaces could still change. 
New releases could still have breaking changes.


## Hotkeys

Useful hotkeys to know:
- in **_Blender_**:
  - F5: Launch Game Engine with current scene or you can use the panel inside Render properties
- in **_INOX_**:
  - F1: Open\Close Info window
  - F2: Open\Close Hierarchy window
  - F9: Start\Stop Profiler and generate profile file on stop


## Screenshots

Following screenshots are just related about INOX engine capabilities:

![NaniteLikeINOX](https://github.com/gents83/INOX/assets/62186646/d0e2af3a-c741-4ce5-98f3-da943fe719b6)

![LODs](https://github.com/gents83/INOX/assets/62186646/e3e70000-be47-47b3-8d3f-da94a74605b4)

![PBR_IBL](https://github.com/gents83/INOX/assets/62186646/7e6ef5f5-f236-4ed2-8909-a4074614e2e3)

![Suzanne](https://github.com/gents83/INOX/assets/62186646/6ea1d292-1ae6-42b0-9265-914c6ef4d6fc)

And some older memories:

![Sponza](https://github.com/gents83/INOX/assets/62186646/1e04e4df-0966-4a1f-af51-6e44397ffaa8)

![Hot-code-reload-test](https://user-images.githubusercontent.com/62186646/130698279-9daa7b9a-1f3c-4556-be0c-37f8a1c4431e.gif)

![Profiler example](https://user-images.githubusercontent.com/62186646/120451742-f9968e80-c391-11eb-962e-13d132e09847.jpg)