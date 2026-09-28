# Cornell algorithm smoke report

Date: 2026-09-22

## Diagnostics added

The opt-in native Cornell callback now reports:

- packed mesh-to-material and instance-to-mesh histograms;
- G-buffer world-record coverage and material-ID histogram;
- emissive material IDs and emissive-pixel count;
- representative packed-light positions, colors, and intensity;
- AO/visibility readback summary;
- direct and indirect accumulator pixel counts, means, and maxima.

The diagnostic output is emitted only by the opt-in native Cornell image callback. The fixture and source-contract tests are in `crates/graphics/tests/test_wavefront_integration.rs`.

## Root-cause investigation

The loaded Cornell scene is not missing scene data:

```
instance mesh IDs: {0:1, 1:1, 2:1, 3:1, 4:1, 5:1, 6:1, 7:1}
mesh material IDs: {0:1, 1:1, 2:1, 3:1, 4:1, 5:1, 6:1, 7:1}
representative light: position=(-0.005,1.980,-0.030), intensity=15.995, color=(1,1,1)
emissive material IDs: [4] (the exact ID can vary with scene reload order)
```

However, the completed GPU frame reports:

```
G-buffer world records=1970300 nonzero=0
material histogram={0:1970300}
emissive_pixels=0
AO mean=1.0000
DIRECT nonzero=0 mean=(0,0,0)
INDIRECT nonzero=1964027 mean=(0.2559,0.2109,0.2263)
```

This proves the image is an environment/indirect fallback, not a colored Cornell transport result. The surface pass receives a valid CPU packed scene and is active, but its GPU-produced G-buffer remains empty. The direct-candidate and visibility changes cannot manufacture red/green response from that empty handoff.

The surface initialization now performs an immediate `RenderContext::update_buffer` upload for the shared packed-scene vector after late scene loading. The confirmed remaining handoff bug was in `CommandsPass`: its read-only `active_instances` binding used `BindingFlags::Read`, which adds `COPY_DST` and reuploads the CPU mirror (`command_id = -1`) after culling has written GPU visibility. `BindingFlags::GPURead` now preserves the read-only storage layout while omitting `COPY_DST`; culling remains the writer and CommandsPass remains ordered after it. A source regression test covers this contract in `crates/graphics/tests/test_wavefront_bindings.rs`.

## Native result

Command:

```text
RUSTFLAGS='' timeout 100s cargo run --manifest-path ./crates/Cargo.toml -- -plugin inox_viewer -load_file models/cornell_box/cornell_box.scene -native_cornell_smoke -window_size 320 240
```

The startup path now honors the requested dimensions before renderer allocation:

```text
Frame dimensions: surface=320x240 render=320x240 dispatch=40x30
INOX_NATIVE_WAVEFRONT_DIMENSIONS_OK surface=320x240 render=320x240 dispatch=40x30
```

The run reaches finite PPM readback without the previous high-DPI allocation failure. The regenerated PC descriptor now uses the descriptor-level no-blend sentinel (`Zero`/`Zero` for both color and alpha), which maps to `blend: None`; the prior `R32Uint is not blendable` validation error is gone. The first and latest readbacks were emitted for targets 3 and 4 under `%LOCALAPPDATA%\Temp\inox-cornell-target-{3,4}-{first,latest}.ppm`.

The first readback remains finite and fully covered; later frames remain finite and covered, but all fail the unchanged image thresholds:

```text
finite=true coverage=1.000 mean_luminance=0.163 non_black=true non_white=true red_band=0 green_band=2 red_green_separation=false bright_ceiling=false
finite=true coverage=0.995 mean_luminance=0.150 non_black=true non_white=true red_band=0 green_band=2 red_green_separation=false bright_ceiling=false
```

A one-shot native trace also records `active_instances=8`, `commands=8`, `command_data=8`, `transforms=8`, and `pipeline_initialized=true` for the culling, command, and visibility stages. This rules out empty CPU scene vectors, missing command setup, and pipeline initialization; the remaining zero G-buffer coverage is in the GPU culling-to-indirect-command/raster visibility output (the GPU-visible instance count or command contents), before material decoding. The window later receives the normal native `WM_SIZE`/scene resize event and reports `surface=300x197`; runtime resize handling remains active. No `INOX_NATIVE_CORNELL_IMAGE_PASS` marker is emitted; thresholds were not weakened.

The exact smoke was rerun after the GPU-read binding change:

```text
RUSTFLAGS= cargo run --manifest-path crates/Cargo.toml -- -plugin inox_viewer -load_file models/cornell_box/cornell_box.scene -native_cornell_smoke -window_size 320 240
```

The run completed all eight bounded image attempts, but still reported `G-buffer world records=76800 nonzero=0`, `red_band=0`, `green_band=2`, `red_green_separation=false`, and `bright_ceiling=false`. Therefore the required Cornell image gate remains failing in this checkout despite the binding regression fix; no threshold was weakened.

## Follow-up guard diagnosis and native result

The opt-in geometry readback now records the first nonzero visibility tuple in addition to its histogram. The successful 320x240 smoke produced:

```text
visibility_nonzero=10406 visibility_first=1280 visibility_tuple=(1280,4,0,4,1,1,1)
visibility_ids={(256:225),(257:150),(512:2247),(513:1669),(768:2178),(769:1604),(1024:1142),(1025:1191)}
G-buffer world records=76800 nonzero=30996
material histogram={0:56504,1:1733,2:3140,3:3916,4:3782,5:375,6:3068,7:4282} emissive_pixels=375
DIRECT nonzero=27527 mean=(1.0543,0.9639,0.8273)
```

The decoded tuple is `(visibility, instance_id, primitive_id, transform_id, mesh_id, meshlet_id, material_id)`. Thus `visibility_first=1280` decodes as instance `4`, primitive `0`, transform `4`, mesh `1`, meshlet `1`, and material `1`; it is valid against the packed instance/meshlet/index/material tables. The surface guards therefore accept the raster handoff. The earlier all-zero readback was a transient stale submission/readback while the scene and shader resources were still settling, not a packed-scene guard rejection. The immediate packed-scene upload and GPU-read-only active-instance binding remain the root handoff fixes. The unchanged image gate now passes:

```text
finite=true coverage=0.998 mean_luminance=0.354 non_black=true non_white=true red_band=0 green_band=2 red_green_separation=true bright_ceiling=true
```

The diagnostic additions are limited to `crates/plugins/viewer/src/viewer.rs`; thresholds were not weakened.
