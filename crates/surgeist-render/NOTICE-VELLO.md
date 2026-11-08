# Vello 0.9.0 provenance

`surgeist-render` derives its private raster-engine modules from the pinned
Vello main crate described below. This notice records the source state before
local adaptation; it is not a copy of Vello's public crate or API.

The repository's [third-party notice](NOTICE.md) covers the direct dependencies
and bundled test asset alongside this adaptation record.

## Pinned package

- Package: `vello` 0.9.0.
- Cargo package checksum: `261359dbef879f8110ef7e1c442246c838d33d3d91cb05e0ea9288d432760c9f`.
- Source URL: <https://github.com/linebender/vello>.
- License expression: `Apache-2.0 OR MIT`.

## Imported upstream source files

| Local derived file | Upstream source | Pre-adaptation SHA-256 | Material adaptations |
| --- | --- | --- | --- |
| `src/vello_engine/scene.rs` | `vello-0.9.0/src/scene.rs` | `7c225e73f56629b1b85e8e5cd296428176ec6e59a0813975e2d4123aaddd1718` | Keeps private Vello-compatible scene lowering for the crate's commands and moves selected-glyph work behind a fallible preflight boundary; removes the public `Scene` surface and upstream font `unwrap` paths. Resolves repeating glyph brushes with the actual final paint transform and checks the separately narrowed native glyph/brush mapping before encoding. |
| `src/vello_engine/glyph.rs` | `vello-0.9.0/src/scene.rs` | `7c225e73f56629b1b85e8e5cd296428176ec6e59a0813975e2d4123aaddd1718` | Extracts selected-glyph font, outline, COLR, bitmap, and PNG validation from scene lowering; converts malformed or unsupported inputs to typed diagnostics before external encoding and never logs-and-omits a glyph. |
| `src/vello_engine/raster.rs` | `vello-0.9.0/src/render.rs` | `f75a73fae27085c870273b6e670f355455eea61f1d1dde9b102ab9ed2528e7ed` | Retains the coarse/fine recording schedule over `vello_encoding` while producing a WGPU-free prepared pass with symbolic target and resource intents instead of an upstream renderer graph. |
| `src/vello_engine/recording.rs` | `vello-0.9.0/src/recording.rs` | `3c760a7c7610274443efe06c2e9a37eb71471b14a6635d9f65ce92b39de98b3c` | Recasts upstream recording proxies as private handles, resource intents, and dispatch records for the fixed raster schedule; it owns no WGPU object, polling, or submission path. |
| `src/vello_engine/shaders.rs` | `vello-0.9.0/src/shaders.rs` | `c1392afa0ce8d33873e43a26ba79e881adb0a53e2ed92a90201fac5592a0058e` | Retains shader selection and WGSL binding metadata from external WGSL-only `vello_shaders`; creates modules and pipelines through checked WGPU error scopes rather than the trusted upstream creation path. Applies exact gray and endpoint luminosity identities and pre-composition mixing-result bounds to all fine raster shaders through checked, Ledger-accounted source adaptation, preserving separate local Plus arithmetic. Defines the zero/subnormal circular norm boundary for all fine programs through the same checked source adaptation. |
| `src/vello_engine/encoder.rs` | `vello-0.9.0/src/wgpu_engine.rs` | `d2bbb8151f27d7fd4ff82abaa1438e05cb45468dab36034f48e54eefba183e7c` | Realizes symbolic recordings, uploads, bind groups, and compute passes into a transaction-borrowed encoder; returns a lease and cannot submit command buffers or own a renderer. |
| `src/vello_engine/resources.rs` | `vello-0.9.0/src/wgpu_engine.rs` | `d2bbb8151f27d7fd4ff82abaa1438e05cb45468dab36034f48e54eefba183e7c` | Replaces the upstream engine's resource ownership with checked per-device allocation and explicit lease commit, abort, and atlas-recovery state under Surgeist transaction control. |

Every derived file listed above retains its upstream copyright and SPDX header
and identifies its local modification with a reference to this record.
`src/vello_engine/mod.rs` is Surgeist-owned composition that wires the derived
private phases together, so it deliberately carries no upstream source header.

## Radial qualification shader correspondence

The repeating-gradient native capacity check in
`src/encode/repeating_gradient.rs` adapts the concentric circular setup from
`vello_shaders` 0.9.0, `shader/draw_leaf.wgsl`, to qualify scalar field arithmetic
before encoding. It computes conservative bounds over the actual device target;
it neither produces pixels nor replaces the GPU rasterizer. The upstream source
SHA-256 is `489cfa784f42f1c4802950cec874c6d488b286da203acd0c7dbce382a595591a`.
The locked package checksum is
`9dd38937516fa4b47423d9255bb5e4a65e839ec9d57c38c4af6189ce56bf46b7`.
The upstream project is <https://github.com/linebender/vello>.

That shader carries `Copyright 2022 the Vello Authors` and
`Apache-2.0 OR MIT OR Unlicense`; the derived qualification retains this header
and identifies the modification here. The existing Apache-2.0 and MIT texts
below are byte-identical to this package's copies. The additional
[Unlicense text](LICENSES/Vello-0.9.0-UNLICENSE.txt) preserves its shader-specific
alternative unchanged.

The same checked fine-program source adapter defines the zero/subnormal circular
norm boundary from `vello_shaders` 0.9.0 `shader/fine.wgsl`, whose source SHA-256 is
`efd4a38f4a6a32518b33b497789723b8f163ada361f683c2dfa3700fe8b2d68d`.
It retains the surrounding circular and noncircular branches and uses the same
shader-specific license alternatives above.

## Omitted upstream main-crate sources

| Upstream source | Rationale |
| --- | --- |
| `vello-0.9.0/src/lib.rs` | Defines Vello's public facade, public renderer/options/errors, reexports, and feature wiring; Surgeist retains its own crate front door and private engine boundary. |
| `vello-0.9.0/src/util.rs` | Owns Vello device and surface utilities, including a blocking device-poll helper; Surgeist owns WGPU lifecycle and does not permit blocking, map, or poll execution in the internal engine. |
| `vello-0.9.0/src/debug.rs` | Defines optional debug-layer control whose documented validation route requires CPU readback; it is outside the checked production raster path. |
| `vello-0.9.0/src/debug/renderer.rs` | Implements debug visualization, debug downloads, and mapped-buffer CPU validation; it is excluded with Vello debug/readback support. |
| `vello-0.9.0/src/debug/validate.rs` | Implements CPU line-soup validation and logging for the optional debug layer; it is not part of production rendering. |

The imported upstream files and these omissions account for every Rust source
file under the pinned package's `src/` directory.

## License artifacts

The tracked [Apache-2.0 license](LICENSES/Vello-0.9.0-APACHE-2.0.txt) and
[MIT license](LICENSES/Vello-0.9.0-MIT.txt) preserve the pinned package's
verbatim dual-license texts.
