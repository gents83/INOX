# Wavefront Audit and Graph Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Audit and complete the portable wavefront renderer, extract a reusable dependency graph, fix runtime dimension/debug correctness, and prove native/WASM behavior.

**Architecture:** Keep concrete GPU passes in `inox_graphics` and viewer factories in `inox_viewer`; move generic pass/resource/dependency modeling and validation into a new dependency-light `inox_graph` crate. Centralize frame dimensions and use GPU counters for runtime bounce scheduling. Preserve packed-scene formats unless tests prove a format defect.

**Tech Stack:** Rust workspace, wgpu, WGSL/Naga, wasm32-unknown-unknown, existing binarizer and headless WebGPU smoke infrastructure.

**Spec:** `docs/superpowers/specs/2026-09-17-wavefront-audit-graph-design.md`

## Global Constraints

- Work only in the current checkout; do not create branches, worktrees, or commits.
- Keep graph code free of `wgpu`, filesystem, window, and native synchronization dependencies.
- Preserve the portable eight-storage-buffer-per-stage budget.
- Use checked arithmetic for dimensions, capacities, offsets, and packed ranges.
- Keep full-resolution storage strides explicit when render scale is reconstruction-only.
- Keep native-only readback behind target configuration; WASM uses GPU-visible diagnostics.
- Run tests after each task and record actual results in `tasks/todo.md`.

---

### Task 1: Establish baseline and audit report

**Files:**
- Modify: `tasks/todo.md`
- Create: `docs/superpowers/reports/2026-09-17-wavefront-audit.md`
- Test: existing workspace and focused wavefront tests

**Interfaces:**
- Consumes the existing viewer registration table, `wavefront_bounce_stages`, packed-scene builders, shader validator, and smoke tests.
- Produces a checked-in-place audit report listing verified invariants, confirmed defects, and tests that reproduce each defect.

- [ ] Run `cargo fmt --manifest-path ./crates/Cargo.toml --all -- --check` and focused viewer/graphics/render/binarizer tests with `RUSTFLAGS=` on Windows.
- [ ] Run the existing shader binding and native headless causal smoke tests; record adapter skips separately from failures.
- [ ] Compare `viewer.rs`, `pathtracing_common.rs`, `update_system.rs`, `debug.rs`, `global_buffers.rs`, and all active wavefront WGSL files against the spec invariants.
- [ ] Add only confirmed findings to the audit report, including exact file/line references and the test that will prove each fix.
- [ ] Update `tasks/todo.md` with baseline command results without marking skipped browser work as passed.

Run:
```bash
RUSTFLAGS= cargo test --workspace --manifest-path ./crates/Cargo.toml --test test_pass_order -p inox_viewer
RUSTFLAGS= cargo test --workspace --manifest-path ./crates/Cargo.toml --test test_wavefront_bindings -p inox_graphics
RUSTFLAGS= cargo test --workspace --manifest-path ./crates/Cargo.toml --test test_wavefront_integration -p inox_graphics
```

---

### Task 2: Add the portable dependency graph crate

**Files:**
- Create: `crates/graph/Cargo.toml`
- Create: `crates/graph/src/lib.rs`
- Create: `crates/graph/tests/test_graph.rs`
- Modify: `crates/Cargo.toml`

**Interfaces:**
- Produces `Graph<R>`, `PassNode<R>`, `Dependency<R>`, `GraphError`, `GraphValidation`, and deterministic `Graph::topological_order()`.
- `R` must be `Clone + Eq + Hash`; graph validation returns structured errors rather than panicking.

- [ ] Add the crate to the workspace with no GPU/native dependencies.
- [ ] Write tests for stable order, duplicate node names, unknown predecessors/resources, cycles, duplicate dependencies, disabled nodes, and resource handoff continuity.
- [ ] Implement node and edge types with explicit enabled state and resource key ownership.
- [ ] Implement Kahn topological sorting using declaration order as the deterministic tie-breaker.
- [ ] Implement validation that rejects duplicate names, unknown endpoints, cycles, and an edge whose resource is not produced by its predecessor or consumed by its successor metadata.
- [ ] Run `cargo test -p inox_graph --manifest-path ./crates/Cargo.toml` and `cargo check -p inox_graph --target wasm32-unknown-unknown`.

Example public shape:
```rust
pub struct PassNode<R> {
    pub name: String,
    pub enabled: bool,
    pub reads: Vec<R>,
    pub writes: Vec<R>,
}

pub struct Dependency<R> {
    pub predecessor: String,
    pub successor: String,
    pub resource: R,
}

impl<R: Clone + Eq + Hash> Graph<R> {
    pub fn validate(&self) -> Result<GraphValidation, GraphError>;
    pub fn topological_order(&self) -> Result<Vec<String>, GraphError>;
}
```

---

### Task 3: Migrate viewer registration and bounce metadata

**Files:**
- Modify: `crates/plugins/viewer/Cargo.toml`
- Modify: `crates/plugins/viewer/src/viewer.rs`
- Modify: `crates/graphics/Cargo.toml`
- Modify: `crates/graphics/src/passes/pathtracing_common.rs`
- Modify: `crates/plugins/viewer/tests/test_pass_order.rs`
- Modify: `crates/graphics/tests/test_wavefront_bindings.rs`

**Interfaces:**
- `inox_viewer` builds an `inox_graph::Graph<Uid>` from one registration source and validates it before constructing concrete passes.
- `inox_graphics` exposes wavefront stage resource metadata but no viewer-specific graph types.

- [ ] Write a failing adapter test that constructs the production graph and asserts the expected active order, disabled legacy nodes, eight bounce stages, and all queue/header/dispatch handoffs.
- [ ] Move generic `ProductionPassMetadata`, `ProductionGraphDependency`, and validation responsibilities to graph types while retaining a viewer-specific factory record containing `ProductionPassFactory`.
- [ ] Make `production_pass_registrations()` produce graph nodes and factory records from the same declaration data; eliminate `production_construction_records()` duplication by deriving construction resources from graph metadata.
- [ ] Move `wavefront_bounce_stage` and `wavefront_bounce_stages` metadata generation to the graph-facing module, preserving all existing UIDs and stage names.
- [ ] Validate the graph before `create_render_passes()` and return/log a clear diagnostic instead of relying on later registration assertions.
- [ ] Remove inactive manually wired bounce factory functions after tests prove they are unused.
- [ ] Run viewer and graphics focused tests plus native/WASM checks for both crates.

---

### Task 4: Centralize runtime frame dimensions and resize propagation

**Files:**
- Create or modify: `crates/render/src/common/frame_dimensions.rs`
- Modify: `crates/render/src/common/mod.rs`
- Modify: `crates/render/src/systems/update_system.rs`
- Modify: `crates/plugins/viewer/src/viewer.rs`
- Modify: affected pass constructors/setters under `crates/graphics/src/passes/`
- Create: `crates/render/tests/test_frame_dimensions.rs`
- Create: `crates/graphics/tests/test_dynamic_dimensions.rs`

**Interfaces:**
- `FrameDimensions { surface: Extent2d, render: Extent2d, storage_stride: u32, dispatch: [u32; 2] }` is validated and derived from one input event.
- Resize updates surface configuration, target sizes, pass dimensions, `ConstantData`, queue capacities, and debug allocations through one path.

- [ ] Add pure tests for zero-size clamping, checked pixel counts, render-scale policy, dispatch rounding, and resize equality across all derived dimensions.
- [ ] Implement checked `FrameDimensions::new(width, height, render_scale)` with minimum one-pixel extents and full-resolution storage-stride semantics.
- [ ] Replace `UpdateSystem`’s independent default-size initialization with the window’s actual initial dimensions.
- [ ] Add a render-context/viewer resize hook that recreates or resizes all dimension-dependent targets and updates pass dimensions before the next frame.
- [ ] Ensure every wavefront pass uses the dimension contract for dispatch and capacity; no pass caches stale dimensions after resize.
- [ ] Add a resize regression test that changes from non-default dimensions and verifies constant data, target dimensions, and dispatch dimensions agree.
- [ ] Run render/graphics tests and native viewer startup smoke at a non-default window size where the environment permits.

---

### Task 5: Fix dynamic WGSL indexing and debug pass

**Files:**
- Modify: `data_raw/shaders/wgsl/debug.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_surface.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_transparency.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_direct_candidates.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_visibility.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_ao_resolve.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_composite.wgsl`
- Modify: `crates/graphics/src/passes/debug.rs`
- Modify: `crates/plugins/viewer/src/viewer.rs`
- Create/modify: `crates/graphics/tests/test_dynamic_dimensions.rs`
- Create/modify: `crates/graphics/tests/test_debug_pass.rs`

**Interfaces:**
- Debug pass receives `FrameDimensions` and an explicit `DebugOptions` value.
- WGSL indexes use `u32(constant_data.screen_width)` and `screen_height` or queue-record pixel indices, with bounds guards before storage access.

- [ ] Add source-contract tests that reject default-resolution indexing in all active and debug shaders and assert dynamic width/height use.
- [ ] Fix debug buffer allocation to use the actual dimension contract and correct element-size semantics.
- [ ] Replace every debug shader default stride with runtime dimensions and add finite/bounds guards.
- [ ] Add `DebugOptions` for pass graph, queue headers, dispatch args, resource IDs, and visual buffer selection; keep default behavior disabled unless configured.
- [ ] Re-enable debug pass construction behind the explicit option and ensure it does not alter production output when disabled.
- [ ] Validate debug shader/pipeline through the binarizer and run focused tests.

---

### Task 6: Make runtime bounces and queue allocation capability-aware

**Files:**
- Modify: `crates/graphics/src/passes/pathtracing_common.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_primary.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_dispatch.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_traversal.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_next_bounce.rs`
- Modify: `crates/render/src/common/global_buffers.rs`
- Modify: `crates/render/src/data/constant_data.rs`
- Modify: `crates/render/src/platform/mod.rs`
- Create/modify: `crates/graphics/tests/test_wavefront_transport.rs`
- Create/modify: `crates/render/tests/test_constant_data.rs`

**Interfaces:**
- `queue_capacity_for_resolution(width, height, record_size, multiplier, limits) -> Result<u32, CapacityError>` uses checked arithmetic and negotiated limits.
- `ConstantData::set_num_bounces` remains runtime-safe and clamps to the graph’s maximum stage count.

- [ ] Add tests for zero, one, maximum, and out-of-range bounce counts; assert no graph rebuild and no dispatch for inactive stages.
- [ ] Add capacity tests for overflow, adapter storage limits, non-default dimensions, and transparency multipliers.
- [ ] Replace unconditional maximum queue allocations where safe with dimension/capability-derived capacities while retaining a bounded fallback.
- [ ] Ensure every queue header and indirect argument buffer is reset before append/dispatch, including inactive stages.
- [ ] Assert dispatch shaders clamp count to capacity and write zero arguments for empty queues.
- [ ] Add diagnostics exposing requested capacity, actual capacity, overflow, active bounce count, and dropped records.
- [ ] Run transport, binding, render, clippy, and WASM checks.

---

### Task 7: Audit and harden packed scene, BVH, BLAS/TLAS, and meshlet traversal

**Files:**
- Modify: `crates/bvh/src/*.rs`
- Modify: `crates/render/src/data/packed_scene.rs`
- Modify: `crates/render/src/common/global_buffers.rs`
- Modify: `data_raw/shaders/wgsl/raytracing.inc`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_traversal.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_ray_traversal.wgsl`
- Modify: `data_raw/shaders/wgsl/common.inc`
- Create/modify: `crates/bvh/tests/test_bvh.rs`
- Create/modify: `crates/render/tests/test_packed_scene.rs`
- Create/modify: `crates/graphics/tests/test_wavefront_integration.rs`

**Interfaces:**
- Existing packed-scene record formats remain stable unless a failing contract proves a required correction.
- Traversal helpers return bounded indices/termination states and never read outside packed ranges.

- [ ] Add failing tests for empty BVHs, one-node BVHs, degenerate bounds, invalid escape links, instance transforms, meshlet ranges, and packed offsets.
- [ ] Implement checked builders and validators for node ranges, BLAS/TLAS references, meshlet indices, and scene word spans.
- [ ] Harden WGSL traversal with explicit stack/range bounds, finite ray/bounds checks, and bounded iteration exits.
- [ ] Verify cluster/meshlet LOD metadata and triangle index decoding against packed CPU data.
- [ ] Measure or test workgroup-local traversal behavior; retain portable workgroup sizes and avoid divergent unbounded loops.
- [ ] Run BVH/render/graphics tests, shader validation, and the causal native smoke.

---

### Task 8: Strengthen transport quality and pass scheduling

**Files:**
- Modify: `data_raw/shaders/wgsl/compute_wavefront_next_bounce.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_shade.wgsl`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_composite.wgsl`
- Modify: `crates/graphics/src/passes/compute_wavefront_next_bounce.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_shade.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_composite.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_denoise.rs`
- Create/modify: `crates/graphics/tests/test_wavefront_transport.rs`
- Create/modify: `crates/graphics/tests/test_passes.rs`

**Interfaces:**
- All lobe and environment sampling functions return finite direction, throughput, and PDF values or an explicit termination result.
- Graph scheduling remains GPU-counter-driven and preserves direct/AO/indirect/final accumulator separation.

- [ ] Add property tests for diffuse, glossy, reflection, refraction, TIR, roulette, environment misses, emissive hits, and zero-PDF termination.
- [ ] Audit MIS/PDF/eta updates and fix only cases producing bias, NaN/Inf, or invalid directions under the current material contract.
- [ ] Add bounded temporal history invalidation for camera/resize/material changes and verify denoiser inputs use the correct full-resolution stride.
- [ ] Verify compute-vs-render choices: retain compute for queue/denoise/reconstruction work and render passes only for display/final composition.
- [ ] Cache immutable pass metadata and avoid per-frame allocations or repeated resource locks in active wavefront passes.
- [ ] Run focused transport/pass tests and causal native smoke.

---

### Task 9: Add cross-platform diagnostics and smoke coverage

**Files:**
- Modify: `crates/graphics/src/passes/pathtracing_common.rs`
- Modify: `crates/plugins/viewer/src/viewer.rs`
- Modify: `crates/app/src/platform/wasm.rs`
- Modify: `web/index.html`
- Modify: `scripts/validate_wavefront_pipeline.ps1`
- Modify: `crates/graphics/tests/test_wavefront_integration.rs`
- Create/modify: `crates/platform/tests/test_web_window.rs`

**Interfaces:**
- Native diagnostics use bounded asynchronous-safe readback or post-submit callbacks.
- WASM diagnostics use DOM markers such as `data-inox-smoke`, frame counters, error counts, and queue diagnostic summaries without synchronous GPU polling.

- [ ] Add native smoke assertions for graph order, dynamic dimensions, queue headers, finite output, nonzero output, and at least one completed frame.
- [ ] Add WASM/browser smoke assertions for canvas ownership, configured dimensions, device/queue setup, completed frame, no console errors, and debug marker availability.
- [ ] Ensure browser smoke reports unavailable browser/WebGPU as a skip with preserved logs, not a pass.
- [ ] Make shader include expansion and storage-binding validation share the production preprocessing implementation or test exact equivalence.
- [ ] Run native and WASM checks under their supported target configurations.

---

### Task 10: Complete strict verification and document results

**Files:**
- Modify: `tasks/todo.md`
- Modify: `docs/superpowers/reports/2026-09-17-wavefront-audit.md`
- Modify: `scripts/validate_wavefront_pipeline.ps1` only if a verified gate is missing

**Interfaces:**
- Consumes all graph, dimension, transport, acceleration, diagnostic, and smoke contracts.
- Produces a final evidence table with command, target, result, and explicit skip reason where applicable.

- [ ] Run `cargo fmt --manifest-path ./crates/Cargo.toml --all -- --check`.
- [ ] Run `RUSTFLAGS= cargo test --workspace --manifest-path ./crates/Cargo.toml`.
- [ ] Run `RUSTFLAGS= cargo clippy --workspace --all-targets --manifest-path ./crates/Cargo.toml --no-deps -- -D warnings`.
- [ ] Run native release build and existing native viewer smoke.
- [ ] Run PC and Web binarizer plus generated-data validation.
- [ ] Run WASM release/check and pinned wasm-bindgen generation.
- [ ] Run browser WebGPU smoke when a supported browser is available; record unavailable status otherwise.
- [ ] Update the audit report and `tasks/todo.md` only with observed results. Do not publish artifacts and do not commit.
