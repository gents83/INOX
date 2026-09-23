# Production portable wavefront renderer

## Final review blockers

- [x] Keep transparency producer indexing at full-resolution G-buffer dimensions; render scale remains reconstruction-only.
- [x] Validate material IDs before every packed material access and reject `INVALID_INDEX`/nonfinite/out-of-range values.
- [x] Use explicit entering/exiting `eta_i / eta_t` transport ratios with CPU and WGSL contracts.
- [x] Expose bounded queue overflow through GPU header `COPY_SRC` staging readback and explicit post-submit diagnostics.

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
- [x] Run PC binarizer, native smoke/profile, WASM build/bindgen, and browser WebGPU smoke in strict order. (PC/Web binarization, native checks/smoke, WASM build/bindgen, and all non-browser markers passed; browser WebGPU remains an explicit unavailable dependency because headless Chrome returned no smoke DOM or engine signals.)
- [ ] Publish only artifacts produced after successful PC+Web binarization, generated-data validation, and browser WebGPU smoke. (No output was published because the browser dependency was unavailable.)

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
- Task 9 validation passed: graphics/render/binarizer tests, focused transport contracts, include-expanded shader validation, clippy, formatting, native checks, and render/graphics WebAssembly checks pass. Transport now supports environment fallback with bounded texture indices, bounded roulette, explicit PDF-consistent reflection/refraction sampling with finite guards, GPU-controlled empty-queue termination without CPU count gates, numeric material-ID decoding, a shared finite 2,097,152-entry queue budget covering the 2,073,600-pixel default (including transparency) with overflow signaling beyond it, full-resolution CPU/GPU composite-denoise-upscale strides, checked `u32` header capacities, per-stage reset, and chained bounce dispatch metadata.
- An earlier strict no-skip rerun on 2026-09-16 stopped at PC binarization because the then-current launcher rejected `compute_bounce_shade.wgsl` at ten compute storage bindings. The latest launcher run passed PC and Web binarization, generated-data validation, native tests/clippy/release, formatting, native viewer startup, WASM release, pinned bindgen installation, and bindgen generation; it stopped at browser smoke because Chrome returned no smoke DOM or engine signals. No artifacts were published. See `docs/superpowers/reports/2026-09-16-wavefront-validation.md`.
- Review fixes add safe temporary browser-profile cleanup, Program Files(x86)/LOCALAPPDATA Chrome and Edge discovery, non-empty platform-directory validation for newly generated `data/pc` and `data/web`, explicit formatting and engine-owned native viewer startup/format marker validation with preserved failure logs, and engine-owned WASM device/queue-submit/frame signals with page and console error capture in browser smoke.
- Native queue diagnostics validation passes: the viewer registers a post-submit callback only on native targets, staging readback handles mapping errors and poll timeouts without indefinite waits or panics, callback snapshots permit re-registration, and WebGPU/WASM explicitly skips synchronous diagnostics.
- Final shader review fixes pass focused and full workspace validation: direct-candidate, visibility, and AO-resolve use full-resolution strides, while shade rejects `INVALID_INDEX`, negative, and out-of-range material IDs before packed access; graphics/render/binarizer tests, clippy, formatting, native checks, and render/graphics/binarizer WASM checks all pass.
- Queue-reset follow-up uses an immediate `RenderContext::update_buffer` upload through `BufferRef::bind` with storage/COPY_DST/COPY_SRC usage in primary, traversal, bounce, and transparency passes; native overflow readback remains observable with transition-deduplicated logging, and temporary WebGPU diagnostics are silent.
- Final queue-reset regression validation passes workspace tests, workspace clippy, formatting, native and WASM checks, and a 35-second native viewer smoke that selected the RTX 4090 Laptop GPU, emitted `INOX_NATIVE_VIEWER_STARTUP_OK`, and reached the running app before timeout.

## Current freeze investigation

- [x] Confirm the queue lock blocker and UID/resource identity root cause.
- [x] Add a regression contract for unique wavefront resource IDs.
- [x] Apply one minimal root-cause fix; no temporary freeze diagnostics were present in the worktree.
- [x] Run focused, connector/graphics/render, and debug viewer smoke validation.
- [x] Review results and commit with the requested attribution.

### Freeze investigation review

`generate_static_uid_from_string` only uses the first 16 input bytes. Queue/header names sharing a `WAVEFRONT_*` prefix therefore returned the same UID. `GlobalBuffers::vector_with_id` intentionally reuses the existing `Arc` without a type check, so the transparent queue and header aliases referred to one `RwLock`; the apparent queue write deadlock was type-alias corruption, not a retained guard, reader, pass-list lock, or GPU resource initialization wait. All wavefront static resource names now have distinct short prefixes, and a pairwise uniqueness regression contract covers every static wavefront resource ID.

## Production shader smoke review

- Added a causal native headless WebGPU smoke graph covering production surface, primary, dispatch, traversal, hit-dispatch, shade, composite, and finalize shader stages with staged GPU readback and exact packed-scene/texture assertions.
- The RTX headless adapter now negotiates its actual limits and the causal smoke passes: `7 passed`; transport contracts: `21 passed`.
- The smoke explicitly reports adapter capability skips for unavailable native texture-array paths; no synthetic fallback is used.

## Task 4 — runtime frame dimensions

- [x] Add checked `FrameDimensions`/`Extent2d` derivation with one-pixel clamping, explicit render-scale policy, full-resolution storage stride, and rounded dispatch dimensions.
- [x] Propagate one resize contract through surface configuration, render targets, pass dimensions, constant data, queue capacities, G-buffer/debug allocations, and initial viewer setup.
- [x] Add render and graphics regression tests for zero, non-default, scaled, and resize-derived dimensions.
- [x] Run focused native tests, native headless wavefront integration, formatting, focused clippy, native checks, and render/graphics/viewer WASM checks.

Task 4 results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/task-4-report.md`.

Review-fix follow-up: resize ordering now applies before surface acquisition and ConstantData use; Hi-Z mip counts are derived from the current extent for creation and recreation; `DepthPyramidPass::resize` rebuilds dimensions and mip bindings. Focused native tests, native causal smoke, clippy, formatting, native checks, and render/graphics/viewer WASM checks pass.

Re-review follow-up: render-target resize advances a binding generation so all texture-backed BindingData rebuilds bind groups against replacement views; Hi-Z tail extents clamp to one pixel before dispatch sizing. Full graphics/viewer native tests, focused regressions, clippy, formatting, native checks, and WASM checks pass.

Final Task 4 re-review: multi-mip/Hi-Z target role is preserved independently of the current mip count, so shrink-to-1x1 followed by growth restores the full mip chain. Regression and full native/WASM/clippy/fmt/check validation pass.

Final initial-dimension fix: viewer startup derives clamped `FrameDimensions` before surface setup and all size-dependent allocations, including zero-size initial windows. Viewer regression plus full native/WASM/clippy/fmt/check validation pass.

## Task 6 — capability-aware queues and runtime bounces

- [x] Add checked, dimension/record-size/capability-aware queue capacity calculation with bounded fallback and adapter byte limits.
- [x] Use a target-aware aggregate host+GPU queue budget: explicitly classified discrete native adapters get a 1536 MiB policy ceiling for full-resolution primary/hit/transparency capacity when adapter byte limits permit, while integrated/CPU/unknown native, WASM, and portable profiles retain the conservative 512 MiB bound; transient upload staging is excluded and no physical-VRAM guarantee is implied.
- [x] Preserve the fixed eight-stage graph while clamping runtime zero/one/max/out-of-range bounce controls through `ConstantData`.
- [x] Reset queue headers and indirect arguments before append/dispatch, including enabled inactive stages; preserve zero dispatch arguments for empty queues.
- [x] Expose requested/actual capacity, overflow, active bounce count, and dropped records through queue headers and native diagnostics.
- [x] Add capacity, dropped-record, fixed-graph, inactive-reset, dispatch-clamp, native smoke, and six-word header fixture tests.
- [x] Run focused and workspace native tests, native release build, render/viewer/binarizer tests, formatting, clippy, and render/graphics/viewer/binarizer WASM debug/release checks.

Task 6 results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/task-6-report.md`.

## Task 7 — packed scene, BVH, BLAS/TLAS, and meshlet traversal

- [x] Add checked BVH construction/validation for empty, one-node, degenerate, non-finite, disconnected, and invalid-reference cases.
- [x] Add checked linearized-BVH construction and escape-link validation.
- [x] Add checked packed-scene offsets, stride/range validation, overlap checks, and checked meshlet LOD range packing without changing packed record formats.
- [x] Validate instance references and compute conservative all-corner transformed TLAS bounds.
- [x] Add bounded WGSL traversal, packed-range checks, finite ray/bounds guards, runtime-array index guards, and iteration/stack exits; preserve portable workgroup sizes.
- [x] Verify meshlet ranges and triangle index decoding contracts against CPU packed data.
- [x] Run BVH/render/graphics/binarizer tests, native causal smoke, formatting, clippy, and affected-crate WASM checks.
- [x] Centralize packed-scene validation for all seven active wavefront WGSL stages; validate header/table ranges and overlap before scene reads, guard dependent/alias/light accesses, reject malformed visibility before ID subtraction, and add source/property contracts for malformed ranges and arithmetic wraparound.

Task 7 results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/task-7-report.md`.

### Task 7 coordinator review follow-up

- [x] Preserve a consistent ray parameter between inverse-transformed BLAS AABB tests and world-space triangle intersections under non-uniform scaling.
- [x] Negotiate downlevel storage-buffer limits without rejecting WebGL2 adapters; expose an explicit full-wavefront layout capability gate.
- [x] Evaluate actual prefix/suffix SAH partitions with a zero-extent fallback.
- [x] Use stack-based BVH reverse traversal instead of front-removing/inserting vectors.
- [x] Gate production wavefront graph and pass construction on `RenderCapabilities::wavefront_layout_supported`; select the validated fallback graph when unsupported.
- [x] Add viewer integration coverage proving unsupported adapters disable wavefront nodes and do not construct full-layout pipelines.
- [x] Rerun viewer/render native and WASM checks, formatting, and clippy.
- [x] Review packed-scene arithmetic: subtraction-based range/index helpers now precede meshlet, index, vertex, and relative-meshlet offset arithmetic in surface/traversal/visibility; visibility decoding rejects only miss encoding zero after the reserved miss sentinel and validates the decoded table ID, preserving instances at and above 254.
- [x] Rerun full graphics/binarizer tests with native causal smoke, clippy, WASM checks, and formatting.

## Task 8 — transport quality and pass scheduling

- [x] Add source/property contracts covering diffuse, glossy, reflection, refraction, TIR, roulette, environment misses, emissive hits, zero-PDF termination, finite guards, and full-resolution denoiser strides.
- [x] Correct confirmed transport bias: dielectric transmission now applies the eta-squared radiance factor and direct reservoirs normalize the selected contribution by proposal count/selected weight.
- [x] Invalidate temporal accumulator/reservoir history through a shared epoch on camera, resize, and material changes; reject stale reservoir epochs.
- [x] Clear the indirect accumulator before each frame and sanitize nonfinite shade/environment/composite values, including infinities.
- [x] Keep queue/denoise/reconstruction stages as compute passes, retain render-only display/final composition, and add the missing Upscale-to-Finalize graph resource handoff.
- [x] Avoid CPU activation gates based on GPU-written queue counts; preserve indirect-dispatch scheduling and direct/AO/indirect/final accumulator separation.
- [x] Run focused graphics/viewer tests, render/binarizer tests, native causal smoke, formatting, clippy, and affected-crate WASM checks.
- [x] Review follow-up: saturate reservoir candidates and reject nonfinite weights, PDFs, confidence, and final estimates.
- [x] Review follow-up: invalidate history for light lifecycle changes and preserve loaded light data through create/update paths.
- [x] Review follow-up: pack only occupied lights/aliases, conditionally decrement light counts, and use explicit two-sided emissive behavior.
- [x] Review follow-up: bind real temporal denoiser history/moments state and keep epoch invalidation plus an explicit per-frame indirect clear regression.
- [x] Review follow-up verification: workspace tests/clippy, native causal smoke, binarizer, viewer, WASM, and formatting pass.
- [x] Final review follow-up: honor active state for create/change/load; test inactive create, activation, deactivation, and reload without count duplication.

Task 8 results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/task-8-report.md`.

## Task 9 — wavefront audit graph evidence

- [x] Add native post-submit queue diagnostics, graph-order assertions, frame-dimension assertions, finite/nonzero output checks, and completed-frame evidence.
- [x] Add WASM/browser markers for canvas ownership, configured dimensions, device/queue setup, graph order, completed frame, and zero console/page errors; require explicit asynchronous GPU validation before queue-header or finite/nonzero-output markers can be published, and preserve explicit WebGPU skips.
- [x] Use production WGSL preprocessing for causal smoke shader loading and storage-binding validation; add browser marker source-contract coverage.
- [x] Run native tests, causal GPU smoke, affected-crate WASM checks, PC/Web binarization, generated-data validation, formatting, and clippy.
- [ ] Complete browser WebGPU smoke and publish output artifacts. Chrome was available but returned no smoke DOM or engine signals, so this remains an explicit unavailable dependency and is not counted as a pass.

Task 9 results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/task-9-report.md`.

## Task 10 — strict verification and evidence

- [x] Formatting check: `RUSTFLAGS= cargo fmt --manifest-path ./crates/Cargo.toml --all -- --check` passed with exit 0 and no output.
- [x] Full workspace tests: `RUSTFLAGS= cargo test --workspace --manifest-path ./crates/Cargo.toml` passed with exit 0; all reported tests passed.
- [x] Workspace clippy: `RUSTFLAGS= cargo clippy --workspace --all-targets --manifest-path ./crates/Cargo.toml --no-deps -- -D warnings` passed with exit 0 and no warnings.
- [x] Native release build: `RUSTFLAGS= cargo build --manifest-path ./crates/Cargo.toml --release` passed with exit 0.
- [ ] Native viewer full smoke: after ping-pong queue reuse and aggregate budgeting, the rebuilt viewer reaches startup, dimensions, graph, complete queue-header, and no-overflow markers without the previous out-of-memory failure; the bounded startup run ends by intentional timeout, so no full-frame pass is claimed.
- [x] PC/Web binarizers and generated-data validation passed. PC and Web launcher invocations exited 0; each `cargo test -p inox_binarizer --tests` run passed mesh (1), shader bindings (5), and shader compiler (12) tests.
- [x] WASM check and release build passed for `wasm32-unknown-unknown` with the configured encoded flags.
- [x] Pinned `wasm-bindgen-cli v0.2.128` installation and temporary web binding generation passed; generated output was removed and not published.
- [ ] Browser WebGPU smoke: the fixed pipeline capture now returns DOM diagnostics. Normal headless Chrome and the `--disable-gpu` SwiftShader fallback both leave `navigator.gpu.requestAdapter()` unresolved; the smoke reports an explicit error-free `adapter-timeout` capability skip, not a pass, and no browser artifacts are published.
- [ ] Existing strict pipeline: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File 'D:/PROJECTS/gents83/INOX/scripts/validate_wavefront_pipeline.ps1' -NativeViewerSmokeSeconds 10` exceeded the 600-second tool timeout during workspace recompilation and was stopped before native/browser stages; no output was published. The exact result and final status audit are recorded in the Task 10 report.

Task 10 exact commands and observed results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/task-10-report.md`.

## Final whole-branch finding 2 — surface/primary graph handoff

- [x] Model the concrete packed G-buffer IDs written by `ComputeWavefrontSurfacePass` and read by `ComputeWavefrontPrimaryPass`.
- [x] Include the shared material/world inputs consumed by transparency and direct-candidate passes.
- [x] Add explicit Surface → Primary resource dependencies and regression coverage for malformed or missing handoffs.
- [x] Run the requested viewer/graphics/render native and WASM tests, formatting, and clippy.

The implementation uses the type-derived IDs already used by the concrete `GPUVector` bindings, preserving supported and unsupported capability graph behavior and declaration order. Focused viewer graph tests (9), graphics integration tests (8, including native causal smoke), render tests (48), viewer/graphics/render WASM checks, formatting, and targeted clippy all pass. The first combined native invocation hit a Windows executable-lock race; the affected graphics integration and render suites were rerun serially successfully.

## Final review finding — invalid surface scene state

- [x] Track packed-scene validity and binding initialization in `ComputeWavefrontSurfacePass`.
- [x] Separate texture/resource readiness from scene validity so initialization retries after transient invalid scene data.
- [x] Invalidate stale CPU data and binding metadata when scene validation fails; block update dispatch until valid initialization.
- [x] Add source regression coverage for invalid-scene initialization followed by update and transient-invalid-then-valid retry.
- [x] Rerun viewer/graphics/render native tests, WASM checks, formatting, and clippy.

## Native wavefront queue OOM follow-up — 2026-09-18

- [x] Replace eight generated ray payload allocations with two ping-pong bounce queues plus the separate primary queue.
- [x] Keep all eight stage headers and dispatch argument resources distinct, with queue-ID-aware graph producer dependencies.
- [x] Add queue alternation, no same-stage read/write alias, UID uniqueness, and default memory-budget regressions.
- [x] Reset the shared hit header immediately before every active traversal dispatch and test primary/bounce command ordering.
- [x] Derive an aggregate host+GPU fallback across ray, hit, and transparency queues with overflow diagnostics preserved; retain the conservative 512 MiB portable bound and provide a 1536 MiB discrete-native policy ceiling for 1900x1037 without overflow when adapter byte limits permit. Transient upload staging is excluded; this is not a physical-VRAM guarantee.
- [x] Run full graphics/render/viewer/binarizer tests, native causal and small-extent allocation/readback smoke, workspace clippy, formatting, WASM checks, and rebuilt native viewer startup smoke.
- [x] Confirm the rebuilt native viewer reaches startup, complete queue-header, and no-overflow markers without the prior native OOM.

Results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/runtime-oom-report.md`.

### Queue host-mirror budget review — 2026-09-18

- [x] Count the CPU `GPUVector` mirrors alongside GPU queue records in aggregate byte estimates.
- [x] Add focused host+GPU byte-accounting and native 1900x1037 no-overflow regressions.
- [x] Re-run focused graphics transport/pass tests, formatting, clippy, and the graphics WASM check.
- [x] Add native-independent queue marker decision tests for clean, overflow/dropped, and partial header readback; retain separate complete and no-overflow runtime markers.

## Browser runtime limitation follow-up — 2026-09-18

- [x] Bind the WASM panic hook to the explicit `console.error` namespace and add a source contract.
- [x] Propagate asynchronous adapter/device failures to browser smoke diagnostics; only mark device readiness after `request_device` succeeds.
- [x] Add render/platform smoke-marker contracts and failure-marker handling.
- [x] Fix PowerShell native stdout and Chrome stderr capture so browser DOM diagnostics are not aborted by host-shell behavior.
- [x] Run focused tests, formatting, WASM check/release build, bindgen, and the serial strict PowerShell pipeline.
- [ ] Resolve the external Chrome headless WebGPU initialization hang and publish browser output artifacts. Direct minimal probes reproduce the unresolved `navigator.gpu.requestAdapter()` promise in Chrome itself, including fallback mode; the smoke now classifies this exact no-error condition as an explicit capability/resource skip and never publishes artifacts.

Results are recorded in `.superpowers/sdd/2026-09-17-wavefront-audit-graph/browser-runtime-report.md`.

## Cornell image correctness smoke — 2026-09-18

- [x] Add a native-only, explicitly enabled Cornell image smoke mode and scene-load detection.
- [x] Add bounded post-submit readback of the final display target with finite, coverage, tonal, and Cornell spatial/color checks.
- [x] Add deterministic Cornell-like fixture/integration coverage for the diagnostic and pass-chain assumptions.
- [x] Trace and fix the confirmed direct-candidate material binding/modulation failure found by native execution; retain the remaining image failure as an explicit diagnostic.
- [x] Run native graphics tests and viewer `-load_file models/cornell_box/cornell_box.scene` smoke, render/binarizer/WASM checks, fmt/clippy.
- [x] Write exact commands/results/limitations to `.superpowers/sdd/2026-09-17-wavefront-audit-graph/cornell-smoke-report.md`.

### Review

The opt-in callback and deterministic fixture pass. Native execution reaches GPU readback and reports finite, fully covered, non-black/non-white output, but the real scene still lacks the required green-wall and ceiling-light signatures (`red_band=2`, `green_band=0`, `bright_ceiling=false`). The direct-candidate pass now has an explicit material G-buffer binding and packed base-color modulation; this fixed the pipeline binding validation error, but the remaining transport/material result is not claimed as passing.

## Task 1 — Hi-Z rectangular-tail safety — 2026-09-23

- [x] Add a failing depth-pyramid shader contract covering input-dimension clamping and the `3x5 -> 1x2 -> 1x1` rectangular mip tail.
- [x] Clamp the doubled source coordinate and all four `textureLoad` source coordinates to the input mip extent before reduction.
- [x] Run the focused depth-pyramid, frame-dimension, and dynamic-dimension tests: 3, 9, and 4 tests passed respectively.

## Task 2 — Initial camera projection alignment — 2026-09-23

- [x] Add a failing viewer contract for authoritative startup camera dimensions and the existing resize path.
- [x] Initialize all camera projections from the render-context surface dimensions after scene setup.
- [x] Reuse the same clamped projection helper for `WindowEvent::SizeChanged`.
- [x] Run focused viewer tests and the viewer WASM check.

Task 2 results are recorded in `.superpowers/sdd/2026-09-23-wavefront-resolution-resize/task-2-report.md`.

## Task 4 � maximize-sized native queue capacity � 2026-09-23

- [x] Add the native `1916x1125` regression and require primary capacity `>= 2_155_500`; portable fallback assertions remain unchanged.
- [x] Raise the discrete-native aggregate retained queue budget from `1536 MiB` to `2 GiB`; derive the hard recovery ceiling as `2_796_202` entries from the checked aggregate bytes-per-entry constant. Portable/WASM fallback remains the `512 MiB` policy.
- [x] Focused transport regression initially failed at `2_097_152` entries, then passed after the native policy update: `43 passed; 0 failed`.
- [x] Native wavefront integration passed: `2 passed; 0 failed`.
- [x] Maximize-sized Cornell smoke passed `INOX_NATIVE_CORNELL_IMAGE_PASS` at `1916x1125`; queue diagnostics reported `INOX_NATIVE_WAVEFRONT_QUEUE_NO_OVERFLOW` and emitted no `wavefront queue primary overflowed` diagnostic.

