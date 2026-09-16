# Production Portable Wavefront Pathtracing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the remaining production steps in `tasks/todo.md` so the packed wavefront pathtracer is portable, observable, transport-complete, and validated on native and WebGPU targets.

**Architecture:** Continue the existing queue-driven renderer in the `wavefront-pathtracer` worktree. Keep all production shaders within the portable eight-storage-buffer stage budget, expose CPU/WGSL layout/version contracts through typed helpers, and make optional GPU capabilities explicit rather than unconditional. Finish the renderer in layers: regression contracts and platform safety, shader/data validation, light transport and generalized bounce queues, then ordered build/runtime gates.

**Tech Stack:** Rust workspace/Cargo, wgpu 30, WGSL/Naga shader preprocessing, wasm-bindgen/WebGPU, existing INOX render/graphics/viewer passes, integration tests under crate `tests/` directories.

**Spec:** `tasks/todo.md`

## Global Constraints

- Preserve the packed-scene and queue-based production graph; do not reactivate the legacy direct/indirect lighting passes.
- Every active production shader must remain within the portable eight-storage-buffer-per-stage profile.
- CPU records shared with WGSL must be `#[repr(C)]`, explicitly sized/aligned, versioned, and covered by tests.
- Optional adapter features must be requested only when supported; fallback paths must remain valid on WebGPU/mobile profiles.
- Shader reload and binarizer failures must preserve the last-known-good resource and emit an actionable diagnostic.
- Keep tests in crate-level `tests/test_TOPIC.rs` files and run the workspace manifest explicitly.
- Do not publish or copy generated artifacts unless both PC and Web binarization and generated-data validation succeed.

---

### Task 1: Restore regression contracts and test inventory

**Files:**
- Modify: `tasks/todo.md`
- Create/modify: `crates/resources/tests/test_buffer.rs`, `crates/render/tests/test_buffer_ref.rs`, `crates/render/tests/test_packed_data.rs`, `crates/render/tests/test_packed_scene.rs`, `crates/render/tests/test_shapes3d.rs`
- Create/modify: `crates/bvh/tests/test_*`, `crates/nodes/tests/test_*`, `crates/plugins/common_script/tests/test_*`, `crates/plugins/viewer/tests/test_*`
- Test: the crate integration-test files above

**Interfaces:**
- Consumes the existing `BufferRef`, packed-scene, shape, BVH, node, common-script, viewer-config, and pass-order APIs.
- Produces deterministic regression tests that later portability and graph tasks can rely on.

- [ ] **Step 1: Inventory existing embedded tests and public constructors.** Record each behavior that was moved or lost during the wavefront changes: logical buffer ranges, packed header/ranges, cylinder/torus connectivity, BVH traversal invariants, node serialization, script registration, viewer config defaults, and production pass order.
- [ ] **Step 2: Add failing assertions for missing contracts.** Cover buffer shrink/grow/release, packed header magic/version/range bounds, exact `repr(C)` sizes, shape index bounds/connectivity, and pass-order uniqueness/position. Use the existing public APIs; do not duplicate private implementation logic.
- [ ] **Step 3: Restore focused behavior tests.** Add tests for BVH/node/common-script public behavior and viewer config deserialization/defaults. Gate native-only tests with the existing target cfgs where the crate cannot compile on WebAssembly.
- [ ] **Step 4: Run focused tests.**

```bash
cargo test -p inox_resources --manifest-path ./crates/Cargo.toml --test test_buffer
cargo test -p inox_render --manifest-path ./crates/Cargo.toml --test test_buffer_ref --test test_packed_data --test test_packed_scene --test test_shapes3d
cargo test -p inox_viewer --manifest-path ./crates/Cargo.toml --test test_pass_order
```

- [ ] **Step 5: Mark only verified coverage complete in `tasks/todo.md`.**

---

### Task 2: Make Web canvas ownership and surface setup robust

**Files:**
- Modify: `crates/platform/src/platform_impl/web/window.rs`
- Modify: `crates/platform/src/platform_impl/web/handle.rs` and `crates/platform/src/platform_impl/web/mod.rs` if handle ownership requires it
- Modify: `web/index.html`
- Test: `crates/platform/tests/test_web_window.rs` or the existing platform test file

**Interfaces:**
- Consumes `Window::create_handle`, `WindowEvent`, `Handle`, and the HTML canvas with id `canvas`.
- Produces one stable canvas owner, a nonzero stable handle, CSS-to-device-pixel mouse coordinates, resize events, and a valid WebGPU surface target.

- [ ] **Step 1: Write failing web-contract tests.** Test the handle id is nonzero and stable, CSS dimensions never divide by zero, device-pixel dimensions are clamped to at least one, and resize/mouse events use the same canvas owner. Keep DOM-dependent assertions in wasm/browser cfgs and test pure coordinate conversion natively.
- [ ] **Step 2: Replace repeated DOM lookup with a retained canvas ownership path.** Store or route all listeners through the canvas selected during handle creation; use `get_bounding_client_rect` with finite positive dimensions and clamp normalized coordinates to `[0, 1]`.
- [ ] **Step 3: Make resize update width, height, scale factor, and the surface configuration from one event path.** Preserve the handle and avoid creating a second canvas or surface owner.
- [ ] **Step 4: Update `web/index.html` CSS and canvas attributes so layout dimensions are distinct from device-pixel dimensions.**
- [ ] **Step 5: Run the platform tests and WebAssembly check.**

```bash
cargo test -p inox_platform --manifest-path ./crates/Cargo.toml
RUSTFLAGS='--cfg=web_sys_unstable_apis --cfg getrandom_backend="wasm_js"' cargo check -p inox_platform --manifest-path ./crates/Cargo.toml --target wasm32-unknown-unknown
```

---

### Task 3: Negotiate adapter features and limits

**Files:**
- Modify: `crates/render/src/platform/mod.rs`
- Modify: `crates/render/src/platform/windows.rs`, `unix.rs`, `macos.rs`, `android.rs`, `ios.rs`, `wasm.rs`
- Modify: `crates/render/src/common/render_context.rs`
- Test: `crates/render/tests/test_platform_limits.rs`

**Interfaces:**
- Consumes `wgpu::Adapter::features`, `wgpu::Adapter::limits`, `required_gpu_features`, `required_gpu_limits`, and `RenderCapabilities`.
- Produces a filtered `DeviceDescriptor` whose required features are a supported subset and whose required limits never exceed adapter limits, with capability flags available to pass creation.

- [ ] **Step 1: Add tests for feature filtering and limit clamping.** Include an adapter with no optional features, an adapter with only a subset, and a limit below the requested storage-buffer count. Assert that unsupported optional features are omitted and portable limits remain valid.
- [ ] **Step 2: Implement `negotiate_gpu_requirements(adapter_features, adapter_limits)` in `platform/mod.rs`.** Separate required baseline capabilities from optional wireframe, timestamp, indirect-count, array, and primitive-index capabilities. Clamp each requested limit to the adapter while preserving the portable minimum; return a structured error if the minimum cannot be met.
- [ ] **Step 3: Use negotiation in `RenderContext::new` before `request_device`.** Never pass the unconditional platform feature mask or a limit larger than the adapter reports.
- [ ] **Step 4: Make dependent passes inspect `RenderCapabilities` and disable unsupported diagnostics/features rather than panic.**
- [ ] **Step 5: Run render tests, clippy, and WebAssembly check.**

---

### Task 4: Preserve last-known-good shader and pipeline resources

**Files:**
- Modify: `crates/render/src/resources/shader.rs`
- Modify: `crates/render/src/resources/compute_pipeline.rs`, `crates/render/src/resources/render_pipeline.rs`
- Modify: `crates/render/src/systems/update_system.rs`
- Modify: `crates/render/src/common/render_context.rs`
- Test: `crates/render/tests/test_shader_reload.rs`

**Interfaces:**
- Consumes resource initialization/reload hooks and existing `check_shaders_to_reload` calls.
- Produces a reload result/status carrying path, diagnostic, and whether the previous resource remains active; failed replacement never clears a valid shader or pipeline.

- [ ] **Step 1: Add tests for successful replacement and failed replacement.** Build a fake/standalone reload state where a valid resource remains initialized after invalid source or pipeline data is presented; assert the diagnostic is retained for observability.
- [ ] **Step 2: Refactor shader initialization to return a structured error rather than only `bool`.** Log path, stage, source/include location, and compiler diagnostic while retaining the previous module.
- [ ] **Step 3: Refactor compute/render pipeline reload to build the replacement off to the side.** Swap only after shader module, bind-group layout, and pipeline creation all succeed.
- [ ] **Step 4: Ensure update-system file watcher handling catches errors and continues processing later files.**
- [ ] **Step 5: Run focused reload tests and a viewer startup smoke.**

---

### Task 5: Remove hot-loop allocations, repeated locks, and lookup hazards

**Files:**
- Modify: `crates/render/src/resources/compute_pass.rs`, `crates/render/src/resources/render_pass.rs`
- Modify: `crates/render/src/common/render_context.rs`, `crates/render/src/common/global_buffers.rs`
- Modify: `crates/graphics/src/passes/*.rs` only where repeated target/meshlet locking remains
- Test: `crates/graphics/tests/test_passes.rs`, `crates/render/tests/test_*`

**Interfaces:**
- Consumes existing pass dispatch/draw methods and render-target/resource handles.
- Produces cached per-pass pipeline/binding metadata, one-time target/meshlet handle acquisition per dispatch, and correct indirect draw stride/count behavior without changing public rendering semantics.

- [ ] **Step 1: Add a regression test that records dispatch arguments and verifies no per-dispatch pipeline vector rebuild and correct indirect stride.**
- [ ] **Step 2: Cache immutable pipeline/binding metadata at pass initialization or pipeline replacement.** Reuse scratch command storage owned by the pass instead of allocating in every frame.
- [ ] **Step 3: Narrow lock scopes in render-target and meshlet paths.** Clone stable ids/handles while holding the lock, then perform GPU encoding after releasing it; avoid repeated `match_resource` scans inside a workgroup loop.
- [ ] **Step 4: Run pass and render tests under debug and release profiles.**

---

### Task 6: Enforce generated shader storage-binding and packed-layout contracts

**Files:**
- Modify: `crates/plugins/binarizer/src/compilers/shader_compiler.rs`
- Modify: `crates/plugins/binarizer/src/system/binarizer.rs`
- Create/modify: `crates/plugins/binarizer/tests/test_shader_bindings.rs`, `crates/plugins/binarizer/tests/test_shader_compiler.rs`
- Modify: `crates/render/src/data/packed_data.rs`, `crates/render/src/data/packed_scene.rs`, `crates/render/src/data/mod.rs`
- Modify: `data_raw/shaders/wgsl/common.inc`, `data_raw/shaders/wgsl/wavefront_records.inc`
- Test: binarizer and packed-data integration tests

**Interfaces:**
- Consumes preprocessed WGSL/pipeline sources and CPU packed record definitions.
- Produces validation errors containing shader/pipeline path, stage, and counted storage bindings; generated packed data carries exact magic/version/word count and typed record strides.

- [ ] **Step 1: Add failing tests for eight-storage-buffer violations, duplicate bindings, missing packed header/version, and CPU/WGSL stride mismatch.**
- [ ] **Step 2: Implement WGSL binding extraction after include expansion.** Count `var<storage>` declarations by shader stage and reject active production shaders over eight; validate pipeline references against the compiled shader.
- [ ] **Step 3: Add typed layout descriptors for `RayQueueRecord`, `HitRecord`, `AccumulationRecord`, `ReservoirRecord`, `AoRecord`, `TransparentHitRecord`, `DispatchArgs`, and `QueueHeader`.** Assert `size_of`, alignment, and field offsets in Rust tests and mirror constants in `wavefront_records.inc`.
- [ ] **Step 4: Validate `PackedSceneHeader` before upload and expose a single version check used by both CPU builders and WGSL consumers.**
- [ ] **Step 5: Run binarizer tests and PC/Web shader validation.**

---

### Task 7: Prove active pass graph order and binding budget

**Files:**
- Modify: `crates/plugins/viewer/src/viewer.rs`
- Modify: `crates/plugins/viewer/tests/test_pass_order.rs`
- Create/modify: `crates/graphics/tests/test_wavefront_bindings.rs`
- Modify: `tasks/todo.md`

**Interfaces:**
- Consumes `PRODUCTION_PASS_ORDER`, pass names/ids, pipeline metadata, and binarizer binding validation.
- Produces deterministic graph tests proving each active stage has exactly one predecessor ordering and remains within eight storage buffers.

- [ ] **Step 1: Add a failing test that enumerates active production passes and compares their names to `PRODUCTION_PASS_ORDER`.** Assert legacy G-buffer/direct/indirect passes are disabled and the finalize/blit tail is last.
- [ ] **Step 2: Add dependency assertions for primary → dispatch → traversal → shade → bounce and direct candidates → visibility → composite.** Reject duplicate pass ids and missing queue counter dependencies.
- [ ] **Step 3: Connect shader binding validation to all active wavefront pipeline descriptors and report the first violating path.**
- [ ] **Step 4: Run viewer/graphics tests and update the TODO only after the graph and budget checks pass.**

---

### Task 8: Add packed light hierarchy and alias-table candidate updates

**Files:**
- Modify: `crates/render/src/data/packed_scene.rs`, `crates/render/src/data/packed_data.rs`
- Modify: `crates/render/src/data/light_data.rs`, `crates/render/src/common/global_buffers.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_direct_candidates.rs`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_direct_candidates.wgsl`, `data_raw/shaders/wgsl/wavefront_records.inc`
- Test: `crates/render/tests/test_packed_scene.rs`, `crates/graphics/tests/test_wavefront_lighting.rs`

**Interfaces:**
- Consumes packed lights and per-frame scene updates.
- Produces a bounded hierarchy/alias representation and deterministic candidate proposals with valid PDFs, selection weights, and overflow handling.

- [ ] **Step 1: Add CPU tests for alias construction with zero, one, and many lights, including nonfinite/zero power sanitization.**
- [ ] **Step 2: Implement packed light hierarchy/alias updates with stable version and count fields.** Keep capacity bounded and mark overflow instead of indexing past the allocation.
- [ ] **Step 3: Update the candidate shader to sample the alias table, compute a nonzero proposal PDF for valid lights, and fall back to a bounded uniform candidate when the table is empty.**
- [ ] **Step 4: Preserve the existing reservoir reuse and confidence/Jacobian correction while replacing only candidate generation.**
- [ ] **Step 5: Run focused lighting tests and shader validation.**

---

### Task 9: Complete environment, multi-bounce, roulette, reflection, and refraction transport

**Files:**
- Modify: `crates/graphics/src/passes/compute_wavefront_next_bounce.rs`, `compute_wavefront_traversal.rs`, `compute_wavefront_shade.rs`, `compute_wavefront_composite.rs`
- Modify: `crates/graphics/src/passes/compute_wavefront_dispatch.rs`
- Modify: `data_raw/shaders/wgsl/compute_wavefront_next_bounce.wgsl`, `compute_wavefront_shade.wgsl`, `compute_wavefront_composite.wgsl`, `compute_wavefront_traversal.wgsl`, `wavefront_records.inc`
- Modify: `crates/render/src/data/constant_data.rs`
- Test: `crates/graphics/tests/test_wavefront_transport.rs`

**Interfaces:**
- Consumes queue headers and ping-pong queue bindings from the existing bounded chain.
- Produces generalized depth-bounded bounce queues, environment radiance on misses, unbiased Russian-roulette termination after the configured minimum depth, and reflection/transmission rays with throughput/PDF updates.

- [ ] **Step 1: Add CPU-side transport tests for empty queues, depth termination, roulette probability bounds, reflection direction, total-internal-reflection fallback, and transmission attenuation.**
- [ ] **Step 2: Add explicit max-depth/min-roulette-depth and environment sampler parameters to the constant-data contract.** Keep defaults deterministic and portable.
- [ ] **Step 3: Implement environment sampling for misses using the configured environment texture index and an analytic sky fallback when no texture is valid.**
- [ ] **Step 4: Replace the fixed three-bounce setup with a queue loop driven by dispatch counters, bounded by max depth and early termination.** Ensure every ping-pong queue is reset before append and every empty queue produces zero dispatch workgroups.
- [ ] **Step 5: Implement lobe sampling with MIS/PDF-consistent throughput for diffuse, glossy reflection, and transmission/refraction.** Handle total internal reflection by converting the transmission sample to reflection without producing NaNs.
- [ ] **Step 6: Run transport tests, generated shader validation, and a Cornell-box/native smoke.**

---

### Task 10: Run ordered deployment gates and document artifacts

**Files:**
- Modify: `.github/workflows/rust-windows.yml`, `.github/workflows/rust-unix.yml`, `.github/workflows/rust-web.yml` only if gates are absent or out of order
- Modify: `.vscode/tasks.json` or `.vscode/launch.json` only if local gate commands are missing
- Modify: `tasks/todo.md`
- Create: `scripts/validate_wavefront_pipeline.ps1` if no existing script can express the strict order

**Interfaces:**
- Consumes the PC/Web binarizer, Cargo build/test/clippy, wasm-bindgen, and browser smoke commands already documented in `CLAUDE.md`.
- Produces no published output unless all generation and validation stages succeed; records exact commands and failures.

- [ ] **Step 1: Add a strict gate script/test that stops on the first failure and deletes or quarantines stale generated output before starting.** Order: native PC binarizer → generated-data validation → native tests/clippy/build → Web binarizer → wasm build/bindgen → browser WebGPU smoke.
- [ ] **Step 2: Add a browser smoke route that loads the generated WebGPU shell, verifies canvas/surface initialization, and runs long enough to observe at least one completed frame without a console error.**
- [ ] **Step 3: Run the complete native and Web gates on the current machine, recording unavailable external dependencies explicitly rather than treating skipped work as passed.**
- [ ] **Step 4: Run `cargo fmt`, workspace tests, clippy, native release build, wasm check/build, and the gate script again from a clean generated-data state.**
- [ ] **Step 5: Update `tasks/todo.md` review with command output summaries and mark only gates actually completed.**
- [ ] **Step 6: Keep generated artifacts unpublished unless the PC+Web gate and generated-data validation both pass.**

---

## Final verification checklist

- [ ] `cargo fmt --manifest-path ./crates/Cargo.toml --all -- --check`
- [ ] `cargo test --workspace --manifest-path ./crates/Cargo.toml`
- [ ] `cargo clippy --workspace --all-targets --manifest-path ./crates/Cargo.toml --no-deps -- -D warnings`
- [ ] Native release build with CI WebGPU flags
- [ ] WebAssembly release build and pinned wasm-bindgen generation
- [ ] PC and Web binarizer gates
- [ ] Native viewer startup smoke
- [ ] Browser WebGPU smoke
- [ ] `tasks/todo.md` review updated with actual results
