# **INOX** 
## Rust Game engine integrated in Blender

[<img alt="github repository" src="https://img.shields.io/badge/github-gents83/INOX-8da0cb?logo=github" height="20">](https://github.com/gents83/INOX)
[<img alt="github pages" src="https://img.shields.io/badge/Docs-github-brightgreen" height="20">](https://gents83.github.io/INOX/)
[<img alt="github workflow sattus" src="https://img.shields.io/github/workflow/status/gents83/INOX/Deploy%20on%20Github%20Pages?style=plastic" height="20">](https://github.com/gents83/INOX/actions)
[<img alt="github sponsor" src="https://img.shields.io/github/sponsors/gents83?style=plastic" height="20">](https://github.com/sponsors/gents83)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

## INOX

It's a Game Engine written in _**Rust**_ with some bindings for external libs and with _**Blender**_ as editor.

INOX is a game engine written in _**Rust**_ and developed by [GENTS](https://twitter.com/gents83). 

The main idea behind INOX is to use [Blender](https://www.blender.org/) as external editor, even being able to create visual logic scripting nodes in it, and then have a button to launch the _**Rust**_ engine, that should be scalable to create games of any scale and for users with every kind of experience.

_**Rust**_ will give to INOX high performance, reliability and safe code, while _**Blender**_ will bring easy customization and powerful editor capabilities.


## Why INOX?

In Japanese languace _Sabi_ means things whose beauty stems from age. It refers to the patina of age, and the concept that changes due to use may make an object more beautiful and valuable. This also incorporates an appreciation of the cycles of life, as well as careful, artful mending of damage.

So it actually resemble the concept of _Rust_ in its beauty and pure meaning.

Maybe one day when the engine will be finished and fully functional it'll be possible to refer to it using Wabi-Sabi japanese expression :)


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

## Notes

Not ready yet for production.
INOX is in active development, it still lacks many features code architecture or interfaces could still change. 
New releases could still have breaking changes.