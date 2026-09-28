# Wavefront Resolution and Resize Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the wavefront pathtracer render the Cornell Box correctly at arbitrary supported extents and remain correct through startup and live window resizing.

**Architecture:** Preserve the existing authoritative `FrameDimensions` contract and resize transaction. Harden the depth-pyramid tail shader against rectangular non-power-of-two extents, make camera projection match the same size contract from the first frame, and add tests/smokes that prove GPU scene handoff, image correctness, and resource dimensions after resize. Do not introduce a second resize system or change the production reconstruction-only render-scale policy.

**Tech Stack:** Rust, WGSL, wgpu, native Windows viewer, WASM/WebGPU contracts, Cargo tests.

**Spec:** `docs/superpowers/specs/2026-09-17-wavefront-audit-graph-design.md`

## Global Constraints

- Keep full-resolution storage strides explicit; reconstruction-only render scale remains the production policy.
- Clamp zero dimensions to one pixel and use checked arithmetic for pixel and dispatch counts.
- Keep active wavefront passes within the portable storage-buffer budget.
- Do not weaken Cornell image thresholds or publish generated artifacts.
- Preserve native-only diagnostics and compile them out of WASM paths.
- Do not modify unrelated pre-existing working-tree changes.

---

### Task 1: Make Hi-Z reduction safe for arbitrary rectangular extents

**Files:**
- Modify: `data_raw/shaders/wgsl/compute_depth_pyramid.wgsl:22-32`
- Modify: `crates/graphics/tests/test_passes.rs:237-248`
- Test: `crates/render/tests/test_frame_dimensions.rs` and `crates/graphics/tests/test_dynamic_dimensions.rs`

**Interfaces:**
- Consumes: `FrameDimensions::hiz_mip_count()` and the existing adjacent-mip bindings.
- Produces: A reduction shader that clamps each source coordinate to the input mip extent before `textureLoad`.

- [ ] **Step 1: Add a failing shader contract test**

  Extend the depth-pyramid source contract to require input-dimension clamping (for example, `textureDimensions(input_mip_texture)` and a clamped `coords` expression), and add a test case documenting a rectangular tail such as `3×5 → 1×2 → 1×1`.

- [ ] **Step 2: Run the focused graphics tests and verify the new contract fails**

  Run:

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_passes depth_pyramid
  ```

  Expected: the new source-contract assertion fails against the current unclamped shader.

- [ ] **Step 3: Implement minimal WGSL clamping**

  Read the input mip dimensions, clamp the doubled source coordinate and each of its four offsets to `input_dimensions - 1`, then perform the existing min reduction. Keep output dispatch and output coordinates unchanged.

- [ ] **Step 4: Run focused tests**

  Run:

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_passes depth_pyramid
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_render --test test_frame_dimensions
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_dynamic_dimensions
  ```

  Expected: all pass, including non-power-of-two and one-pixel mip contracts.

- [ ] **Step 5: Update `tasks/todo.md` with the completed Hi-Z fix and evidence.**

---

### Task 2: Align the first camera projection with authoritative startup dimensions

**Files:**
- Modify: `crates/plugins/viewer/src/systems/viewer_system.rs:127-189,400-420`
- Modify: `crates/plugins/viewer/tests/test_pass_order.rs` or the existing viewer test module
- Test: viewer native and WASM checks

**Interfaces:**
- Consumes: `RenderContext::frame_dimensions()`/`resolution()` and `WindowEvent::SizeChanged`.
- Produces: Camera projections initialized and updated using clamped, nonzero surface dimensions without a one-frame default-aspect mismatch.

- [ ] **Step 1: Add a failing source/integration contract**

  Assert that viewer camera creation or initial camera setup reads the authoritative render-context dimensions, and retain the existing resize-event contract for subsequent changes.

- [ ] **Step 2: Run the focused viewer test and verify failure**

  Run:

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_viewer --test test_pass_order
  ```

  Expected: the new initial-projection contract fails before implementation.

- [ ] **Step 3: Implement the smallest initialization fix**

  Add one helper in `ViewerSystem` that applies `set_projection` to all cameras using `render_context.frame_dimensions().surface`, clamped to one pixel. Call it during viewer initialization after camera creation/scene setup and keep the existing `SizeChanged` handler using the same helper so startup and resize share one path.

- [ ] **Step 4: Run viewer tests and checks**

  Run:

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_viewer --test test_pass_order
  RUSTFLAGS= cargo check --manifest-path crates/Cargo.toml -p inox_viewer --target wasm32-unknown-unknown
  ```

- [ ] **Step 5: Update `tasks/todo.md` with the initial-camera result.**

---

### Task 3: Verify end-to-end resolution and live resize behavior

**Files:**
- Modify: `crates/plugins/viewer/tests/test_pass_order.rs` only if a missing source contract is needed.
- Modify: `crates/graphics/tests/test_wavefront_integration.rs` only if a missing dimension/image assertion is needed.
- Modify: `tasks/todo.md`

**Interfaces:**
- Consumes: the corrected Hi-Z shader and unified camera/frame-dimension path.
- Produces: reproducible evidence for Cornell rendering at multiple startup sizes and after a runtime resize.

- [ ] **Step 1: Run focused native tests**

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_wavefront_bindings --test test_wavefront_integration
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_render --test test_frame_dimensions --test test_packed_scene
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_viewer --test test_pass_order
  ```

- [ ] **Step 2: Run Cornell smoke at a small non-default extent**

  ```bash
  RUSTFLAGS= timeout 120s cargo run --manifest-path crates/Cargo.toml -- -plugin inox_viewer -load_file models/cornell_box/cornell_box.scene -native_cornell_smoke -window_size 256 160
  ```

  Expected: startup dimensions report `256×160`, the native resize event reports a new extent, and `INOX_NATIVE_CORNELL_IMAGE_PASS` is emitted with `red_green_separation=true` and `bright_ceiling=true`.

- [ ] **Step 3: Run Cornell smoke at a second aspect ratio**

  ```bash
  RUSTFLAGS= timeout 120s cargo run --manifest-path crates/Cargo.toml -- -plugin inox_viewer -load_file models/cornell_box/cornell_box.scene -native_cornell_smoke -window_size 320 240
  ```

  Expected: the same image gate passes after the runtime resize and all queue/header diagnostics remain complete and overflow-free.

- [ ] **Step 4: Run the full verification suite**

  ```bash
  RUSTFLAGS= cargo fmt --manifest-path crates/Cargo.toml --all -- --check
  RUSTFLAGS= cargo test --workspace --manifest-path crates/Cargo.toml
  RUSTFLAGS= cargo clippy --workspace --all-targets --manifest-path crates/Cargo.toml --no-deps -- -D warnings
  RUSTFLAGS= cargo check --manifest-path crates/Cargo.toml -p inox_render -p inox_graphics -p inox_viewer --target wasm32-unknown-unknown
  ```

- [ ] **Step 5: Record exact results and any environmental skips in `tasks/todo.md`.**

- [ ] **Step 6: Review the final diff for scope and verify no generated artifacts were added.**

---

### Task 4: Prevent maximize-sized native queue truncation

**Files:**
- Modify: `crates/graphics/src/passes/pathtracing_common.rs:977-1005`
- Modify: `crates/graphics/tests/test_wavefront_transport.rs:183-220,500-520`
- Modify: `tasks/todo.md`

**Interfaces:**
- Consumes: negotiated discrete-native adapter policy and checked `queue_capacity_for_resolution` arithmetic.
- Produces: A discrete-native fallback capacity that covers the observed `1916×1125` maximize client extent without changing portable/WASM capacity policy.

- [ ] **Step 1: Add a failing maximize-capacity regression**

  Add a native transport test using `1916×1125` and assert the primary queue capacity is at least `2,155,500` when the negotiated limits use the discrete-native policy. Keep the portable fallback assertion unchanged.

- [ ] **Step 2: Run the focused test and verify it fails**

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_wavefront_transport native_default_dimensions_fit_queue_budget_when_adapter_limits_permit
  ```

  Expected: the new maximize assertion fails because the current native policy returns `2,097,152`.

- [ ] **Step 3: Increase only the discrete-native aggregate policy with checked constants**

  Raise the discrete-native retained queue budget to `2 GiB`, update the hard fallback ceiling so error recovery cannot silently lower that policy, and preserve the portable/WASM `512 MiB` policy. Update comments to state that this covers the tested maximize extent but remains bounded by negotiated adapter byte limits.

- [ ] **Step 4: Run focused queue and integration tests**

  ```bash
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_wavefront_transport
  RUSTFLAGS= cargo test --manifest-path crates/Cargo.toml -p inox_graphics --test test_wavefront_integration native_gpu
  ```

- [ ] **Step 5: Run the maximize-sized native Cornell smoke**

  ```bash
  RUSTFLAGS= timeout 120s cargo run --manifest-path crates/Cargo.toml -- -plugin inox_viewer -load_file models/cornell_box/cornell_box.scene -native_cornell_smoke -window_size 1916 1125
  ```

  Expected: the Cornell image gate passes and no `wavefront queue primary overflowed` diagnostic is emitted during the maximize-sized frame or subsequent resize.

- [ ] **Step 6: Record the bounded native policy and evidence in `tasks/todo.md`.**

---

### Task 5: Make Cornell smoke read the completed frame only

**Files:**
- Modify: `crates/graphics/src/passes/finalize.rs:18-25,185-193`
- Modify: `crates/graphics/src/passes/pathtracing_common.rs:896-967`
- Modify: `crates/graphics/tests/test_passes.rs`
- Modify: `tasks/todo.md`

**Interfaces:**
- Consumes: the finalize pass's ping-pong frame index.
- Produces: Native Cornell diagnostics that inspect only the frame target just rendered and emit no PPM artifacts.

- [ ] **Step 1: Add failing source-contract tests**

  Require a finalize active-target publication and require the Cornell diagnostic to use that target rather than iterating both frame targets; assert the diagnostic contains no PPM artifact writer.

- [ ] **Step 2: Implement active-target publication and artifact removal**

  Publish the completed frame target after the finalize draw and have the native diagnostic read only that target. Remove PPM file generation and path markers.

- [ ] **Step 3: Run focused graphics/viewer tests and native Cornell smoke.**
