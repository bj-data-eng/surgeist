# surgeist-render

A Rust library for Surgeist integrators who need GPU rendering of scenes onto
headless or presented surfaces. It owns scene validation, GPU execution, surface
resources, explicit headless readback, and render diagnostics.

The current implementation is GPU-only, with direct Vello and WGPU graph routes.
It requires a compatible GPU; browser execution and application/window lifecycle
belong to the integrating host. The current style-facing exports and intended
crate boundary are described in the [architecture guide](docs/explanation.md).

Native geometry limits produce a typed contextual refusal before publication.
The [receiving contract](docs/reference.md#native-geometry-realization) describes
coordinate, segment and transform qualification and preserves prior pixels and
statistics when a required representation cannot qualify.

Layers support all sixteen standard numeric-sRGB mixing functions, twelve basic
Porter-Duff operators and explicit isolated groups. New non-source-over operators
use the GPU graph; source-over mixing retains both routes and the existing local
Plus operation remains distinct. [Layer blending](docs/reference.md#layer-blending)
and [composition](docs/reference.md#layer-composition-and-isolation) describe the
alpha, operation-domain and capability contracts.

Prepared native convolution runs arbitrary layer children through the GPU graph.
The [convolution contract](docs/convolution.md) describes parameter admission,
source and result domains, anisotropic grids, edge modes, color and alpha math,
precision, accounting and typed refusals.

Prepared [native Gaussian blur](docs/gaussian.md) admits independent standard
deviations and authentic finite, constant or periodic sources. It preserves
independent domains, full outer transforms and color semantics through bounded
float32 GPU accumulation and atomic publication.

Prepared [native drop shadow](docs/native-shadow.md) preserves a single original
capture above an alpha-only Gaussian, finite displacement and colored flood. It
keeps independent stage domains, working color, bounded outer composition and
atomic resource lifetime.

Prepared [native offset](docs/offset.md) translates the captured input inside
independent source, filter and result domains. It preserves numeric
premultiplied color and the full outer affine, with exact aligned copies and
explicit fractional reconstruction for each ImageRendering hint.

Ordinary linear and positive elliptical radial paints preserve ordered finite
`f64` stop positions, exterior colors and coincident steps. Repeating fields
share the stop primitive and choose a solid premultiplied-sRGBA average when
final device resolution cannot resolve a cycle. Outgoing `GradientHint` values
control analytic premultiplied interpolation; repeating averages retain the
specified stop-only weights.
Horizontal-distance fields mirror physical stop distances around a local x
origin. They execute native zero-width radial gradients through the same fill,
stroke and glyph routes; [their contract](docs/reference.md#horizontal-distance-fields)
defines the fold, phase and precision boundaries.
[Gradient paints](docs/reference.md#gradient-paints) describe range, period,
transform and accounting behavior.

With `render-paint`, `prepare_linear_gradient`, `prepare_radial_gradient` and
their `prepare_repeating_*` counterparts consume Paint's native geometry and
used stops through a `RenderAccountingContext`. They return owned fields for
normal scene commands. The [native Paint consumer](docs/reference.md#native-paint-consumer)
defines positive and zero-width geometry support, unit conversion and typed refusals.
`prepare_zero_height_gradient_color` consumes Paint's authenticated zero-height
color measure and returns an owned solid `Color`; its [reference](docs/reference.md#zero-height-paint-colors)
describes the separate preparation contract.

`FontData` accepts raw OpenType, collections, WOFF and WOFF2 through
`surgeist-font`. `FontData::prepare_glyph_run` admits immutable glyph resources
without a GPU.
It retains qualified outline paths, COLR paint operations or decoded sbix PNG
pixels, with representation-derived `Ink` or proven `Empty` bounds. The PNG
path interprets supported `sRGB`/`gAMA`/`cHRM` source colors into straight-alpha
sRGB, with explicit native defaults, white adaptation and terminal clipping. Direct and
graph drawing consume those same resources. The [glyph admission
reference](docs/reference.md#glyph-admission) defines the supported formats,
effective instances, numeric rules and explicit resource limits.

The optional `render-text` feature adds `prepare_text_layout`. It consumes the
exact immutable placed `surgeist_text::Layout`, preserving its font instances,
glyphs, lines and positions, including unrotated native upright and clockwise
or counterclockwise sideways runs. One `TextLayoutLimits` budget covers the complete
Scene; a later refusal publishes no prefix. Text shaping, line selection and
live computed-style authentication belong to their producing subsystems.

The same feature adds `prepare_leader_pattern`, which borrows one exact finalized
Text leader and admits its original glyph representations and paint groups.
The receipt reports conservative ink envelopes in Text's physical pattern-local
frame, including proved Empty, with central `RenderAccountingContext` limits.
It preserves canonical combined fills and intrinsic paint barriers. Layout owns
logical mapping, period and repetition; whole-group masking and command lowering
remain separate operations.

Ledger owns Render's named work, capacity admission and optional bounded profiling.
Set `Options::with_accounting` before creating a Renderer. Each frame has a fresh
work allowance, while Renderer CPU backing and native device generations retain
their own lifetime custody. `Renderer::last_attempt` reports success, refusal,
failure or cancellation separately from last-successful `Stats`; pure lifetime
snapshots expose resources retained between frames.
The [frame accounting reference](docs/reference.md#frame-accounting) explains
these domains, limits and host-observed timings.

Standalone preparation, scene inspection and allocating style normalization use
a reusable `RenderAccountingContext`. Their reports describe the operation and
explicit output transfer; returned values carry no hidden accounting callbacks.
See [standalone accounting](docs/reference.md#standalone-accounting) and
[preparation accounting](docs/reference.md#preparation-accounting) for the measured
units. Native [device-generation custody](docs/reference.md#device-generation-custody)
also covers readback staging and late callbacks. Explicit readback has its own
fresh work allowance and `Renderer::last_readback_attempt` report, preserving
frame statistics and reports. The [readback accounting reference](docs/reference.md#readback-accounting)
distinguishes logical pixel bytes, actual output capacity and final caller transfer.

## Start

From this checkout, run the focused render-and-readback test:

```sh
CARGO_NET_OFFLINE=true cargo test -p surgeist-render --lib tests::surface::headless_render_can_be_read_back -- --exact
```

Expect one passing test: it renders a 4 × 4 scene on the GPU and reads back
64 RGBA bytes. See [getting started](docs/getting-started.md) for prerequisites
and the full first-success walkthrough.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Render and read back your first headless frame. |
| [How-to](docs/how-to.md) | Exercise native presentation and check the browser compilation boundary. |
| [Reference](docs/reference.md) | Find public interfaces, features, diagnostics, and verification commands. |
| [Native convolution](docs/convolution.md) | Prepare and execute resolved convolution with explicit domains, grids and numeric policy. |
| [Native resolved drop shadow](docs/native-shadow.md) | Blur alpha, displace its finite result and merge the unchanged original above a colored shadow. |
| [Native offset](docs/offset.md) | Translate finite captured images with independent clipping and resolved reconstruction intent. |
| [Explanation](docs/explanation.md) | Understand execution routes, publication, precision, and ownership. |

## License and attribution

The project is licensed under the [MIT License](LICENSE). [Third-party
attribution](NOTICE.md) covers direct dependencies and bundled material.
