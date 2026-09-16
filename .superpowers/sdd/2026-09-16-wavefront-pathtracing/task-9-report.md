# Task 9 report

## Delivered

- Added max path depth, minimum roulette depth, and environment texture index to the CPU/WGSL constant-data contract.
- Made dispatch and transport passes use queue header counts for empty-queue early termination and reset destination headers before append.
- Added environment texture sampling with an analytic sky fallback for invalid/unconfigured textures.
- Replaced fixed throughput attenuation with explicit-lobe diffuse/glossy/specular/transmission sampling, PDF-consistent throughput, Fresnel selection, and total-internal-reflection reflection fallback.
- Added finite guards for material values, directions, BSDF/PDF terms, throughput, roulette, and spawned-ray origin, plus finite radiance sanitization at composite.
- Corrected generated bounce dispatch handoffs, moved traversal hit-header reset before all early returns, clamped roulette depth when max depth changes, and bounded environment texture indices while preserving valid black textures.
- Added focused contracts for dispatch chaining, lobe PDFs, finite guards, environment bounds, queue reset, depth/roulette, reflection/refraction/TIR, and accumulator separation.
- Follow-up review fix: total internal reflection now executes before lobe selection and collapses transmission into a unit-probability delta reflection, preventing diffuse fallback and estimator bias.
- Follow-up energy fix: TIR throughput now bypasses the cosine-weighted BSDF estimator and preserves the finite incoming throughput for oblique reflection; a focused regression contract covers this path.
- Strengthened the TIR regression to structurally extract the TIR and non-TIR branches, verify direct throughput assignment occurs before lobe selection, and verify the cosine-weighted estimator remains isolated to non-TIR paths.
- Removed CPU queue-count activation gates from dispatch, traversal, next-bounce, and shade; static resource availability now schedules the pass while GPU-written indirect arguments perform empty-queue termination.
- Corrected numeric material-index decoding to match the surface pass's `f32(material_id)` encoding, and added a cross-shader contract.
- Unified all wavefront queue capacities on the actual full-resolution path (`render_scale = 1.0`) with a documented finite 2,097,152-entry cap; the 2,073,600-pixel default is fully represented and larger workloads set the GPU overflow marker.
- Made composite, denoise, and upscale use full-resolution producer/consumer strides, and made CPU denoise dispatch cover the same full dimensions; render scale no longer changes storage indexing and remains a reconstruction-quality control.
- Bounded queue header capacities through checked `u32` conversion and retained overflow signaling for over-cap workloads; transparency now shares the same 2,097,152-entry finite budget as every wavefront queue.
- Added focused contracts for static activation, default-resolution and over-cap capacity, transparency capacity parity, checked header conversion, material encoding, and non-native render-scale dispatch/stride behavior.

## Verification

- `cargo test -p inox_graphics --manifest-path ./crates/Cargo.toml`
- `cargo test -p inox_render --manifest-path ./crates/Cargo.toml`
- `cargo test -p inox_binarizer --manifest-path ./crates/Cargo.toml`
- `cargo clippy -p inox_graphics -p inox_render -p inox_binarizer --manifest-path ./crates/Cargo.toml --no-deps -- -D warnings`
- `cargo fmt --manifest-path ./crates/Cargo.toml --all -- --check`
- Native checks for render, graphics, and binarizer
- WebAssembly checks for render and graphics

All listed checks passed. A full native viewer smoke and browser WebGPU smoke remain outside this focused task run.
