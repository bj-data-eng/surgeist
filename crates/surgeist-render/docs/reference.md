# Reference

## Package and public API

[Cargo.toml](../Cargo.toml) defines package `surgeist-render`, version `0.1.0`,
Rust edition 2024, and library import name `surgeist_render`. The public front
door is [src/lib.rs](../src/lib.rs); its reexports and their rustdoc own the
current source API.

| Area | Public entry points | Source |
| --- | --- | --- |
| Renderer and options | `Renderer`, `Options`, `Parameters`, `EffectQualityPolicy` | [Renderer](../src/renderer/mod.rs), [options](../src/renderer/options.rs), [surface parameters](../src/surface.rs) |
| Draw data | `Scene`, geometry, shapes, paints, images, layers, and positioned text runs | [Scene](../src/scene.rs), [public reexports](../src/lib.rs) |
| Placed Text consumer (`render-text`) | `prepare_text_layout`, `PreparedTextScene`, `TextLayoutLimits`, `TextLayoutError` | [Adapter](../src/text_layout.rs) |
| Surface and pixels | `Surface`, `SurfaceOptions`, `Attachment`, `ImageBuffer` | [Surfaces](../src/surface.rs), [images](../src/image.rs) |
| Support and diagnostics | `Capabilities`, `RuntimeCapabilities`, `Error`, `ErrorCode` | [Capabilities](../src/capability.rs), [errors](../src/error.rs) |
| Successful frame observations | `Stats`, `RenderRoute`, `EffectPrecision` | [Statistics](../src/stats.rs) |
| Preparation accounting | `RenderAccountingContext`, `RenderAccountingOptions`, `RenderLimits`, `RenderAttemptReport`, named work/resources/phases | [Accounting](../src/accounting/mod.rs), [units and reports](../src/accounting/model.rs) |
| Current style-facing models | Background, border, outline, image-placement, mask, clip, and filter values | [Style module](../src/style/mod.rs), [public reexports](../src/lib.rs) |

The `Renderer` methods `new`, `create_surface`, `create_headless`,
`resume_surface`, `render`, and `read_headless` are asynchronous GPU operations.
`Renderer::read_headless` returns an
`ImageBuffer` containing tightly packed straight-alpha RGBA8 physical pixels from
the current complete headless publication. Rendering does not invoke this
readback implicitly.

## Capabilities and errors

`Capabilities::CURRENT` reports semantic support for authored rendering
operations and compiled surface availability. Its surface report supports
headless construction and enables its web-canvas flag only with both
`render-web` and the `wasm32` architecture. Native window attachment is separately
compiled behind `render-window`.

`Renderer::runtime_capabilities` reports availability or immutable facts about
the selected device and surface: surface format, supported effect precisions,
and maximum two-dimensional effect texture size. Cargo features select compiled
host adapters; enabling a feature does not establish runtime device support.

Unsupported semantic operations and unavailable runtime capabilities have
distinct typed diagnostics. Missing adapters, required formats, device limits,
and surface lifecycle availability do not authorize partial frame publication.
The [error API](../src/error.rs) defines the operation and reason payloads.

## Image sampling

`Image::rendering` accepts an `ImageRendering` hint already selected by the
surrounding style owner. Callers resolve inheritance and apply that selection to
each applicable image. Render executes the selection for image draws, image
paints and resolved image alpha masks without inspecting an element tree.

| Hint | Current execution |
| --- | --- |
| `Auto` | Default bilinear sampling. |
| `Smooth` | Bilinear smoothing. |
| `HighQuality` | Bicubic smoothing. |
| `CrispEdges` | Nearest-neighbor sampling, preserving source colors. |
| `Pixelated` | Nearest-neighbor enlargement to the nearest positive integer multiple on each source axis, followed by bilinear scaling to the destination. |

Pixelated multiples use each transformed source axis independently, including
device scale. A half-integer scale selects the larger multiple, and downscaling
uses a multiple of one. Color images are enlarged by a GPU compute pass into
the image atlas. Alpha masks sample the equivalent virtual enlarged grid.
Both paths smooth premultiplied contributions; transparent source colors do not
bleed into visible output.

`Image::quality` remains available for explicit backend quality selection.
Selecting a rendering hint replaces that quality selection, and selecting a
quality clears the rendering hint. `Image::rendering_hint` reports the selected
hint. Pixel content and its compact fingerprint do not change, while complete
image equality includes the sampling selection.

There is currently no automatic quality degradation under resource pressure.
Checked atlas representation and runtime device limits can refuse a frame;
publication remains atomic. `HighQuality` therefore retains its selected
smoothing rather than silently falling back to another hint.

Image-atlas preparation uses `surgeist-ledger` to admit aggregate original upload
bytes and expanded texel work before packing and recording uploads. These are
defined operation costs, not allocator or process-memory measurements. The
existing resource manager continues to own GPU residency and cross-frame leases.

## Host features

| Configuration | Leaf boundary |
| --- | --- |
| Default features (`[]`) | Native headless GPU execution without a window. |
| `render-text` | Optional direct Text dependency, without Text default features; immutable placed-layout consumption, independent of window/web/accessibility adapters. |
| `render-window` | Enables `surgeist-window` and native window attachment; live presentation requires its graphical host lifecycle. |
| `render-web` | Enables `wgpu/webgpu`; web-canvas support requires `wasm32`. The documented wasm leaf check compiles `--lib --tests`. |
| `render-window,render-web` on native | Additive compiled features; uses the same native live-window smoke target. |

Native WebCanvas surface creation returns a typed unsupported-platform
diagnostic. Real browser event-loop execution and presentation belong to root
integration, even after a successful wasm compilation.

## Compatibility

This crate's [manifest](../Cargo.toml) declares edition 2024 and intentionally has
no `rust-version` field. The [root manifest](../../../Cargo.toml) owns the
facade integration floor.

## Glyph admission

`FontData::try_from_bytes` takes an original face index and caller-supplied
`surgeist_font::DecodeLimits`. Font admits raw SFNT or collection bytes and
decodes WOFF or WOFF2. Decode refusals remain available as the source of the
Render error. `FontData::from_face` takes an already admitted Font face.
`FontData::admitted_face` exposes its original index, decoded resource and
exact encoded source. These entry points are available in standalone Render.

`FontData::prepare_glyph_run` retains the selected representation and its font,
executed `TextInstance` and original `TextGlyph` facts. Its `PreparedGlyphRun`
provides representation-derived `Ink` or source-proved `Empty` bounds.
`Scene::prepared_glyph_run` shares those immutable resources with direct and graph
execution; glyph advances never substitute for ink.

Consecutive foreground outline operations share one NonZero fill, preserving
combined winding and opacity. An intrinsic COLR or bitmap selection separates
those spans even when its output is Empty. Intrinsic paints retain layer order,
clip and currentColor behavior. Caller paint applies to foreground operations.

The supported intrinsic profile is COLRv0, a COLRv1 solid/glyph/translation/static
clip subset, and sbix noninterlaced RGBA8 PNG containing only IHDR, contiguous IDAT
and IEND chunks. Other paint operations, formats and PNG chunks refuse explicitly.
Color and bitmap admission requires a default effective instance without
synthesis; supported outline variation and synthesis follow the policy below.
Malformed selected COLRv1 data cannot fall back to a COLRv0 representation.

Bitmap placement follows the pinned Skrifa 0.42.1 Skia/CoreText convention:
font-unit outer bearings and strike-pixel offsets establish the selected quad.
Drawing fills the retained canonical endpoint rectangle with its retained image
sampling mapping. PNG admission validates chunk order and CRC, exact zlib input
and output extent, Adler checksum and all five scanline filters before retaining
pixels. The inflater uses fixed-state miniz_oxide with caller-owned buffers.

`GlyphRunLimits` bounds glyphs, retained outline path elements, scratch storage
and retained logical custody. Charges include the retained Font resource,
defined fixed result units, actual buffer capacities, fixed provider workspace
and growth overlap. Font's separate `DecodeLimits` cover container admission.
These limits do not measure the allocator or bound CPU time. A failure discards
the entire unpublished result; the caller's inputs remain available for retry.
The authored `TextRun` route retains its existing provider behavior and supports
`Unspecified` bounds only on its direct route.

The optional `prepare_text_layout` adapter consumes the actual placed Text
snapshot without reshaping or choosing lines. Its complete `TextLayoutLimits`
budget also covers retained Font resources, mapping buffers and decoration
commands. The adapter retains each run's actual face and charges the resource
once per invocation, including when several faces use one collection. It
consumes each run's executed `GlyphFrame` with its physical origin, outline axes
and independent pen direction in outline coordinates.
Native upright runs keep their outlines unrotated and place each glyph at
`(origin.x + dx, origin.y + pen + dy)`. Combined cells retain horizontal inner
pens and the outline frame supplied by Text inside their vertical outer line.
Clockwise and counterclockwise sideways runs receive their exact right-angle
basis. The counterclockwise matrix is
`[0,-1,1,0,origin.x,origin.y]`; its raw pen is converted into the actual glyph
basis before applying that matrix. Existing outline synthesis operates in glyph coordinates
before that basis; caller placement follows once. Decorations are already
physical and receive caller placement once. Text owns admission of the actual
shaping profile; font, outline and numeric refusals retain their existing owners.

## Preparation accounting

`FontData::prepare_glyph_run` and `prepare_text_layout` take a mutable
`RenderAccountingContext` after their existing limits. Reuse the context to
change `RenderAccountingOptions` for the next call, inspect `last_attempt`, or
move its report to caller custody with `take_last_attempt`. This is a deliberate
source API change; every preparation path requires the context.

The measured boundary is standalone glyph and placed-Text preparation, including
outline/variation qualification, provider callbacks, intrinsic color/bitmap
processing, PNG decoding and coordinate conversion. The complete named
`RenderWork`/`RenderResource` vocabulary also reserves later scene/GPU dimensions;
those remain zero in preparation reports, and `RenderLimits::with_work_limit`
rejects their configuration with `InvalidLimit`. Text-only dimensions also
require the `render-text` feature. Renderer, device-generation and
readback accounting are outside this preparation contract.

| Preparation work | Unit and boundary |
| --- | --- |
| `GlyphInputs`, `CoordinateInputs`, `TextRuns`, `TextDecorations` | Original glyphs, normalized coordinates, nonempty runs and decoration records accepted once. Whole-Text preflight owns admission; nested glyph runs never accept these again. |
| `GlyphVisits`, `FontRecordVisits`, `CoordinateVisits` | One actual glyph fact, directory/instance record, bitmap strike/index record or scalar/axis record inspected. Repeated traversals charge separately; early exits retain only the visited prefix. |
| `PathElementVisits`, `OutlinePointVisits`, `VariationTupleVisits`, `GlyphComponentVisits` | One raw contour endpoint/provider path element, point-flag record/point/delta, variation tuple or composite child edge inspected. Dependency internals are not reported as Render visits. |
| `PaintVisits`, `ResourceProbes` | One raw COLRv1 paint node, consumed recorded layer/retained paint (including repeated placement qualification), or shared-font identity comparison. |
| `PathElementsEmitted` | One retained glyph path element appended, including actual synthesis and clip expansion. This is the existing path-element limit unit. |
| `SerializedBytes`, `PngByteVisits`, `ImageTexelVisits` | IDAT payload bytes copied; chunk type/payload bytes processed by CRC plus scanline bytes unfiltered; decoded RGBA pixels inspected for transparency. |
| `PreparedTransferredBytes` | Explicit retained backing units transferred only when the complete prepared result returns successfully. This dimension cannot be caller capped. |

`RetainedCpuBytes` covers actual Vec capacities multiplied by element size,
font resources using Font's admitted retained-byte unit once per `ResourceId`,
immutable image payload lengths, and explicit fixed result units. A glyph run's
fixed unit is `size_of::<PreparedGlyphRunData>()`;
a Text scene's is `size_of::<PreparedTextScene>()`. Opaque Arc/header estimates
are excluded from both result units.
`ScratchCpuBytes` covers actual live qualifier/provider/coordinate/mapping/PNG
buffer capacities and the fixed inline miniz decoder state and its declared
Huffman arrays. Arc allocator headers, hash-table bucket overhead, dependency
internal allocations and process/RSS usage are outside these units.
The fixed accounting participant, bounded reserved report/profile storage and
typed error/control-plane storage are also outside admitted preparation backing.
Failure reporting therefore does not recursively consume an exhausted output cap.

Capacity growth previews the requested replacement without committing usage,
reserves fallibly, admits its observed actual capacity while the old backing is
still live, then drops and releases the old backing. Reservation failure creates
no acquired peak. All temporary holds release at drop. Failed results discard
their retained output before closing custody; successful complete outputs transfer
their admitted backing to the caller. Transfer is not deallocation. Subsequent
caller clones and output lifetimes have no hidden accounting callbacks.

The fixed tree has `Attempt -> Preparation`, with `TextPreflight`, `GlyphPrepare`,
`Coordinates`, `Output` and `Cleanup` below Preparation. GlyphPrepare has
`Qualify`, `Outline`, `Color` and `Bitmap`; `Png` is below Bitmap. The general
limits apply at Attempt, and existing finite glyph/Text limits at Preparation.
They therefore cover all Text runs, preflight and glyph phases together, without
remaining-limit arithmetic. CPU resource custody originates at Preparation;
work and elapsed phases retain their named origin. Phase usage is inclusive:
never add child values to ancestor values. Phase durations may overlap and do
not sum to wall time or measure GPU execution time.

Defaults are checked-unlimited general limits with `RenderProfile::Off`. Off and
untimed profiles perform no clock reads. Timed profiles use Ledger's platform
clock or an injected `RenderClock::Host` callback; unavailable timing refuses
setup. Detailed profiles require an explicit event bound and optional phase
filter. Filtering and dropped detail never change authoritative totals or peaks.

One report slot retains the latest started attempt as `Succeeded`, `Refused`,
`Failed(ErrorCode)` or `Canceled`, with exact accepted-prefix usage, cleaned
closing balances and first typed accounting refusal. All bounded report storage
is reserved before starting; setup failure preserves the previous report. Legacy
glyph/Text cap refusals preserve their typed resources and actual item locations.
Nested Text path refusals report whole-attempt required/limit values: two
five-element runs under a cap of nine report `required: 10, limit: 9` at the
second run, in place of the former remaining-run values `5, 4`.
These attempted reports are separate from last-successful-frame `Stats`.

## Outline admission

The outline branch of `FontData::prepare_glyph_run` retains one canonical path
for measured ink and GPU drawing. Static glyf composites admit signed XY translations, including
explicit identity transforms, complete opaque unhinted programs and first-record
overlap flags. Their full original child closure is checked before provider draw;
repeated children retain every occurrence. Provider depth32 and real expanded
points65531 (plus four phantom points) are actual provider boundaries. Empty loca
children contribute no real points to metadata, but their actual draw-time load
still consumes a recursion level. Cycles, truncated fields and
malformed simple extents refuse as invalid font data.

Composite variation, point attachment, nonidentity transforms, metric overrides,
unknown flags and nonzero composite residual bytes remain unsupported. These are
supported-domain limits, not assertions that every refused font is malformed.
Simple residual bytes after fully proved consumed extents remain uninterpreted.
Valid simple encodings whose complete flag prefix exceeds their logical point
count refuse as `Provider`: the installed decoder cannot faithfully consume them.
A repeat count of zero is accepted when the complete prefix fits that domain.

Existing storage limits include live qualifier buffers and their growth overlap,
then the provider's actual draw-buffer capacity after qualifier disposal. They do
not bound CPU time. All failures discard the unpublished run, while the original
font, instance and glyph inputs remain available for retry. Other outline formats and
sparse IUP retain their explicit capability refusals. Concrete
positive bold appends a local-x copy before canonical point conversion under the
[selected policy](../../../docs/text-layout-contract.md#concrete-outline-synthesis);
a displacement absorbed by conversion refuses with a synthesis numeric error.

## Verification sources and commands

The focused tests are grouped in [src/tests/mod.rs](../src/tests/mod.rs), covering
models, authored-style normalization, frame planning, GPU passes, Vello raster
execution, surfaces/readback, and platform contracts. The [Ahem
fixture](../tests/fixtures/fonts/ahem/PROVENANCE.md) provides deterministic glyph
inputs. CPU pixel reference algorithms are compiled only for tests.

The repository command inventory is owned by [AGENTS.md](../AGENTS.md). With
dependencies already present, its standard Cargo commands can run offline:

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-render
CARGO_NET_OFFLINE=true cargo test -p surgeist-render
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-render --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo fmt -p surgeist-render --check
```

Tests include real GPU work. No passing result is implied when the required
adapter is unavailable. The [how-to guide](how-to.md) owns the live native
presentation and wasm compilation procedures; the [getting-started
guide](getting-started.md) selects one focused headless proof.
