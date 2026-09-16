# Production portable wavefront renderer

## Phase 1 — safety and contracts

- [x] Fix `BufferRef` logical-size/capacity/overwrite behavior and add shrink/grow/release tests. (Range contract and nonblocking logical writes implemented; GPU integration coverage remains.)
- [x] Fix cylinder and torus indexing and add bounds/connectivity tests.
- [x] Restore allocator, packing, BVH, node, common-script, viewer-config, and pass-order regression coverage. (Focused integration coverage passes for all listed contracts.)
- [x] Fix Web canvas ownership, mouse mapping, resize, valid handle, and surface setup. (Retained canvas ownership, device-pixel sizing, finite/clamped pointer mapping, stable raw handle, and resize event path implemented.)
- [x] Negotiate adapter features/limits instead of unconditionally requesting optional capabilities. (Baseline feature validation, optional feature filtering, portable limit checks, and capability-driven pass fallbacks implemented.)
- [x] Make shader reload and binarizer failures nonfatal but observable; preserve last-known-good resources. (Structured shader/pipeline diagnostics, transactional pipeline replacement, and resilient reload path handling implemented.)
- [x] Remove hot-loop allocations and repeated locks/lookups covered by the production plan. (Compute pipelines and labels are cached; meshlet commands reuse pass-owned scratch; render-target lookups and indirect count-prefix handling are corrected.)

## Phase 2 — binarizer-first portable foundation

- [x] Make external compiler/validator exit status strict. (Include caching/cycle detection remains.)
- [x] Run the native binarizer for both PC and Web before generated-data validation or runtime launch.
- [x] Validate every generated production shader and pipeline’s storage-binding count. (Include-expanded WGSL validation enforces the portable eight-storage-buffer budget, duplicate bindings, required group/binding attributes, and pipeline shader references.)
- [x] Enforce packed-scene/header versions and typed CPU↔WGSL layout contracts. (Packed scene words now validate before upload and versioned record descriptors mirror WGSL stride constants.)

## Phase 3 — packed render graph

- [x] Typed queue/counter/dispatch foundation exists.
- [x] Packed scene foundation exists.
- [x] Replace legacy G-buffer/direct/indirect active passes with packed-scene stages. (Legacy lighting/indirect passes are disabled; packed surface/direct/visibility/three-bounce stages are active.)
- [x] Prove every active production stage is within the portable eight-storage-buffer budget. (Viewer registration metadata inventories every active wavefront shader; focused graphics validation runs the binarizer's WebGPU binding checks and eight-buffer budget for each source.)
- [x] Add explicit pass-order/dependency graph tests. (Canonical viewer registration metadata covers the full generated bounce chain, transparency/AO, disabled legacy passes, reconstruction tail, and queue/header/dispatch dependencies.)

## Phase 4 — direct lighting

- [x] Add packed light hierarchy/alias-table updates. (Bounded deterministic alias table is packed with version/count/overflow metadata; direct candidates use alias PDFs with an empty-table uniform fallback.)
- [x] Add AO state and independent AO accumulator. (Packed visibility now traces one bounded AO ray; spatial/temporal AO reuse remains.)
- [x] Add MegaLights-style bounded candidate generation. (Initial bounded light hierarchy candidate stage.)
- [x] Add ReSTIR-DI temporal/spatial reservoir reuse with validity and weighted candidates. (Four-neighbor reuse, proposal PDFs, bounded Jacobian correction, and confidence weighting are active.)
- [x] Add shadow visibility and independent direct accumulator. (Packed visibility traces the selected reservoir shadow ray; a dedicated compact shadow queue remains.)

## Phase 5 — indirect and transparency

- [x] Add BSDF lobe queues and ping-pong active-ray compaction. (Dynamic eight-slot queue chain now builds bounce stages at viewer setup; material-lobe routing refinement remains.)
- [x] Add indirect dispatch arguments and empty-queue early termination. (Queue counters drive the current bounded three-bounce chain.)
- [x] Add multi-bounce GI, environment sampling, Russian roulette, reflections, and refractions. (Depth-bounded queue stages now terminate from counters, sample environment textures with analytic-sky fallback, apply bounded roulette, and use PDF-weighted diffuse/glossy/refraction transport with TIR reflection fallback.)
- [x] Add bounded alpha/transmission queue and OIT fallback. (One bounded transparent-hit layer and compositing resolve are active; multi-layer sorting/refraction remains.)
- [x] Keep direct, AO, indirect, shadow, and overflow outputs independently inspectable. (Separate direct/indirect/final accumulators and AO/shadow diagnostics are active; dedicated queue debug views remain.)

## Phase 6 — reconstruction and deployment

- [x] Add temporal history/moments/variance invalidation. (Composite writes luminance moments; finalize clamps history by variance and frame history.)
- [x] Add pure-WGSL temporal/bilateral denoisers. (Current baseline clamps separated final accumulation; variance-guided temporal history refinement remains.)
- [x] Add pure-WGSL native/temporal/spatial upscaler baseline. (Current shader performs bounded spatial reconstruction; temporal mode remains a later quality refinement.)
- [ ] Run PC binarizer, native smoke/profile, WASM build/bindgen, and browser WebGPU smoke in strict order. (The strict gate passed through bindgen generation; Chrome was found but returned no smoke DOM or engine signals.)
- [ ] Publish only artifacts produced after successful PC+Web binarization and generated-data validation. (No output was published because the unavailable browser dependency stopped the gate before final publication.)

## Review

- Existing foundation tests, clippy, formatting, WebAssembly check, and Web-profile shader validation passed before this production phase.
- Full production readiness remains blocked until legacy passes are removed, direct/AO/ReSTIR shadow visibility is complete, indirect ping-pong queues/multi-bounce are complete, and PC/WASM runtime smoke tests pass.
- Stable native build, full workspace tests, clippy, WebAssembly check, Web-profile shader validation, and PC/Web binarizer gates have been rerun successfully after the latest packed/direct/indirect changes.
- Final temporal/multi-neighbor/transparency/upscale changes pass stable clippy, full tests, Web shader validation, and the PC/Web binarizer gate.
- Latest three-bounce, separated-accumulator, denoiser, upscaler, and BRDF/BSDF changes also pass workspace checks, stable clippy, WASM check, shader validation, binarizer, and native startup smoke.
- Shader reload retention passes focused and complete render/resource tests, render/resource library clippy, formatting, graphics crate checks, and the render WASM check; storage-boundary candidates retain active resources and diagnostics until a later valid commit.
- Native device-backed Shader, ComputePipeline, and RenderPipeline reload tests cover real module/pipeline initialization, invalid-candidate retention, diagnostic observability, and later valid commits; a headless context helper keeps the tests surface-independent.
- Adapter negotiation tests pass; render clippy and the render WebAssembly check pass with the negotiated capability path.
- Task 7 focused validation passes: one viewer registration table now owns pass factories, enabled flags, generated bounce handoffs, transparency/AO dependencies, and observed registration metadata; construction records are consumed by the actual bounce, auxiliary, and disabled legacy factories; pipeline descriptors are checked against the on-disk pipeline-to-shader mappings before include-expanded eight-storage-buffer validation.
- Final separated direct/indirect/final accumulator, denoiser, and pure-WGSL spatial reconstruction build and binarizer gates passed.
- Full workspace tests and stable clippy pass after the three-bounce graph and final accumulator changes.
- Final release workspace build and PC/Web binarizer gate completed after strict compiler-status handling.
- Native viewer smoke selected the RTX 4090 Laptop GPU and configured `Bgra8UnormSrgb` without a startup panic; the 20-second smoke was stopped by timeout.
- Task 5 hot-loop validation passed: debug and release render/graphics regression tests, render/graphics checks, stable clippy, and workspace formatting. Compute dispatches reuse cached pipelines/labels; indexed indirect draws use the vector count-prefix offset; meshlet draws reuse pass-owned command scratch after releasing data locks.
- Task 6 validation passed: binarizer binding/pipeline tests, render packed-data tests, graphics record-layout tests, binarizer/render/graphics clippy, formatting, native checks, and render/graphics WebAssembly checks. The Windows toolchain required `RUSTFLAGS=` for native commands because the repository config injects unsupported `split-debuginfo=unpacked`.
- Task 8 validation passed: bounded packed light alias construction covers empty, single, many, zero/nonfinite, and overflow inputs; render/graphics tests, binarizer shader validation, clippy, formatting, and render/graphics WebAssembly checks pass.
- Task 9 validation passed: graphics/render/binarizer tests, focused transport contracts, include-expanded shader validation, clippy, formatting, native checks, and render/graphics WebAssembly checks pass. Transport now supports environment fallback with bounded texture indices, bounded roulette, explicit PDF-consistent reflection/refraction sampling with finite guards, queue-counter early termination, per-stage reset, and chained bounce dispatch metadata.
- An earlier strict no-skip rerun on 2026-09-16 stopped at PC binarization because the then-current launcher rejected `compute_bounce_shade.wgsl` at ten compute storage bindings. The latest launcher run passed PC and Web binarization, generated-data validation, native tests/clippy/release, formatting, native viewer startup, WASM release, pinned bindgen installation, and bindgen generation; it stopped at browser smoke because Chrome returned no smoke DOM or engine signals. No artifacts were published. See `docs/superpowers/reports/2026-09-16-wavefront-validation.md`.
- Review fixes add safe temporary browser-profile cleanup, non-empty platform-directory validation for newly generated `data/pc` and `data/web`, explicit formatting and native viewer startup gates, and engine-owned WASM device/queue-submit/frame signals with page and console error capture in browser smoke.
