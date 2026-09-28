# Wavefront Renderer Audit, Graph Extraction, and Cross-Platform Completion

## Goal

Complete a correctness-first audit and implementation pass over the Rust and WGSL wavefront renderer, extract its pass/resource dependency graph into a portable crate, make runtime dimensions and debug diagnostics authoritative, and validate native and WebGPU execution without committing changes.

## Scope

The work covers the active wavefront graph, queue and packed-scene data contracts, dynamic resize/indexing, debug observability, runtime bounce control, transport robustness, acceleration-structure validation, pass scheduling, and native/WASM smoke coverage. Existing completed contracts in `tasks/todo.md` remain the baseline and are not replaced with duplicate mechanisms.

## Architecture

A new portable `inox_graph` crate owns generic pass nodes, resource keys, dependency edges, stable ordering, and graph validation. It has no `wgpu`, filesystem, window, or native synchronization dependency. `inox_graphics` maps graph resources to engine `Uid`s and concrete GPU passes. `inox_viewer` retains factories, platform capability policy, target creation, and debug policy while consuming the graph description.

A single runtime frame-dimension contract feeds surface configuration, target recreation, pass dispatch dimensions, shader indexing, queue capacity, and debug buffers. Full-resolution storage strides remain explicit when render scale is reconstruction-only. All queue dispatches are driven by GPU-written counters and reset arguments, with runtime bounce count selecting active work inside the fixed portable stage budget.

## Required invariants

- Every active pass has a unique graph node and a validated predecessor/resource handoff.
- The graph is acyclic and its topological order matches concrete registration order.
- CPU shared records are `#[repr(C)]` with tested size, alignment, and offsets matching WGSL.
- Runtime shaders never use `DEFAULT_WIDTH` or `DEFAULT_HEIGHT` for storage indexing.
- Surface, target, pass, constant-data, queue-stride, and debug dimensions agree after initial creation and resize.
- Queue capacities are checked against adapter limits and checked arithmetic; overflow is observable.
- Empty GPU queues produce zero dispatch arguments and never reuse stale indirect work.
- Runtime bounce count is bounded and changes do not require rebuilding the graph.
- Native diagnostics do not enter the reusable graph crate or WASM paths.
- Optional GPU features and limits are negotiated from adapter capabilities.
- Native and WASM shader preprocessing/validation use equivalent include and binding rules.
- Completion claims require actual formatting, tests, clippy/build, and available native/WebGPU smoke evidence.

## Implementation sequence

1. Run the current baseline gates and inventory findings against existing tests.
2. Add and migrate the reusable graph crate and remove duplicated viewer graph metadata.
3. Centralize frame dimensions, resize/target updates, dynamic indexing, and debug options.
4. Make queue allocation and bounce scheduling capability-aware and runtime-configurable.
5. Audit and harden transport and acceleration-structure traversal without changing packed formats unnecessarily.
6. Add causal native/WASM smoke checks and strict final verification.

## Non-goals

Do not replace the renderer with a new backend, add unsupported hardware-specific ray-tracing requirements, or introduce speculative BVH formats. Do not publish generated artifacts. Do not create a branch, worktree, or commit.
