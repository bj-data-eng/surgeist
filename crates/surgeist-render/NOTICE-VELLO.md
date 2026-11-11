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
| `src/vello_engine/scene.rs` | `vello-0.9.0/src/scene.rs` | `7c225e73f56629b1b85e8e5cd296428176ec6e59a0813975e2d4123aaddd1718` | Keeps private Vello-compatible scene lowering for the crate's commands and moves selected-glyph work behind a fallible preflight boundary; removes the public `Scene` surface and upstream font `unwrap` paths. Resolves gradient glyph brushes with the actual final paint transform, checks the separately narrowed native glyph/brush mapping, and captures owned analytic sampling records before encoding. |
| `src/vello_engine/glyph.rs` | `vello-0.9.0/src/scene.rs` | `7c225e73f56629b1b85e8e5cd296428176ec6e59a0813975e2d4123aaddd1718` | Extracts selected-glyph font, outline, COLR, bitmap, and PNG validation from scene lowering; converts malformed or unsupported inputs to typed diagnostics before external encoding and never logs-and-omits a glyph. |
| `src/vello_engine/raster.rs` | `vello-0.9.0/src/render.rs` | `f75a73fae27085c870273b6e670f355455eea61f1d1dde9b102ab9ed2528e7ed` | Retains the coarse/fine recording schedule over `vello_encoding` while producing a WGPU-free prepared pass with symbolic target and resource intents instead of an upstream renderer graph. |
| `src/vello_engine/recording.rs` | `vello-0.9.0/src/recording.rs` | `3c760a7c7610274443efe06c2e9a37eb71471b14a6635d9f65ce92b39de98b3c` | Recasts upstream recording proxies as private handles, resource intents, and dispatch records for the fixed raster schedule; it owns no WGPU object, polling, or submission path. |
| `src/vello_engine/shaders.rs` | `vello-0.9.0/src/shaders.rs` | `c1392afa0ce8d33873e43a26ba79e881adb0a53e2ed92a90201fac5592a0058e` | Retains shader selection and WGSL binding metadata from external WGSL-only `vello_shaders`; creates modules and pipelines through checked WGPU error scopes rather than the trusted upstream creation path. Applies exact gray and endpoint luminosity identities and pre-composition mixing-result bounds to all fine raster shaders through checked, Ledger-accounted source adaptation, preserving separate local Plus arithmetic. Routes color dodge, color burn, soft light and nonseparable blend helpers through shared Render shader math. Defines the zero/subnormal circular norm boundary and ordinary gradient endpoint selection for all fine programs through the same checked source adaptation. |
| `src/vello_engine/encoder.rs` | `vello-0.9.0/src/wgpu_engine.rs` | `d2bbb8151f27d7fd4ff82abaa1438e05cb45468dab36034f48e54eefba183e7c` | Realizes symbolic recordings, uploads, bind groups, and compute passes into a transaction-borrowed encoder; returns a lease and cannot submit command buffers or own a renderer. |
| `src/vello_engine/resources.rs` | `vello-0.9.0/src/wgpu_engine.rs` | `d2bbb8151f27d7fd4ff82abaa1438e05cb45468dab36034f48e54eefba183e7c` | Replaces the upstream engine's resource ownership with checked per-device allocation and explicit lease commit, abort, and atlas-recovery state under Surgeist transaction control. |

Every derived file listed above retains its upstream copyright and SPDX header
and identifies its local modification with a reference to this record.
`src/vello_engine/mod.rs` is Surgeist-owned composition that wires the derived
private phases together, so it deliberately carries no upstream source header.

## Field qualification shader correspondence

The shared native field capacity checks in
`src/encode/gradient_field.rs`, used by ordinary and repeating realization, adapt
linear endpoint projection and the concentric circular setup from
`vello_shaders` 0.9.0, `shader/draw_leaf.wgsl`, to qualify scalar field arithmetic
before encoding. They compute conservative bounds over the actual device target;
they neither produce pixels nor replace the GPU rasterizer. The upstream source
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

Ordinary realization uses this correspondence with absolute linear phase and
the actual pair of concentric radii, including a negative starting radius.
Repeating realization retains its separate phase reduction and averaging rules.

Sweep receiving qualification follows the same `draw_leaf.wgsl` source's
transform/center composition and inverse setup, together with
`shader/shared/transform.wgsl` recorded below. It checks the actual effective
native factors, rather than encoding a replacement CPU inverse. Its bounded
consumer follows the integer raster lanes and slope/angle expressions in the
same `fine.wgsl` source identified below. The native angle polynomial, center
handling and ramp sampling remain unchanged. The qualification retains the
existing shader license alternatives and modification notice.

The same checked fine-program source adapter defines the zero/subnormal circular
norm boundary from `vello_shaders` 0.9.0 `shader/fine.wgsl`, whose source SHA-256 is
`efd4a38f4a6a32518b33b497789723b8f163ada361f683c2dfa3700fe8b2d68d`.
It retains the surrounding circular and noncircular branches and uses the same
shader-specific license alternatives above.

The ordinary gradient adaptation checks the linear, radial and sweep lookup
boundaries in that same `fine.wgsl` source for every Area, MSAA8 and MSAA16
program. A private mode-3 draw word selects an eight-word descriptor appended to
the existing RGBA8 gradient texture. The descriptor selects an unhinted Pad ramp
or an analytic Pad/Repeat segment program, including horizontal-distance
sampling; native ramp rows remain unchanged. The eighth word retains the phase
only for a distance Repeat program and is zero for other interpretations.
The Surgeist-owned helper in
`src/vello_engine/shaders/gradient_endpoints.wgsl` loads flat addresses across
512-wide rows, reconstructs scalar words with `pack4x8unorm`, and compares
endpoint words as ordered binary32 representations before native Pad clamping.
It canonicalizes signed zero without floating-point endpoint arithmetic, retains
subnormal threshold bits, and selects the last endpoint at an all-coincident
threshold. Radial selection restores the native swap orientation before its
endpoint comparison. The existing coverage operation consumes the selected
premultiplied color once. Native modes 0, 1 and 2 retain their lookup operations;
unhinted strict interior samples retain the original ramp row and 512-sample
rounding. NaN coordinates retain the previous native Pad lookup on that row.
Analytic programs carry one seven-word record per actual color stop: position,
four premultiplied binary32 channels, outgoing segment kind and exponent. These
words use the same exact-byte transport. A bounded ordered search preserves
coincident runs before linear, power or endpoint-limit interpolation. The
Surgeist-owned coefficient and whole-segment numerical certificate qualify the
power arithmetic before encoding; they do not derive from an upstream color
stop approximation. Selection is relative to the actual native computed
coordinate, without changing or
recovering earlier field arithmetic. The private native GPU probe executes this
same helper against RGBA8 byte/scalar words, row crossings, endpoint cases and
analytic segment programs.

Horizontal-distance sampling uses the actual linear draw carrier. The
Surgeist-owned helper clears its signed coordinate's sign bit before endpoint
comparison or Repeat phase subtraction and wrapping. Signed ordinary controls
remain unshifted; distance programs and coincident steps carry one coupled
interpretation through capture and deferred glyph encoding. These additions
reuse the checked fine-program lookup boundaries and preserve the surrounding
coverage operation. The native probe also covers folded controls, phase seams
and subnormal step thresholds. No upstream license text or source hash changes.

## Native geometry receiving correspondence

The private `src/vello_engine/geometry.rs` receiving checks qualify the actual
native coordinates and factors before stream publication. Their segment-retention
correspondence follows `vello_encoding` 0.9.0's path encoder, including its local
zero-length and initial-tangent predicate. Matrix and deferred glyph composition
follow the same package's actual narrowed factors. These checks preserve the
existing native curve/coverage approximation and refuse an unqualified required
representation; they do not replace GPU geometry execution.

The arithmetic correspondence also follows `vello_shaders` 0.9.0's native
flattening and shared transform expressions, including nested fused operations
and permitted subnormal behavior. The selected source files are:

| Package and source | SHA-256 |
| --- | --- |
| `vello_encoding` 0.9.0, `src/math.rs` | `59d65cad5505858a13b5eb11251d31166119cf5ae1bc1cc3514009523344014f` |
| `vello_encoding` 0.9.0, `src/path.rs` | `ea94b3018a243f29e6954f4669a247dcea2d2fbaac23b9f376d00ad9cab78c49` |
| `vello_encoding` 0.9.0, `src/resolve.rs` | `b246758ed97ece009f2b5852c9632b387dc7950dfb84604ff0802974623bb03b` |
| `vello_shaders` 0.9.0, `shader/flatten.wgsl` | `d610683837057363eb37186e55377b7873ada7e06cfa58b58eddfded1badd403` |
| `vello_shaders` 0.9.0, `shader/shared/transform.wgsl` | `da5d9c0a83f04d3f70432a13250466e8632960761cda1ae69632d2bc325d3552` |

The encoding sources retain `Copyright 2022 the Vello Authors` and
`Apache-2.0 OR MIT`; the shader sources additionally permit `Unlicense`.
The existing linked license texts preserve these alternatives unchanged.

Source-to-native point correspondence uses an explicit 0.25-device-pixel
Euclidean receiving budget. The pinned flattening shader's curve and arc
`tol` values inform this policy, but do not establish a total conversion
or raster guarantee: the retained backend approximation is separately additive.
The source/native affine-difference bound is Surgeist-owned qualification.
Image and blurred-rectangle inverse qualification follows the actual
`shared/transform.wgsl` determinant, reciprocal, matrix and translation chain.
Its consumer envelope also follows the permitted upward rounding of the actual
fine-shader integer conversion, lane addition and image pixel-center addition.

Native glyph qualification also uses the existing locked Skrifa 0.42.1 provider
to distinguish a true empty outline from loss during its default scaled
FreeType conversion. An unscaled outline is consulted only when the actual
scaled result has no real segments; it is a qualification witness and is never
substituted for the cache's drawing geometry or settings.

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
