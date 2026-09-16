# Production portable wavefront renderer

## Phase 1 — safety and contracts

- [x] Fix `BufferRef` logical-size/capacity/overwrite behavior and add shrink/grow/release tests. (Range contract and nonblocking logical writes implemented; GPU integration coverage remains.)
- [x] Fix cylinder and torus indexing and add bounds/connectivity tests.
- [x] Restore allocator, packing, BVH, node, common-script, viewer-config, and pass-order regression coverage. (Focused integration coverage passes for all listed contracts.)
- [x] Fix Web canvas ownership, mouse mapping, resize, valid handle, and surface setup. (Retained canvas ownership, device-pixel sizing, finite/clamped pointer mapping, stable raw handle, and resize event path implemented.)
- [x] Negotiate adapter features/limits instead of unconditionally requesting optional capabilities. (Baseline feature validation, optional feature filtering, portable limit checks, and capability-driven pass fallbacks implemented.)
- [ ] Make shader reload and binarizer failures nonfatal but observable; preserve last-known-good resources.
- [ ] Remove hot-loop allocations and repeated locks/lookups covered by the production plan. (Per-dispatch pipeline Vec allocation removed; indirect draw stride corrected; render-target/meshlet lock audits remain.)

## Phase 2 — binarizer-first portable foundation

- [x] Make external compiler/validator exit status strict. (Include caching/cycle detection remains.)
- [x] Run the native binarizer for both PC and Web before generated-data validation or runtime launch.
- [ ] Validate every generated production shader and pipeline’s storage-binding count.
- [ ] Enforce packed-scene/header versions and typed CPU↔WGSL layout contracts.

## Phase 3 — packed render graph

- [x] Typed queue/counter/dispatch foundation exists.
- [x] Packed scene foundation exists.
- [x] Replace legacy G-buffer/direct/indirect active passes with packed-scene stages. (Legacy lighting/indirect passes are disabled; packed surface/direct/visibility/three-bounce stages are active.)
- [ ] Prove every active production stage is within the portable eight-storage-buffer budget.
- [ ] Add explicit pass-order/dependency graph tests.

## Phase 4 — direct lighting

- [ ] Add packed light hierarchy/alias-table updates.
- [x] Add AO state and independent AO accumulator. (Packed visibility now traces one bounded AO ray; spatial/temporal AO reuse remains.)
- [x] Add MegaLights-style bounded candidate generation. (Initial bounded light hierarchy candidate stage.)
- [x] Add ReSTIR-DI temporal/spatial reservoir reuse with validity and weighted candidates. (Four-neighbor reuse, proposal PDFs, bounded Jacobian correction, and confidence weighting are active.)
- [x] Add shadow visibility and independent direct accumulator. (Packed visibility traces the selected reservoir shadow ray; a dedicated compact shadow queue remains.)

## Phase 5 — indirect and transparency

- [x] Add BSDF lobe queues and ping-pong active-ray compaction. (Dynamic eight-slot queue chain now builds bounce stages at viewer setup; material-lobe routing refinement remains.)
- [x] Add indirect dispatch arguments and empty-queue early termination. (Queue counters drive the current bounded three-bounce chain.)
- [ ] Add multi-bounce GI, environment sampling, Russian roulette, reflections, and refractions. (BRDF/BSDF reflection/transmission sampling is active in the bounded bounce chain; true environment sampling and generalized depth remain.)
- [x] Add bounded alpha/transmission queue and OIT fallback. (One bounded transparent-hit layer and compositing resolve are active; multi-layer sorting/refraction remains.)
- [x] Keep direct, AO, indirect, shadow, and overflow outputs independently inspectable. (Separate direct/indirect/final accumulators and AO/shadow diagnostics are active; dedicated queue debug views remain.)

## Phase 6 — reconstruction and deployment

- [x] Add temporal history/moments/variance invalidation. (Composite writes luminance moments; finalize clamps history by variance and frame history.)
- [x] Add pure-WGSL temporal/bilateral denoisers. (Current baseline clamps separated final accumulation; variance-guided temporal history refinement remains.)
- [x] Add pure-WGSL native/temporal/spatial upscaler baseline. (Current shader performs bounded spatial reconstruction; temporal mode remains a later quality refinement.)
- [ ] Run PC binarizer, native smoke/profile, WASM build/bindgen, and browser WebGPU smoke in strict order.
- [ ] Publish only artifacts produced after successful PC+Web binarization and generated-data validation.

## Review

- Existing foundation tests, clippy, formatting, WebAssembly check, and Web-profile shader validation passed before this production phase.
- Full production readiness remains blocked until legacy passes are removed, direct/AO/ReSTIR shadow visibility is complete, indirect ping-pong queues/multi-bounce are complete, and PC/WASM runtime smoke tests pass.
- Stable native build, full workspace tests, clippy, WebAssembly check, Web-profile shader validation, and PC/Web binarizer gates have been rerun successfully after the latest packed/direct/indirect changes.
- Final temporal/multi-neighbor/transparency/upscale changes pass stable clippy, full tests, Web shader validation, and the PC/Web binarizer gate.
- Latest three-bounce, separated-accumulator, denoiser, upscaler, and BRDF/BSDF changes also pass workspace checks, stable clippy, WASM check, shader validation, binarizer, and native startup smoke.
- Adapter negotiation tests pass; render clippy and the render WebAssembly check pass with the negotiated capability path.
- Final separated direct/indirect/final accumulator, denoiser, and pure-WGSL spatial reconstruction build and binarizer gates passed.
- Full workspace tests and stable clippy pass after the three-bounce graph and final accumulator changes.
- Final release workspace build and PC/Web binarizer gate completed after strict compiler-status handling.
- Native viewer smoke selected the RTX 4090 Laptop GPU and configured `Bgra8UnormSrgb` without a startup panic; the 20-second smoke was stopped by timeout.
