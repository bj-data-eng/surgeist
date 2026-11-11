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
| Gradient paints | `Gradient`, `RepeatingGradient`, `GradientStop`, `GradientHint`, `Paint` | [Paint sources](../src/paint.rs), [repeating inputs](../src/paint/repeating.rs) |
| Native Paint consumer (`render-paint`) | `prepare_linear_gradient`, `prepare_radial_gradient`, `prepare_repeating_linear_gradient`, `prepare_repeating_radial_gradient`, `PaintGradientError` | [Adapter](../src/paint_gradient.rs) |
| Native zero-height Paint colors (`render-paint`) | `prepare_zero_height_gradient_color` | [Color preparation](#zero-height-paint-colors) |
| Placed Text consumer (`render-text`) | `prepare_text_layout`, `PreparedTextScene`, `TextLayoutLimits`, `TextLayoutError` | [Adapter](../src/text_layout.rs) |
| Surface and pixels | `Surface`, `SurfaceOptions`, `Attachment`, `ImageBuffer` | [Surfaces](../src/surface.rs), [images](../src/image.rs) |
| Support and diagnostics | `Capabilities`, `RuntimeCapabilities`, `Error`, `ErrorCode` | [Capabilities](../src/capability.rs), [errors](../src/error.rs) |
| Successful frame observations | `Stats`, `RenderRoute`, `EffectPrecision` | [Statistics](../src/stats.rs) |
| Frame and lifetime accounting | `RenderFrameReport`, `RenderCpuSnapshot`, `RenderDeviceSnapshot`, named scopes and typed intervals | [Frame accounting](#frame-accounting), [generation custody](#device-generation-custody) |
| Preparation accounting | `RenderAccountingContext`, `RenderStandaloneReport`, `RenderAccountingOptions`, `RenderLimits`, `RenderAttemptReport`, named work/resources/phases | [Accounting](../src/accounting/mod.rs), [units and reports](../src/accounting/model.rs) |
| Current style-facing models | Background, border, outline, image-placement, mask, clip, and filter values | [Style module](../src/style/mod.rs), [public reexports](../src/lib.rs) |

The `Renderer` methods `new`, `create_surface`, `create_headless`,
`resume_surface`, `render`, and `read_headless` are asynchronous GPU operations.
`Renderer::read_headless` returns an
`ImageBuffer` containing tightly packed straight-alpha RGBA8 physical pixels from
the current complete headless publication. Rendering does not invoke this
readback implicitly.

## Qualified Scene inputs

[`Layer::with_transform`](../src/layer.rs) and
[`Scene::transform`](../src/scene.rs) consume a finite
[`Transform`](../src/geometry.rs) infallibly. Singular finite transforms are valid;
composing transforms remains fallible when the resulting coefficients overflow.

[`TextDecorationLine`](../src/text.rs) validates its endpoints, thickness and other
inputs at construction and retains its qualified solid stroke. Its thickness
reader and `Scene::text_decoration_line` consume that accepted value directly.
Invalid raw thickness still returns the owning text-decoration input diagnostic.
Identity transforms lower directly to a stroke; other transforms retain the
nested layer and authored command order.

## Capabilities and errors

`Capabilities::CURRENT` reports semantic support for authored rendering
operations and compiled surface availability. Its surface report supports
headless construction and enables its web-canvas flag only with both
`render-web` and the `wasm32` architecture. Native window attachment is separately
compiled behind `render-window`.

`Renderer::runtime_capabilities` reports availability or immutable facts about
the selected device and surface: surface format, supported effect precisions,
and maximum two-dimensional effect texture size. Cargo features select compiled
host and producer adapters; enabling a feature does not establish runtime device support.

Unsupported semantic operations and unavailable runtime capabilities have
distinct typed diagnostics. Missing adapters, required formats, device limits,
and surface lifecycle availability do not authorize partial frame publication.
The [error API](../src/error.rs) defines the operation and reason payloads.

## Native geometry realization

A valid finite scene can exceed the native geometry representation. Render
qualifies the actual path coordinates, transforms, required stroke/blur values
and glyph placement factors before submission. `ErrorCode::GeometryRealization`
and `Error::geometry_realization_diagnostic()` distinguish that contextual limit
from invalid caller input, unsupported semantic operations and device failures.
`GeometryRealizationError` identifies an unrepresentable transform, path element,
stroke, glyph occurrence or blur value, or insufficient native proof capacity.
Path and glyph indices are local to the receiving input sequence.
Font and intrinsic glyph-preparation errors retain their preparation domain;
gradient field refusals retain `GradientRealization`.
Standalone placed-Text preparation meters its own placement validation and
retains its glyph numeric diagnostics. Runtime frame encoding performs the
separate contextual native receiving checks.

Separate narrowing can erase geometry even when its final device bounds are
finite. Examples include a huge local rectangle under a tiny scale, a translated
unit edge whose endpoints round together, and tiny local edges that the pinned
encoder eliminates before a large transform makes them visible. The shared
receiver checks the accepted native factors and explicit segment retention;
uncertifiable rank, composed arithmetic and subnormal behavior refuse. It encodes
the accepted representation and preserves the existing native curve and coverage
approximation. Ordinary nonzero rounding remains supported; this contract adds
no exact source-f64 raster, arbitrary winding or affine-boundary tie guarantee.

The receiver also bounds source-to-native device placement. It admits at most
0.25 device pixel of Euclidean point displacement from coordinate/factor
narrowing and permitted GPU point arithmetic. Outward bounds compare the source
and actual native factors without evaluating a cancellation-sensitive source
sum. This is a separate receiving capacity policy; the backend's curve and
coverage approximation remains additive. The policy does not promise identical
pixels at affine boundaries. A finite map whose separately narrowed factors
move the geometry beyond this budget refuses as insufficient native capacity.

Surviving image and blurred-rectangle brushes also require qualified native
inverse arithmetic. A forward matrix can retain rank in widened arithmetic while
its actual GPU determinant underflows; that is insufficient native capacity.
The proof covers reassociated division/product forms and propagates bounded
zero neighborhoods and subnormal flushing. Required nonzero factors must not
collapse. It also encloses finite image/blur mapping across the raster format's
padded device-coordinate domain, including permitted upward rounding of the lane
and pixel-center additions, without adding a color-accuracy guarantee.
Gradient field precision and its diagnostics remain with the gradient owner.

Qualification follows existing semantic elimination. Empty paths and glyphs,
true point degeneracy, zero-width strokes, empty clips and eliminated subtrees
retain their behavior and do not acquire requirements from unused geometry or
resources. Fill, stroke, clip, image, blur and prepared/native glyph paths share
the receiving checks. Gradient field qualification remains a separate owner.

For native glyphs, the default scaled font provider can lose an entire small
outline before path encoding. An empty scaled result therefore receives a
qualification-only check of the same authentic unscaled outline. A nonempty
source refuses at its original glyph index; true spaces retain their empty
behavior. The witness does not change the native drawing size or path style.

Ledger admits actual conversion, retention, rank and composition work, including
repeated walks and copies. Real backing is admitted before private stream writes.
A refused frame releases unpublished custody, submits nothing and preserves
previous pixels and successful statistics; a fresh valid attempt can retry.
The [geometry contribution](https://github.com/bj-data-eng/surgeist/issues/2111)
records the receiving scope and precision boundaries.

## Path stroke support bounds

Path bounds include the centerline bounding box and the selected stroke support.
For width `w`, round and bevel joins require half-width support. A square cap at
either end can project up to `w / sqrt(2)` onto an axis. Miter joins also require
`(w / 2) * max(miter_limit, 1)` support. The bounds use the greatest applicable
support. Square-cap support is rounded outward; only miter joins use the
configured miter limit.
Rotations and nonuniform transforms preserve coverage of these local bounds.

This conservative calculation uses the fixed stroke style without another path
walk or outline materialization. An unrepresentable bounds rectangle returns
typed `InvalidInput` before publication; native stroke receiving retains its
separate `GeometryRealization` capacity checks. The
[bounds owner](../src/command.rs) and
[public rendering regressions](../tests/stroke_bounds.rs) cover these boundaries.

## Native stroke dash cycles

`Dash` retains a finite authored offset and finite nonnegative interval values.
Its constructor owns that intrinsic validity; stroke construction and scene
normalization do not repeat the immutable interval validation.

After source-path elimination, the native receiver qualifies the actual copied
pattern before entering the locked dash initializer. It checks the sequential
period sum, odd-pattern doubling, canonical remainder and phase-search arithmetic, and
certifies completion within two finite traversals. A required cycle that cannot
qualify returns `GeometryRealizationError::InsufficientNativeCapacity` before
submission. The original lengths and offset reach the dependency unchanged;
closed stroked paths do not receive fitted border spacing.
In particular, a negative small offset whose remainder rounds to the complete
finite period refuses rather than silently losing its required initial gap.

Empty and all-zero patterns retain solid-stroke behavior. A positive subnormal
cycle is not reclassified as solid. The existing path materialization tolerance
is retained: `0.1` for empty patterns and `0.01` for nonempty patterns, including
all-zero patterns. Source and post-dash segment elimination both use the shared
geometry cursor. This initialization certificate does not assert a bound on
every later dependency-internal curve operation; emitted output remains subject
to the ordinary admitted path-materialization limits.

Owned scalar proof, interval conversion, source/output visits and actual backing
are accounted separately. `DashInitializations` counts one qualified opaque
initializer actually entered, admitted immediately before entry. Its dependency
arithmetic is an indivisible operation, rather than fabricated interruptible
scalar prefixes. Solid or eliminated sources enter no initializer. The work
limit applies to frames; standalone preparation and readback do not produce it.
Adding this unit also adds a required field to the public `RenderWorkTable`.

## SDR sRGB color preparation

`prepare_sdr_srgb_color` accepts a resolved `surgeist_color::Color` for a
caller-selected SDR sRGB destination. It returns the straight numeric-sRGB
`Color` accepted by existing scene paints. In-gamut channels pass through.
Out-of-gamut colors use [CSS Color 4 binary search with Local MINDE][local-minde].
Missing components contribute zero for rendering. Mapping preserves alpha.

This standalone operation records an `SdrColor` attempt with timed preparation
and coordinate phases when timing is enabled. It admits each color conversion
as three coordinate visits and accounts for the scalar mapping and output work.
The caller supplies an accounting context and can reuse it after a refusal.

[local-minde]: https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#css-gamut-mapping

## Gradient paints

`Gradient` supplies ordinary linear, positive elliptical radial, horizontal-distance
and full-turn sweep fields. `GradientStop::try_new` preserves any finite `f64` field position
and a validated numeric-sRGB color. Linear and radial constructors retain a
nonempty, nondecreasing list, including negative, outside-unit and equal
positions. Radial construction takes horizontal and vertical radii; equal axes
represent a circle. Sweep retains normalized `0..=1` positions and supplied order.
`RepeatingGradient` shares the stop primitive while additionally requiring a
finite last-minus-first period and allowing its selected zero-height average.
A single stop is a solid paint, including for coincident linear endpoints.

Ordinary fields use first/last colors outside the supplied interval. A multicolor
all-coincident list is a true step: first below the threshold, last at and above
it. This equality rule is deterministic native policy. Source equality is tested
in `f64` before normalization or narrowing. Nonzero linear/radial intervals map
to a canonical native range with exact endpoint records zero and one.
Horizontal-distance fields retain unshifted controls as described below.
Steps use an unshifted,
qualified comparison basis and directed threshold rounding, preserving subnormal
thresholds. These rules compare the actual native computed coordinate; they do
not promise exact-real geometric boundary coverage.

Unhinted linear/radial/sweep interpolation retains the native 512-sample RGBA8 ramp. Hinted lists
use the [analytic segment program](#transition-hints) below. Realization uses
the full paint transform and final target,
including shear and elliptical axes. Negative radial stops can contribute to the
center by interpolation. No repeating phase reduction or small-period averaging
applies to ordinary fields. Finite source endpoints do not require a finite
subtracted span: scaled whole-range arithmetic can represent an overflowing
source interval when its mapped geometry is finite.

Sweep realization qualifies the actual native paint transform after path
representation selection or deferred glyph composition. Its narrowed center is
included in the shader's transform composition before determinant, reciprocal,
inverse matrix and translation qualification. Sampling bounds include the final
target's padded raster lanes and permitted subnormal flushing. An inverse that
can lose a required direction is refused before native brush encoding. This
preserves the existing angle approximation and center handling; it adds no exact
source-angle or raster-boundary guarantee. A single-stop solid or unused draw
does not require a sweep inverse.

`ErrorCode::GradientRealization` and `Error::gradient_realization_diagnostic`
distinguish degenerate required linear fields, unrepresentable geometry, a stop
whose required native interval collapses, and insufficient native arithmetic
capacity. `UnrepresentableHint` identifies a hint control or exponent that cannot
retain its required native meaning. These are contextual capability refusals for
valid gradients. Actual
resource limits retain Ledger diagnostics. A refusal preserves prior successful
pixels/statistics and releases admitted backing. Submit `Gradient` through `Scene`
for this execution contract; no infallible export to an external native sampler
carries its endpoint/comparison semantics.

For repetition, positions are units of the supplied field: the linear start-to-end vector or
radial ending ellipse; horizontal-distance stops use physical logical distances.
A linear field from zero to forty logical units with
stops at one-quarter and one-and-one-quarter therefore repeats a forty-unit
cycle beginning at ten, including beyond the original field endpoints. Period
is last minus first, with no box or unit-interval clamp. Duplicate positions
retain their supplied order; cycle boundaries preserve the end/start jump.
Radial phase preserves the center and concentric ellipses.

Native gradient color is sampled at integer physical pixel indices in the
current raster engine, consistently with ordinary gradients. Geometry coverage
is a separate operation. Linear realization preserves the transformed field
normal as well as its phase under shear.

At each draw, realization includes the complete paint transform and surface
scale. Native policy averages a positive period strictly smaller than two
device pixels; exactly two remains a pattern. Linear spacing uses the field
normal's inverse-transpose mapping under shear. Radial spacing uses the smallest
singular value of the transformed period axes, so an unresolved direction
averages the whole paint. These thresholds and directional rules are Render
policy, not constants imposed by the pinned standard.

Solid fallback averages adjacent premultiplied-sRGBA endpoints by their
relative segment lengths, excluding hints. This is the specified stop-only
trapezoid average, even when a hint changes the resolved curve. A zero or
actually rounded-zero whole period instead
uses the original colors/count at uniform replacement positions. Coincident
linear endpoints select that case. `try_radial` requires a positive horizontal
radius and nonnegative vertical radius: positive-width zero-height is an
already-selected weighted-average case. Zero-width realization uses the distinct
[horizontal-distance field](#horizontal-distance-fields). The [pinned repeating-gradient clauses](../../../references/css-images-3--CRD-css-images-3-20231218--74eb4b475aa0.md#repeating-gradients)
own repetition and averaging requirements.

Unit native patterns and brush transforms retain period/phase before narrowing.
Actual period construction collapse is distinct from a small positive period;
other unrepresentable required native geometry produces a typed diagnostic.
Repeating-field qualification compares the realized linear or radial field with its
intended device-space reference over the actual target's integer sample rectangle.
The linear reference is retained before local compensation; native glyph encoding
checks the actual separately narrowed transform product against that reference.
Both phase granularity and conservative arithmetic error must fit within one
existing native ramp interval. The certificate encloses permitted operations under
[published shader accuracy rules](https://www.w3.org/TR/2026/CRD-WGSL-20261106/#floating-point-accuracy),
including division domains and subnormal flushing. Circular sampling selects zero
for zero/subnormal squared norms and evaluates its square root on a normal positive
argument; the certificate includes this lost norm. A resolved cycle that exceeds
numerical capacity returns typed invalid input before publication, preserving
successful pixels and statistics. This conservative policy can reject individual
samples that happen to look valid.
Solid fallback needs no unused pattern backing. Repeating paint works through
the existing fill, stroke and text paint consumers, preserving glyph/run-local
coordinates. Graph captures use the output grid for this source, so a mask or
nested capture preserves a pattern resolved by final magnification.

`RenderPhase::GradientPrepare` is a Ledger child of Encode for ordinary and
repeating brush preparation, sweep receiving qualification and native glyph
representation checks. Actual stop
walks, coordinate work, backing, serialization and ramp resources retain their
existing named work and lifetime owners; optional elapsed time uses Ledger.
Ordinary multistop preparation charges actual endpoint inspection, range and
comparison qualification, scalar/color conversion and every materialized native
record. Intrinsic constructor proofs avoid redundant later validation walks. Encode separately charges native record visits, emitted-record capture,
final draw-stream association and descriptor/ramp serialization. Actual backing,
analytic program records and their captured copies, row padding and upload
resources retain their existing lifetime owners. CPU coordinate visits measure
preparation; shader evaluations remain GPU work.
See [frame accounting](#frame-accounting) for inclusive phases and host timing.
`supports_repeating_gradients` reports this concrete resolved domain. `surgeist-paint`
retains authored gradient-box geometry, stop/hint fixup and degenerate semantic
selection. Render owns its direct native Paint consumer; root retains facade
wiring and broader authored composition.

### Horizontal-distance fields

`Gradient::try_horizontal_distance(origin_x, stops)` and
`RepeatingGradient::try_horizontal_distance(origin_x, stops)` define the physical
logical-distance field `r = abs(local_x - origin_x)`. They require a finite
origin and the existing ordered finite color/hint list; repeating construction
also requires a finite last-minus-first period. Both types expose
`horizontal_distance_origin()`. Their other geometry getters return `None`.
These constructors work without `render-paint` and retain no unused y origin
or fictitious ending ellipse.

The fold occurs before source offsets and repetition. Ordinary red at distance
2 and blue at 6 is red at the center, half red/blue on both sides at distance 4,
and blue at distance 6 and beyond. Applying `abs` after subtracting the first
stop would move those colors. Repeating red at 1 and blue at 5 has period 4:
the center's blue weight is .75, both distance-1 seams start red, and distance 3
is half red/blue. The field is constant along local y before the complete paint
transform. Rotation, reflection, shear and surface scale act on that same field.

The native [ZeroWidth contribution](https://github.com/bj-data-eng/surgeist/issues/2105)
uses Paint's authentic case and used stops at basis zero. Percentage contributions
vanish while length contributions, hints, omissions and source order remain real.
Both-zero ellipses retain the ordered `ZeroWidth` case. A zero-radius circle has
a separate infinitesimal-boundary question and is outside this consumer.

One-color lists are solid. Ordinary last positions at or below zero are solid
last color on the nonnegative distance domain. A coincident positive threshold
uses first color inside and last at and outside the two mirrored boundaries.
For repetition, zero spans use uniform replacement over actual colors; positive
unresolved spans use the specified stop-only weighted average. Local stripe
spacing under a transform follows the inverse-transpose field normal, including
shear. The existing strictly-below-two-device-pixel rule applies on both halves.

Nonzero ordinary distance intervals use an analytic program even without hints.
An unshifted signed linear carrier is folded by its sign bit before endpoint
tests. A power-of-two scale bounds accepted native controls without requiring
an overflowing raw span or a range offset before the fold. Repetition uses its
physical period as scale, then subtracts the retained phase and wraps once.
Unrepresentable controls/hints and uncertifiable phase/geometry retain typed
capability refusals; numerical difficulty does not select a different average.
Steps retain directed threshold rounding and their comparison-only finite
certificate, including native subnormal threshold bits.
The field's bounded final-coefficient proof covers its carrier and sampling.
Complete scenes also pass [native geometry realization](#native-geometry-realization)
and retain their finite inverse-domain constraints; a qualified carrier does not
certify separately narrowed path coordinates or transforms.

Qualification checks the complete final target and each actual glyph transform
product. Smooth field error is measured relative to the accepted stop span,
or in periods for repetition. The fold is exact on native representations and
does not increase coordinate error at the cusp. Analytic weight error is a
separate bound about the lowered curve; neither bound promises aggregate exact
source-real colors at arbitrary affine discontinuities. Exact native endpoint,
run and seam rules still apply, including the existing rounded-wrap endpoint.

The coupled sampling interpretation survives prepared/direct/graph/native glyph
copies. Its one eight-word descriptor and seven-word program records use the
existing exact RGBA8 transport. Ledger counts actual field/phase proofs, record
walks, carrier and program capacity, overlapping scene copies, checked packing,
texture rows and uploads. Header costs also reflect the eight-word format for
existing fields. Refused preparation releases unpublished backing and preserves
the accepted prefix; later frame/device publication remains atomic.

### Transition hints

Create a finite `GradientHint` with `GradientHint::try_new(position)` and attach
it to its left color with `GradientStop::with_hint_after`. `hint_after` returns
that outgoing hint. Its position uses the same field units as the color stops.
List construction requires a following color and inclusive containment between
the two ordered colors. `GradientHintError` distinguishes `TrailingHint`,
`HintOutsideInterval` and `UnsupportedSweep`, each with its owning `stop_index`.
`ErrorCode::GradientHint` and `Error::gradient_hint_diagnostic` expose this
intrinsic input diagnosis. Sweep rejects its first outgoing hint; its unhinted
behavior is unchanged.

For different positions `a < b` and an interior hint `h`, let
`P = (q-a)/(b-a)` and `H = (h-a)/(b-a)`. The interpolation weight is
`P.powf(ln(.5)/ln(H))` in premultiplied numeric-sRGBA. Thus a quarter-interval
hint gives half the premultiplied color at that quarter, and three quarters
at `P = 9/16`. Hints contribute no additional colors.

Native endpoint policy completes the singular limits explicitly. A hint at
`a` selects the left color at `a` and the right color in the open interval.
A hint at `b` retains the left color throughout the open interval and selects
the right at `b`. Source equalities are classified before division or narrowing;
zero-width intervals have no power computation. Interior coincident runs select
their last source record, while the existing whole-field endpoint and
all-coincident step rules above retain precedence. These singular completions
are native policy, rather than a claimed standards resolution.

Per-use lowering preserves source equality and strict ordering in accepted
binary32 controls. An interior hint that collapses onto a neighbor is refused
as `UnrepresentableHint`; it never becomes an endpoint tag by rounding. The
analytic reference is the ideal power curve through those accepted native
controls, evaluated at the actual finite native field coordinate. It does not
promise original-`f64` interior colors or exact-real pixel ties.

A bounded [coefficient calculation and whole-segment certificate](../src/encode/hint_program.rs) enclose
division, subnormal flushing, coefficient rounding, `log2`, multiplication and
`exp2`. Accepted power segments have weight error at most `1/511`, before
premultiplied blending and output rounding. This bound is separate from the
field-coordinate qualification above. An uncertified segment returns
`InsufficientNativeCapacity`. Endpoint tags, solid paints, coincident steps and
repeating averages avoid unused logarithms and program backing. No artificial
minimum hint fraction, substituted color stops or small transition span is
introduced.

## Native Paint consumer

The optional `render-paint` feature enables `prepare_linear_gradient` and
`prepare_radial_gradient`, returning an owned ordinary `Gradient`, and
`prepare_repeating_linear_gradient` and `prepare_repeating_radial_gradient`,
returning `RepeatingGradient`. Each takes real Paint geometry,
`GradientStops<Color>` from its shared resolver, and a mutable
`RenderAccountingContext`. Calling a repeating front door explicitly selects
repetition. The output works with normal fill, stroke and glyph commands after
borrowed inputs and the preparation context are dropped.

Linear geometry must have positive length. For the positive radial Ellipse case,
its horizontal radius supplies the ray basis. The stop result's
`line_length` must exactly equal that accepted `f64` basis. Each used physical
position `p` becomes the field position `q = p / L`, retaining equal, double,
negative and outside positions in source order. The conversion preserves all
stop occurrences, colors and outgoing hints. A hint following a double-position
stop attaches after its second position; a following double-position stop begins
the next interval at its first position. The consumer does not rerun direction,
geometry or fixup. For radial `ZeroWidth`, the accepted basis is exactly zero:
the consumer captures the required center x and copies used physical positions
identically into a horizontal-distance field, with no division. Unused center y
and original height are not retained. The same source/Hint/Two associations apply.

Native qualification requires finite `q`, preserves nonzero positions and the
equality or strict order of adjacent positions, and refuses a collapsed or
reversed interval. Ordinary nonzero floating-point division rounding is allowed;
there is no epsilon or exact back-mapping guarantee. For repetition, the converted last-minus-first
span must also be finite; ordinary conversion has no such restriction. An overflowing physical span is allowed when conversion
produces a valid finite field span.

`ErrorCode::PaintGradient` and `Error::paint_gradient_diagnostic` preserve typed
basis, unsupported geometry, source/position conversion, span and storage
diagnostics. Named-limit and setup failures retain `RenderAccountingError`.
Positive-basis conversion qualifies hint positions as well as color positions, preserving the
actual source item index; a hint has position ordinal zero. `ZeroCircle` and
`ZeroHeight` remain refused by these field adapters; no substitute radius is inferred.
Authored/computed adapters remain outside these native consumers.

`RenderAttemptKind::PaintGradient` reports one standalone preparation. Ledger
records actual geometry/result/item visits and scalar work, one actual
stop-vector capacity and the fixed returned gradient size. Qualified construction
consumes the adapter's invariant proof without another stop traversal. Borrowed inputs are outside its custody. Failure returns no field
and releases admitted backing; success records `PreparedTransferredBytes` and
releases standalone custody as output transfers to the caller. Later clones and
drops are caller-owned. Optional timing and filtering use the existing Ledger
profile. Per-use `GradientPrepare`, native qualification and GPU publication
still run later in the normal frame attempt.

For this consumer, `PaintVisits` counts inspections of geometry, the stop owner
and the constructed native gradient. `GradientStopVisits` counts every actual
record in the full preflight and conversion: 2n visits for n prepared records.
Each visit is admitted before projecting Paint's borrowed record view. Conversion
starts with the mandatory typed first color, then consumes the tail; hints update
that pending color, and following colors emit and replace it. Endpoint and final
pending state remain required values. `CoordinateVisits` counts each
actual scalar inspection or calculation, including comparisons. Repeated passes
are charged separately; a refusal retains only admitted work.

## Zero-height Paint colors

With `render-paint`, `prepare_zero_height_gradient_color` consumes an authentic
Paint `ZeroHeightGradientColors<Color>` and a `RenderAccountingContext`. It
returns owned `Color`, ready for normal solid fill, stroke or glyph paint after
the source, measure and contexts have been dropped.

Paint owns the [zero-height basis and color weights](../../surgeist-paint/docs/reference.md#zero-height-gradient-colors),
including mixed-unit fixup, omission/hint distribution and repetition selection.
Render uses Paint's actual `selected_last_color()` selection to copy Last
directly, preserving straight RGB even at zero alpha. Average consumes the real
borrowed weighted records, including zero-weight occurrences, in premultiplied
numeric-sRGBA with the same arithmetic owner as existing repeating fallbacks.
An average with zero alpha produces transparent output; nonzero alpha is
converted back to straight channels with the existing final color rounding.
The positive-field adapters retain their geometry/used-position contracts.

Preparation uses a standalone `RenderAttemptKind::PaintGradient` report and
existing named Ledger work, phases, limits and optional timings. Actual record
visits and weight/channel work count once per performed action. Input storage
is borrowed; the fixed `Color` result is admitted and explicitly transferred to
the caller. Refusal publishes no partial output and releases admitted custody.
Later solid rendering and atomic frame/device publication use their usual owners.

## Layer blending

`Layer::blend` accepts `BlendMode::{Normal, Multiply, Screen, Overlay, Darken,
Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion,
Hue, Saturation, Color, Luminosity}`. The first twelve functions combine
corresponding straight numeric-sRGB source and backdrop components independently;
the final four operate jointly on RGB. The result is bounded to the component range and
modulated by backdrop alpha before source-over composition. Transparent
backdrops therefore preserve the source contribution. Inputs, intermediate
textures and readback keep their existing alpha conventions; public readback
is straight-alpha RGBA8.

Every direct raster policy bounds the mixing result before backdrop-alpha
modulation, including capture-local groups with a live local Plus source.
Local Plus retains its separate additive composition arithmetic.

Color dodge returns zero for a zero backdrop before considering a source of
one. Color burn returns one for a backdrop of one before considering a source
of zero. Hard light selects its curve at a source component of one half.
Overlay uses the backdrop component for that selection. Soft light uses its
polynomial curve through a backdrop of one quarter, then its square-root curve.

Hue preserves backdrop saturation and luminosity with source hue. Saturation
preserves backdrop hue and luminosity with source saturation. Color preserves
backdrop luminosity with source hue and saturation; luminosity preserves
backdrop hue and saturation with source luminosity. These use
`Lum(C)=0.3R+0.59G+0.11B` and `Sat(C)=max(C)-min(C)`, rather than HSL lightness.
Joint gamut correction follows the complete luminosity/saturation sequence;
clamping individual channels before that sequence would change the result.
Equal-channel gray uses zero saturation without an epsilon cutoff.
Saturation preserves the original RGB channel positions.
Luminosity adjustment selects exact gray, black-target and white-target
results and supplies neutral clipping input for those cases. Interior non-gray
colors use the gamut correction equations.

`BlendMode::Plus` remains a separate local additive composition choice. It cannot
serve as the mixing function of a basic Porter-Duff pair. Background-layer blend
interpretation and its typed capability boundary remain separate from this
supported layer set. `CompositingCapabilities::supports_blend_modes` describes
the complete concrete layer choices.

Effect-free layers use the direct Vello route. A resolved image mask or other
supported graph requirement selects the GPU graph without changing the selected
blend function; nested capture-local layers can still use Vello raster spans.
Clipping, mask alpha and opacity attenuate the premultiplied source before
composition. Both routes retain the same Ledger admission, resource custody and
atomic publication rules as other layers. Host-observed profiling durations do
not establish pixel-value-independent execution timing.

`Layer::with_resolved_luminance_mask` consumes straight sRGB RGBA8 texels
mapped to layer-local bounds. `MaskLuminanceSpace` selects encoded-sRGB or
linear-sRGB luminance. The GPU multiplies each texel's luminance by alpha
before coverage interpolation. Resolved alpha and luminance masks both use
`supports_resolved_alpha_mask_execution`. Authored mask interpretation uses
the separate `supports_luminance_mask_mode` capability. [CSS Masking 1, mask processing](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#MaskValues)

`Scene::masked_group` applies resolved mask coverage to isolated children.
Supply `ResolvedMaskLayer` entries from top to bottom. Render starts with
the bottom layer's coverage and combines each higher layer with the result.
`ResolvedMaskSource::Transparent` participates with zero coverage. An empty
stack appends the children directly. The resolved path uses the existing
GPU coverage and Porter–Duff capabilities. [CSS Masking 1, mask compositing](https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#the-mask-composite)

## Layer composition and isolation

`LayerComposition::try_porter_duff(operator, blend)` pairs one of the twelve basic
`PorterDuffOperator` choices with a mixing function. It rejects `BlendMode::Plus`
with typed invalid input because Plus is an entire local operation. Install the
pair with `Layer::with_composition` and inspect it with `Layer::composition`.
`Layer::blend` replaces the entire composition with source-over mixing or local
Plus, including a previously selected operator. There are no independent setter
states in which Plus also carries a basic operator.

The basic operators are Clear, Copy, Destination, SourceOver, DestinationOver,
SourceIn, DestinationIn, SourceOut, DestinationOut, SourceAtop, DestinationAtop
and Xor. Blending first computes the joint or separable bounded `B(Cb,Cs)` on
straight RGB, then `Cs'=(1-ab)Cs+ab*B`. The selected factors produce premultiplied
`co=as*Fa*Cs'+ab*Fb*Cb` and `ao=as*Fa+ab*Fb`. Mixing does not change output alpha.
Straight color has no meaningful mathematical value when `ao=0`.

`LayerComposition::try_porter_duff(PorterDuffOperator::PlusLighter, BlendMode::Normal)`
selects saturating additive composition from
[Compositing and Blending Level 2](https://drafts.csswg.org/compositing/#porterduffcompositingoperators_plus_lighter).
It rejects every other mixing choice. The graph sums premultiplied source and
backdrop, capping each RGB channel and alpha independently at one. Layer clip
coverage, mask alpha and opacity attenuate the source before the sum.
Transparent source samples preserve the normalized backdrop. An isolated group
can use this operation for paired cross-fades over its own transparent
starting image.

A basic operation runs over the finite current parent or group image in parent
coordinates. Source samples outside source support are transparent. Masks and
opacity attenuate that source; active operation clips instead interpolate
`(1-t)*parent+t*operator_result`. Copy with zero source opacity therefore clears
its operation domain, while zero clip coverage preserves the parent. Empty
children and zero-sized source masks do not remove destructive operations.
Source support, affected domain and result support remain distinct planning facts.
These are resolved native execution rules; callers still own CSS, Canvas or SVG
interpretation and group creation.

Graph clip-coverage captures use the composed parent image's spatial grid,
including its signed device origin, extent, raster scale and texel centers.
Coverage is evaluated over that complete image before composition. Source
support, clip geometry and alpha-mask image dimensions retain their separate
roles; the clip path's own bounds do not select another capture grid.

Layer transforms map source content and the layer's own clip. They do not remove
the parent image domain. A singular source transform yields transparent source;
an unclipped destructive operation still runs. A singular own geometric clip has
zero operation coverage. Other active parent-space clips remain applicable.

`Layer::isolated` forces children to start over transparent black. Later children
see the accumulated internal result, and the outer composition consumes that
completed group. Default identity-looking wrappers can forward their surrounding
parent. Retained non-source-over operators and explicitly isolated groups use the
graph, including inside captures, with finite transparent group sources even
when no source ink survives. Existing source-over and local Plus routes remain
available. The graph retains sampled parent, source and replacement result as
distinct admitted resources through their proven last uses.

`CompositingCapabilities::supports_basic_porter_duff_modes` reports the twelve
basic choices. `supports_plus_lighter` reports saturating addition and
`supports_isolated_layers` reports explicit resolved groups.
The broad `supports_porter_duff_composite_modes` flag remains false: the pinned
lighter equations print unbounded color and alpha sums while another clause
requires normalized range. Existing local Plus behavior does not resolve that
normative conflict. Root-backdrop and broad backdrop-policy boundaries remain
separate. Ledger owns actual graph work, resource custody and optional timings;
refusal preserves the previous complete pixels and successful statistics.

The graph compositor and internal Vello fine programs share the color dodge,
color burn, soft light and nonseparable blend math. Blend mode selects the
shader operation. Color endpoints, channel order and gamut clipping select
values inside that operation. The shader structure is verified separately
from hardware execution timing.

## Foreground layer filters

`Layer::with_filter_list` installs a validated `FilterList` in authored
order, replacing the previous foreground effect. `FilterList::none()`
clears the effect. Each function takes the previous function's result.
The color functions operate in numeric sRGB with a clamp between
operations. Gaussian blur and drop shadow expand the affected bounds
using layer-local units and signed shadow offsets.

`Layer::try_filter(Filter::try_blur(sigma)?)` applies one scalar Gaussian
standard deviation in layer-local logical units. The authored constructor
accepts finite nonnegative values. Scene normalization admits `0..=256`
and reports `InvalidInput` above that range. Device and Ledger limits apply
to the resulting work and allocations.

Render captures the layer's completed children over transparent black.
Positive sigma uses horizontal and vertical RGBA Gaussian passes with
transparent-black sampling outside the input. The output includes the
filter halo wherever it can contribute to the parent. A blur with zero
device sigma still groups the children and skips both Gaussian passes.
A drop shadow with zero device sigma samples the unblurred source alpha.
Layer clipping, resolved alpha masking, opacity, and composition apply to the
group output. See the [layer composition contract](#layer-composition-and-isolation)
for the finite operation domain of destructive operators.

`FilterCapabilities::supports_layer_filters` and
`OffscreenPipelineCapabilities::supports_layer_filter_execution` report
these foreground functions. A frame may also use one bounded,
untransformed, top-level backdrop filter. It samples the completed prior
parent, including earlier foreground filter results. When a single layer
has both effects, its foreground list consumes the group formed by the clipped
backdrop and the children before the outer layer effects. Nested,
transformed, and repeated backdrop requests return the typed
`BroadBackdropExecution` diagnostic.

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

### Source-region draws

`Scene::image_region` maps a nonempty source rectangle in pixel coordinates
onto a destination rectangle in scene coordinates. Normalization checks the
source against the original image axes. The mapping keeps the fractional source
origin and extent. The atlas window contains the texel cells that intersect the
source rectangle, and sampling clamps to that window. For an integral source
rectangle, neighboring slices supply no samples.

The draw uses the image's selected quality and the existing per-axis Pixelated
rule. The GPU copies the source window to the atlas, applying integer
expansion when selected. The original CPU pixel backing is shared. Atlas
identity includes the window and expansion, so draws using the same image
window and expansion reuse one entry. Ledger counts the full original
backing staged for each distinct entry and the actual cropped or expanded
atlas texels. Source validation, atlas preparation and encoding can return
typed refusal before the frame publishes.

## Host features

| Configuration | Leaf boundary |
| --- | --- |
| Default features (`[]`) | Native headless GPU execution without a window. |
| `render-text` | Optional direct Text dependency, without Text default features; immutable placed-layout consumption, independent of window/web/accessibility adapters. |
| `render-paint` | Optional direct Paint dependency; native positive and zero-width geometry/used-stop consumption, independent of host adapters. |
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

## Frame accounting

Configure `RenderAccountingOptions` through `Options::with_accounting` before
`Renderer::new`. Limits, profile mode and clock are fixed for that Renderer and
reach lifetime owners before native initialization. Defaults are checked-unlimited
limits with `RenderProfile::Off`. A frame uses three disjoint Ledger owners:

| Domain | Scope and admission |
| --- | --- |
| Fresh frame attempt | Actual forward work, including normalization, repeated visits, raster streams, graph/filter work, native requests/probes, uploads, encoded passes/copies, submissions and surface operations. Work limits restart for each frame. Finite repeat and atlas quotas apply separately to each local call. |
| Renderer CPU lifetime | Actual working and persistent backing, history drafts, shader keys/indexes, and imported font/image/prepared-run payloads. CPU limits bound simultaneous live backing, including old/new growth overlap; they are not a fresh byte allowance per frame. |
| Device generation | Descriptor-defined GPU custody, opaque native handles, cache classifications and mandatory lifecycle work. This owner survives Ready-state destruction and retires once; there is no caller-configurable active GPU allocation cap. |

Work units describe actual visits and invocations. Unordered source traversal can
vary comparison/probe totals between otherwise equal inputs; those comparisons
remain measured in their owning domain.
Resource probes include complete registered-entry validation, idle summaries and
repeated explicit entry lookups as well as candidate selection. Forward registry
audits spend the current attempt's allowance before inspecting each entry; a
refusal preserves registry and lease state. Mandatory return, abort, removal and
summary walks use generation-lifetime observations and continue after a spent
forward allowance. Opaque container-internal comparisons are not estimated.
Both image-atlas routes charge the selected source bytes and texels before
building the atlas. Ordinary residency follows the producer's Blob identity;
pixelated residency additionally distinguishes the selected integer expansion.
Repeated preparation rebuilds and charges that work even when content history
already recognizes the image. Local quotas renew independently of cumulative work.
Ordinary `Atlas` elapsed time includes the producer's indivisible glyph, ramp and
packed-stream resolution; it does not isolate image packing from that shared call.

Lifecycle/transfer units cannot be configured as work limits. Fixed Renderer
options accept the union of actual frame and [readback](#readback-accounting)
producers; each operation receives a fresh allowance. Standalone fronts reject
limits belonging to other operations. Mandatory releases and
rollback cannot be vetoed by an exhausted forward-work allowance. Reused native
resources make no construction request. Native requests are charged before the
constructor; generation custody observes returned provisional handles even when
later native error-scope validation rejects them.

On first poll, a render reserves its attempt/report and CPU/generation interval
storage before frame effects. An unpolled future starts nothing. Setup failure
preserves the previous report. `Renderer::last_attempt` borrows the latest started
`RenderFrameReport`; `take_last_attempt` moves it to caller custody. Terminal
status is `Succeeded`, `Refused`, `Failed(ErrorCode)` or `Canceled`, after inner
cleanup. `attempt`, `renderer_cpu` and `generation` expose distinct typed closing
intervals. A domain whose extraction failed returns its typed fault;
already closed intervals keep their real boundary, including when later rollback
occurs. A cleanup fault remains separate from the causal diagnostic.
Accounting errors retain their typed cause without allocating during construction.
`Error::message` formats and caches human-readable text on first observation;
typed diagnostic access and Display formatting leave that cache uninitialized.

### Resolved accounting reports

Accounting reports resolve Ledger scope handles through their owning typed table
when they are constructed. The constructor checks the registered root, unique
rows, parent relationships and every referenced scope, including both scopes of
a refusal. Invalid attribution returns `RenderAccountingError::Invariant` before
publication. Display labels do not determine scope meaning. Scope creation follows
the declared parent relationships; report summaries use semantic scope order.

Published reports retain resolved numeric summaries, typed events and observation
metadata. Their readers perform no scope search or validation and cannot silently
drop an unresolved event. Forward-report equality retains the validated operation
identity; CPU and generation reports retain their owner identities. Equality uses
those identities and resolved content, rather than display labels or redundant
raw rows.

Resolved summary and event backing is reserved fallibly before attempt or
checkpoint work, within the fixed scope count and configured event bound.
Snapshots reserve at observation setup. Raw Ledger buffers and reserved adapter
buffers coexist during conversion; raw buffers are discarded before publication.
This backing belongs to the existing bounded observer/control-plane exclusion,
so observing an operation does not consume its algorithm byte caps or work units.
Off intervals still retain all fixed-scope numeric summaries. Off snapshots retain
exact root usage and identity without role or event buffers.

Owned reports have no allocating `Clone` implementation. Borrow the latest record
or take it into caller custody to retain it. This applies to forward, CPU and
generation reports and their frame/readback composition records; nonallocating
identity and report-slot handle clones retain their existing behavior.

`Renderer::cpu_accounting_snapshot` observes current Renderer CPU lifetime usage.
`device_accounting_snapshot(None)` observes the existing default generation;
passing `Some(&surface)` selects that Renderer's owned surface generation. These
read-only operations neither select a new adapter nor retire a device, replace
the latest attempt or require a renderable generation. They expose initialization,
current custody and retained terminal state without an archive of old generations.
Snapshots provide exact root usage with profiling Off; scoped lifetime usage
requires an enabled profile. Frame intervals retain exact scoped usage in all
profile modes.

Native shader setup records binding visits and emitted source bytes under
`DeviceBacking`. `ShaderSourceScanBytes` counts the input bytes supplied to
each complete replacement-pattern scan. Preparation checks match cardinality
and disjoint byte ranges before acquiring output scratch, then emits from those
ranges in source order. A frame-owned observer retains its binding work in
the frame attempt.

The device pass cache admits usable sampler, layout, shader-module and
pipeline index capacity under `DeviceBacking`. Its byte unit is the
container's guaranteed record capacity times the key/value record size.
Bucket storage beyond that capacity and allocator overhead are outside the unit. Both
provisional and committed capacity count while their backings coexist.
Growth admits the replacement before relocating records. Publication uses
the capacity reserved by the checked cache update.

`Summary` and `Detailed` optionally collect host elapsed time using Ledger's
platform clock or `RenderClock::Host`. Timed phases surround actual work and
awaits, including pending preparation, submission resolution and surface
acquisition/presentation. Off and untimed modes perform no clock reads. Detailed
event limits apply independently to each owner and interval; phase/domain filters
select detail only. Lost events do not alter authoritative usage, peaks or
outcomes. Owner lifetimes include idle gaps; phase durations are inclusive and
can overlap. Do not add parent/child durations, peaks or the three domains, or
interpret these durations as GPU hardware execution timing. Persistent cleanup
and timed snapshots can observe the clock after a frame publishes. A host clock
must remain monotonic, nonpanicking and free of observable side effects.

Successful `Stats` projects normalized emissions and actual encoded native work
from the attempt, plus cache allocation/reuse work and idle classification from
the generation interval. Counts use `u64`. `frame_time`, `preparation_time`,
inclusive `execution_time`, `presentation_time` and `surface_acquisition_time`
are `Option<Duration>`, absent when untimed. `texture_copy_operations` counts
native texture copies rather than graph nodes. `image_passes` includes actual
image-processing shader draws and attachment-clear passes. `queue_upload_bytes`
counts actual queue-upload input bytes. Exact-content `image_content_hits`,
`image_content_misses` and `first_observed_image_bytes` describe this frame's
observations against Renderer history, independently of debug diagnostics.

CPU custody measures actual Vec/String capacity bytes, explicit separately
heap-owned state and known producer payloads, including image-expansion and
font-backing index records. Known heap-owned font and image adapter payloads are
counted separately from opaque Arc/Blob headers. Each image Blob adapter owns a
known `Arc<[u8]>` payload: Renderer imports of clones share its admission, while
distinct adapters over one pixel allocation have separate admissions, including
zero-byte images.
History-only pixel identities retain no adapter. Pixel backing deduplicates by
Arc allocation identity, fonts by producer `ResourceId`, and prepared-run allocations
by their canonical Arc identity. Run-owned bytes are separate from shared
font/image payloads. Inline structs, allocator headers/buckets, Ledger/control
storage, dependency-private scratch and process/RSS usage are excluded. Native
device-state, manager and handle-registry infrastructure is also excluded; its
nested source/key/index Vecs and retained payloads remain measured.
Temporary text values and resource iterators retain their backing admissions
through actual destruction inside the owning CPU disposal phase, including
early failure and cancellation.
`StorageRelocations` counts existing elements moved to a replacement owned-vector
backing. It excludes new element writes, insertion suffix shifts and in-place
rotations. Renderer storage charges this checked-unlimited work to its CPU
lifetime owner; it does not spend a fresh frame or readback work allowance.
Standalone preparation charges its own operation and permits a work limit where
that entrypoint produces relocations. Existing image-identity and raster-resource
copy units retain their new writes and suffix shifts, without counting the same
replacement move again. The eleven owned encoding-stream replacement backings
use this relocation unit too; `EncodingStreamCopies` counts their independent
stream copies instead.
The CPU import registry also charges its actual existing entry count before a
replacement move, at `ImagePayloads`.
The bounded raster adapter admits replacement capacity for its eleven public
Encoding streams before producer calls. The dependency-owned mask lookup
constructors offer no fill API: their known 1024/8192-byte payloads are previewed,
then actual returned capacity is admitted before retention or upload. Their
infallible allocation and private scratch remain an explicit dependency boundary.

Exact-content history uses measured sorted records and preserves canonical
baseline identities. A draft accounts for baseline/new records and simultaneous
backing. Finalization can dispose old storage before report closure; a later
failure removes new records in place without allocating, preserving previous
logical history while charging any larger surviving capacity. All fallible
finalization and statistics projection precede successful metadata/headless
publication. The interval may still include old/new headless textures; dropping
the old publication is later generation-lifetime release. Host presentation may
already have occurred before a later fault resolves, so this is not physical
display rollback. Explicit readback has its own report and final caller-transfer
boundary, described below.

## Readback accounting

Awaiting `Renderer::read_headless` starts a separate readback attempt on first
poll. An unpolled future starts nothing. Report and CPU/generation checkpoint
storage is reserved before native effects; setup failure preserves the previous
readback report. `last_readback_attempt` borrows its latest started
`RenderReadbackReport`, and `take_last_readback_attempt` moves it to caller custody.
These operations preserve the latest frame report, last-successful `Stats`,
canonical image observations and complete headless publication.

Available zero-area surfaces return validated empty output without native work.
Nonzero surfaces without a publication return the existing typed uninitialized
error; rejected requests produce started-attempt outcomes without fabricated
copy, submission or mapping work. A generation interval is present only after a
generation is associated with the nonempty readback.

The attempt admits real native constructor requests through the existing
generation producers, followed by actual copy, submission, mapping and row work:

| Unit | Meaning |
| --- | --- |
| `ResourceAllocationRequests` | Staging-buffer, command-encoder and command-buffer constructors actually invoked. |
| `ReadbackCopies` | Encoded texture-to-buffer copies; texture-to-texture `CopyOperations` remains distinct. |
| `Submissions` | Actual queue submissions. |
| `ReadbackMapRequests` | Actual native mapping requests registered. |
| `ReadbackRows` | Source rows inspected, including an incomplete row that fails validation. |
| `ReadbackBytes` | Logical unpadded RGBA8 bytes actually copied into output. |

Each row is admitted before inspection, and its logical payload before the copy.
Refusal or malformed rows discard the complete private output, retaining only
actual accepted-prefix work. There is no CPU alpha-conversion loop: readback
copies the rendered straight-alpha pixels. Native map internals and driver
storage are outside these observable units.

Output uses fallible Vec construction with actual returned capacity under
`RenderCpuScope::ReadbackOutput`. Its admission remains paired with the data
through ImageBuffer validation, final device validation and report extraction.
Renderer CPU caps cover this allocation together with existing lifetime backing;
they do not reset for each readback. Failure and cancellation destroy owned data
before mandatory release. Callback/helper owners carry only generation custody,
never a readback attempt. A callback can still retain staging after the report
closes; late delivery, destruction and retirement cannot change that report.

Successful report intervals close **before final caller transfer**, with complete
output capacity still live. The final CPU-lifetime atomic batch records
`ReadbackTransferredBytes` for that admitted capacity and releases its custody
without destroying the data. Rejection commits neither change, disposes the
guarded output and records the precise failed outcome. Already-closed intervals
retain their earlier numbers; later disposal appears in lifetime observations.
After a successful batch, only infallible report storage and the plain
ImageBuffer move remain. Caller values and clones carry no hidden participant.

For 3×2 RGBA8, two row visits copy 24 logical bytes from a 512-byte padded staging
descriptor. If the Vec's actual capacity is `C ≥ 24`, the successful CPU interval
closes with `C` output bytes still live and zero transfer work. Subsequent CPU
lifetime usage records `C` transferred bytes and releases that output custody.
Transferred capacity is distinct from logical payload and is not deallocation.

Ledger's `Readback` phase covers the body through guarded pretransfer completion.
`ReadbackPrepare` includes native construction/copy encoding and its
`ReadbackSubmit` child; `ReadbackResolve` measures actual awaited error scopes.
`ReadbackMap` includes registration, Pending waits and completion, including its
`ReadbackDecode` child. `ReadbackCleanup` times staging-owner disposal; the CPU
owner times output allocation destruction. These are
inclusive host observations, not GPU hardware timings or additive durations.
Intervals and timing end before final transfer. Off/untimed modes read no clock;
bounded detail loss or filtering does not alter usage, limits or outcomes.

## Standalone accounting

`Scene::inspect` requires a mutable `RenderAccountingContext` and returns a
`SceneSummary` of nested authored commands, fills, strokes, shadows, images,
glyphs and layers. Its `Authored*` units differ from normalized `*Emitted` work;
actual repeated traversals charge visit units separately. Image-content
observations are local to that inspection, not Renderer history.
Inspection stores borrowed image observations: its scratch backing is measured,
while `ImageIdentityDrops` remains zero because no owned retained-identity record
is created or destroyed. That unit covers owned history/identity records rather
than borrowed slots or ordinary Image clones.

`ImageRepeatPlan::resolve`, `BackgroundNormalizationInput::normalize`,
`ClipInput::normalize` and `BoxDecorationInput::normalize` likewise require the
context. Nested work uses the same participant and local quotas. Actual
scratch/output backing is admitted; only complete successful output transfers to
caller custody. Pure construction, validation and constant-size
`ImagePlacementInput::resolve` remain context-free. Returned values and later
caller clones do not retain the context's participant.

`RenderAccountingContext::last_attempt` borrows a `RenderStandaloneReport` for
the latest started operation. Its `kind` and terminal `status` identify that
operation even when cleanup or report extraction fails. `attempt()` returns
the validated forward report or its typed extraction error; `cleanup_fault()`
exposes the first mandatory cleanup failure separately. Cancellation remains
`Canceled` with any cleanup fault, while a cleanup fault prevents a successful
status. A fault belongs to that operation and does not fail the next operation.
An existing semantic error or forward refusal remains the returned cause;
a cleanup fault instead prevents a successful body from returning output.
Setup failure before an operation starts preserves the preceding record.

Inspect the record after completion or guard drop, including after unwinding,
before destroying the context. `take_last_attempt` moves it to caller custody;
the owned record remains observable after the context is destroyed, without
starting another operation. Discarding both the context and its untaken record
ends that observation lifetime. The record does not allocate or clone its
report. Named Ledger invariant failures retain their exact
`RenderAccountingError::LedgerDefect` payload; Render's own `Invariant` remains
a distinct error. Usage and release failures found during destruction retain
their typed cause while remaining mandatory releases continue.

Derived image tiles and outline targets preserve `Rect`'s finite maximum
coordinates as well as its component constraints. Intrinsic overflow returns
the owning numeric input diagnostic before any result is published. Accounted
placement proves two maximum coordinates; each repeated tile proves two more
before its quota and `RepeatTiles` emission admission. Outline targets admit
four component derivations and then two maximum-coordinate proofs. Numeric
failure retains completed work, releases operation backing, and transfers no
output. Standalone image placement performs the same proof without a context.

Both standalone image repetition and background normalization admit repeat-axis
work in the `Repeat` phase. `CoordinateVisits` covers tile and clip checks,
intersection tests, first repeated positions, axis and grid counts, and
position generation. An empty axis retains its admitted work, and the
`RepeatTiles` count advances for admitted tile emissions. Ledger refusal
stops the planner at the corresponding check or calculation.

## Preparation accounting

`FontData::prepare_glyph_run` and `prepare_text_layout` take a mutable
`RenderAccountingContext` after their existing limits. Reuse the context to
change `RenderAccountingOptions` for the next call, inspect `last_attempt`, or
move its terminal record to caller custody with `take_last_attempt`, as described
under [standalone accounting](#standalone-accounting). Every preparation path
requires the context.

The measured boundary is standalone glyph and placed-Text preparation, including
outline/variation qualification, provider callbacks, intrinsic color/bitmap
processing, PNG decoding and coordinate conversion. Scene/GPU work dimensions
remain zero in preparation reports, and configuring them for a preparation
operation returns `InvalidLimit`. Text-only dimensions also
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
The [resolved report buffers](#resolved-accounting-reports) use the same exclusion.
Failure reporting therefore does not recursively consume an exhausted output cap.

Capacity growth previews the requested replacement without committing usage,
reserves fallibly, admits its observed actual capacity while the old backing is
still live, then drops and releases the old backing. Reservation failure creates
no acquired peak. All temporary holds release at drop. Failed results discard
their retained output before closing custody; successful complete outputs transfer
their admitted backing to the caller. Transfer is not deallocation. Subsequent
caller clones and output lifetimes have no hidden accounting callbacks.
A preview refusal is retained in the attempt status and typed refusal; it does
not create a Ledger admission event or update admission observations.

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

## Device-generation custody

Every native device generation creates one Ledger before engine and shader
initialization. It remains available after Ready state is dropped, and terminal
transition or device-state drop retires it once. Work is checked-unlimited;
Renderer's selected profile and clock apply from initialization onward. Typed
generation intervals and `RenderDeviceSnapshot` expose its usage separately from
fresh frame work and Renderer CPU lifetime custody.

`GpuBytes` measures descriptor-defined buffer lengths and texture payload sizes
for observed handles returned to Render, including provisional handles awaiting
checked error-scope validation. It does not prove successful driver allocation,
resident memory or allocator overhead. Native errors can arrive after a handle
has been returned; rejected provisional owners still release custody exactly
once. `ResourceAllocationRequests` counts actual constructor invocations,
`ResourceAllocations` counts returned descriptor-bearing handles, and
`GpuObjectsCreated` counts returned opaque handles/explicit utility aggregates.
A refused noncommitting preview invokes no native constructor and changes no
usage or peak. Setup failure also leaves a healthy generation retryable.

| Fixed generation scope | Observed ownership |
| --- | --- |
| `render.device -> cache -> vello / effects / masks / kernels` | Managed buffer/texture descriptor payload, cache registration, idle classification, allocation/reuse/probe/return/eviction work. |
| `render.device -> external -> surface_targets / pass_buffers / readback_staging` | Headless/presented target textures, pass operation/parameter buffers, and padded staging-buffer descriptors. |
| `render.device -> objects -> shader_module / bind_group_layout / pipeline_layout / compute_pipeline / render_pipeline / sampler / bind_group / texture_view / command_encoder / command_buffer / dependency_utility` | One observed opaque handle per owner; short-lived shader modules/layouts/encoders release at their actual consumption/drop. |
| `render.device -> retirement` | Once-only terminal retirement work, after custody cleanup. |
| `render.device -> initialization` | Actual awaited native engine initialization. |

These are fixed named scopes, including the root and grouping scopes. Attribution is
inclusive: ancestor and child quantities must never be added. `OpaqueGpuObjects`
uses one unit for each explicitly owned handle, and one aggregate unit for the
opaque `wgpu::util::TextureBlitter` dependency utility. Its internal objects and
per-copy creations are outside that aggregate observation; no primitive count is
inferred from dependency implementation details. Imported swapchain textures,
device/queue anchors, dependency/driver internal allocations and internal handle
retention are outside these descriptor units. Render-created views of imported
swapchain textures are observed opaque handles. Borrow-scoped render/compute pass
recording guards remain within command-encoder custody; they are recording phases,
not additional standalone native allocation units. Actual pass/dispatch work is
charged to the surrounding frame attempt at native encoding.

Native clones share a single physical hold. Explicitly created texture views also
share their known parent texture hold, so dropping the texture wrapper while a
view survives does not prematurely release its descriptor payload. This tracks a
direct relationship established by Render's factory; it does not infer a general
WGPU dependency graph. Existing resource leases and pass/submission owners retain
their source buffers/views through their observed use. Native values drop before
their shared custody token, and native constructors, Drop callbacks, submission,
and awaits run without a Ledger/lifecycle lock. Driver retention after the last
explicit Render owner ends is outside the measured domain.

`RegisteredCacheGpuBytes` classifies payload represented by current managed
entries; `IdleCacheGpuBytes` classifies registered entries eligible for idle
retention. Neither is additional physical storage. Removing registration clears
those classifications while a surviving native clone/view can keep physical
`GpuBytes` live. Cache retention compares the target with all physical cache bytes,
including active and idle resources, and evicts only idle entries. The target is
an idle-retention policy, never an active-GPU allocation cap.

Exact entry-set/identity checks independently validate registration against the
Ledger. Committed transition/invariant failures preserve the first health fault
and block new work. Mandatory release remains nonpanicking and cannot be vetoed
by cleanup work-counter overflow. Retirement releases each authoritative leaf
and its projections once; surviving owners and late readback callbacks then
release harmlessly and cannot acquire into the retired generation. Readback
cancellation retains staging custody until its native map callback is delivered
or destroyed, rather than releasing while a callback-owned buffer clone survives.

One outer public-render checkpoint spans direct or graph execution, nested cache
lease scopes and cache cleanup. Successful publication projects `cache_allocations`
and `cache_reuses` from the cache subtree's interval work, and `idle_cache_bytes`
from its closing classification. These `u64` fields replace
`effect_texture_allocations`, `effect_texture_reuses` and `retained_effect_bytes`;
this is an intentional source API change.
A cold direct Vello frame now reports its real managed allocations. Failed or
canceled attempts preserve prior published headless pixels and successful
metadata/`Stats`. Host presentation may already have occurred before a later
backend error resolves; this does not promise rollback of the physical display.

If a poisoned registration balance underflows during mandatory cleanup, a later acquisition
can now return `ErrorCode::Accounting` with the exact first
`GenerationRefused { quantity: RegisteredCacheGpuBytes, current, amount, reason:
Underflow }` diagnostic before invoking another native constructor. The original
manager identity/fault diagnostic remains available and unchanged.

The interval closes before the nonfallible surface/publication commit. Its
closing physical custody may therefore include both the previous published
headless texture and the new draft; replacing/dropping the old publication is a
later generation-lifetime release outside that interval. The three cache fields
already follow cache cleanup and do not claim a complete postpublication frame
resource report. Generation timing is independent of frame elapsed time; the
attempt supplies successful-frame duration and semantic work projections.

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
