# **NRG** 
## New Rust Game engine - with Blender as Editor

[<img alt="github" src="https://img.shields.io/badge/github-gents83/NRG-8da0cb?logo=github" height="20">](https://github.com/gents83/NRG)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

## NRG

It's a Game Engine written in _**Rust**_ with some bindings for external libs and with _**Blender**_ as editor.

NRG is a game engine written in _**Rust**_ and developed by [GENTS](https://twitter.com/gents83). 

The main idea behind NRG is to use [Blender](https://www.blender.org/) as external editor, even being able to create visual logic scripting nodes in it, and then have a button to launch the _**Rust**_ engine, that should be scalable to create games of any scale and for users with every kind of experience.

_**Rust**_ will give to NRG high performance, reliability and safe code, while _**Blender**_ will bring easy customization and powerful editor capabilities.

## Vision

The engine is developed with following pillars:
- The game engine should be obviously written in _**Rust**_
- The engine should support multiple platforms (PC, Mobile, Sony Playstation, Microsoft XBox, Nintendo Switch, etc)
- The rendering engine should support different GFX API as well (like Vulkan, DirectX, Metal, etc)
- The engine should be multi-threaded both on CPU and GPU to reach high-end performances
- The engine should support streaming, quick background loading and hot-reload of Code and Data
- _**Blender**_ should be used as external 3D scene, Material, Animation and other content edition 
- NRG should generate a _**Blender**_ addon that could be installed in order to launch and communicate with it
- _**Blender**_ should be used as external editor with new custom NRG editors (like Logic Node Visual Scripting, etc) 
- From _**Blender**_ the user should be able to see the same scene rendered in NRG just pressing a button
- NRG Engine could be used just as a high quality rendering engine
- NRG Engine could be used to run game logic  

## Notes

Not ready yet for production.
NRG is in active development, it still lacks many features code architecture or interfaces could still change. 
New releases could still have breaking changes.