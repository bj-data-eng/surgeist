# Native rectangular morphology

Prepare resolved radii and a component operator, declare the actual source, and
append it with `Scene::morphology_layer`. Render executes rectangular erosion or
dilation through the ordinary Layer/Scene GPU transaction. Sources can contain
Images, vector draws and nested layers. Callers retain SVG syntax, unit
resolution, named filter graphs and document adapters.

## Build a finite source

```rust
use surgeist_render::{
    Color, FilterColorSpace, FilterDomains, Layer, MorphologyInput,
    MorphologyOperator, MorphologySource, Rect, RenderAccountingContext, Result,
    Scene, prepare_morphology,
};

fn dilated_scene(accounting: &mut RenderAccountingContext) -> Result<Scene> {
    let domain = Rect::try_new(0.0, 0.0, 32.0, 24.0)?;
    let operation = prepare_morphology(
        MorphologyInput::new()
            .with_radii(2.0, 1.0)
            .with_operator(MorphologyOperator::Dilate),
        FilterColorSpace::LinearSrgb,
        accounting,
    )?;
    let mut input = Scene::new();
    input.fill(Rect::try_new(8.0, 6.0, 16.0, 12.0)?, Color::BLACK);
    let source = MorphologySource::finite(
        input, FilterDomains::try_new(domain, domain, domain)?,
    );
    let mut output = Scene::new();
    output.morphology_layer(Layer::new(), operation, source)?;
    Ok(output)
}
```

This scene dilates each premultiplied RGBA component of the captured rectangle
independently. Select `MorphologyOperator::Erode` for component minima.
`MorphologyInput::new()` and `Default` select Erode with radii `[0,0]`.
`with_radius(r)` sets both axes; `with_radii(x,y)` sets them independently.
`PreparedMorphology` is immutable inline data with no coefficient backing or
native handle. Its grid-dependent interpretation belongs to frame planning.

Preparation checks X then Y for finiteness. Both checks precede the rule that
**any supplied radius <=0 disables the whole primitive**, including signed zero.
Thus `[0,2]` is a clipped identity request. Strictly positive `[0.25,2]` remains
an active request: at density one it has a point X axis and a radius-two Y axis.
Positive `[0.25,0.25]` quantizes to a point kernel on both axes. Disabled and
positive point-kernel requests preserve their distinct admitted meanings even
when their pixels coincide.

For positive radii, the selected native grid policy is
`Rx=floor(rx*density_x)`, `Ry=floor(ry*density_y)`. A grid-center neighborhood is
the inclusive rectangle `[-Rx,+Rx] × [-Ry,+Ry]`, with `(2Rx+1)(2Ry+1)` centers.
For example, radius 0.999 selects R0 and radius 1 selects R1 at density one;
radius 0.5 selects R1 at density two. This is a native floor/center-sampling
policy supported by pinned implementation precedent. The morphology clause
does not prescribe fractional-pixel discretization. Radii are never capped to
the output drawing rectangle.

The receiver replaces the Layer's exclusive foreground effect. Complete outer
transform, isolation, clip, mask, opacity and composition remain outside
morphology evaluation. Nest layers for consecutive effects. Source commands
move into the existing iterative child owner; metadata retains no second Scene.
Cloning, inspection, formatting and disposal use that owner's bounded call stack.

## Domains, coverage and authentic sources

`MorphologySource::finite(scene, domains)` always has transparent exterior and
offers no edge-mode selector. `FilterDomains` keeps source S, filter F and result
P independent. Input D=S intersect F receives raster-cell area coverage once
before extrema; output O=P intersect F receives its independent coverage once
after final color resolve. A smaller P cannot remove contributing input. The
horizontal stage evaluates output-X columns for every D row so the vertical
stage can consume rows outside P. No coverage is repeated between table levels
or axes.

Finite erosion includes exterior transparent black. For the scalar row
`[1/4,3/4,1/2]`, an R1 neighborhood gives dilation `[3/4,3/4,3/4]` and erosion
`[0,1/4,0]`, when the other axis stays inside sufficiently repeated rows. At the
ends, erosion sees zero outside D. Clipping the neighborhood to available pixels
would change that result. A half-covered opaque D cell dilated into an output
cell with half O coverage yields alpha 1/4: input and output coverage each enter
once.

Finite input includes supported completed same-layer backdrop before filtering.
If capture expands the group, a checked raw rebase restores D's lattice before
coverage or color work. All maps retain the complete outer affine, including
reflection, rotation, skew and unequal scale. Shared relative-grid maps avoid
uploading large absolute origins followed by lossy GPU subtraction.

`MorphologySource::constant(color,F,P)` declares authentic constant unbounded
input. Both operators return that same color with final O coverage and outer
semantics. This analytic route needs no radius quantization, extrema texture or
color-transfer round trip, even for huge finite positive radii. A large finite
fill remains finite and retains transparent exterior.

`MorphologySource::periodic(tile_scene,T,F,P)` owns an actual positive fundamental
period. At selected density rho it captures an integer canvas
`Nx=max(1,ceil(T.width*rho))`, `Ny=max(1,ceil(T.height*rho))`, with separate axis
densities Nx/T.width and Ny/T.height. Nested planning uses that scoped texel
canvas; the enclosing context is restored for following content. Seam cells do
not receive unrelated global-grid partial coverage. Extrema wrap the complete
period, and final output uses qualified four-neighbor wrapped bilinear
reconstruction of the filtered canvas. Disabled and point-kernel periodic
requests reconstruct that actual canvas without extrema tables or RGB transfer.

Declared constants and periods use their specific infinite-source behavior
during evaluation; F and P clip output. This precedence is an explicit native
resolution of the overlap with the common input hard-clip rule, matching the
[native Gaussian source contract](gaussian.md#finite-constant-and-periodic-inputs).
It is not a claim of normative consensus or universal browser pixels. Infinite
sources are never inferred from finite geometry. The receiver rejects same-layer
backdrop for either infinite declaration before append, leaving the destination
Scene unchanged. Nested finite tile content retains its supported backdrop rules.

Empty O schedules no operation. Empty D or ink gives transparent output while
retaining a nonempty bounded O for destructive composition. A final output
footprint follows the actual completed producer through the full affine.
Clear, Copy, SourceIn, SourceOut, DestinationIn and DestinationAtop preserve the
parent outside reconstructed O; source alpha, opacity and masks do not redefine
that footprint. Author clips still cover the whole completed operation. Singular
outer affines have an empty bounded footprint. Ordinary format quantization and
hardware sampling error remain permitted.

## Exact extrema and color phases

Erode computes the minimum of each premultiplied component; Dilate computes the
maximum. Components may come from different pixels. For example,
`A=(.25,0,0,.5)` and `B=(0,.5,0,.75)` dilate to `(.25,.5,0,.75)`, rather than
selecting either entire pixel. Component extrema preserve RGB<=alpha.

`FilterColorSpace::Srgb` uses numeric premultiplied RGB and alpha directly.
`LinearSrgb` guardedly unpremultiplies, decodes RGB and repremultiplies once
before active extrema. Tables and both axes retain that representation; final
spatial reconstruction precedes one encode/repremultiply at resolve. Alpha has
no transfer conversion. The shared effective-subnormal-alpha policy treats alpha
below `f32::MIN_POSITIVE` as zero before division. Ordinary f32 rounding and
flush-to-zero remain permitted.

Active converted and extrema intermediates use shared `Rgba32Float` resources,
without half-float or eight-bit rounding between nonlinear conversion and
extrema. Actual output/capture storage still follows the selected working
format. Active routes require `supports_filter_float32`; a refusal identifies
`RuntimeOperation::MorphologyRendering`. Constant, disabled and whole point-copy
routes do not require this capability. There is no CPU production renderer or
universal bitwise color-round-trip guarantee.

The separable GPU algorithm builds power-of-two range-extrema tables. For an
axis of length N and component operator E:

```text
T0(i) = input(i)
Tk(i) = E(T(k-1)(i), T(k-1)(i + 2^(k-1)))
```

Finite Tk contains only valid starts, so its axis length is N-2^k+1. Periodic
tables retain N cells and wrap their circular spans. An inclusive interval
`[lo,hi]` of length L>0 selects `k=floor(log2 L)` and returns
`E(Tk(lo),Tk(hi-2^k+1))`. These overlapping intervals cover the query exactly
because min/max is idempotent. Integer texture loads perform no interpolation
inside extrema.

Finite dilation clips queries to D; empty intersections return zero. Omitting
exterior zero is exact for maxima of nonnegative components. Erosion separately
requires the whole nominal neighborhood inside D, returning zero otherwise.
Periodic queries reduce their window to at most a full period: any consecutive
N cells contain the entire axis period.

Variable finite dilation lengths can select different table levels across one
output plane. A FirstQuery fills length-one queries and seeds remaining pixels;
each reached level then builds its table and runs a distinct RefineQuery. The
refinement reads that table and the previous query output, fills its matching
length class, and copies completed results elsewhere. The seed is not a final
transparent fallback. Only the completed level sequence may feed the next axis
or final resolve. Constant-length queries instead consume their selected final
table directly.

A positive radius that quantizes to zero on one axis performs point selection,
with no extrema table for that axis. The explicit one-cell route is a FirstQuery
whose proved constant length is one and maximum level is zero. It is immediately
complete with one scheduled fetch per cell, so no refinement follows it. Point
selection may alternatively be fused with preparation/resolve or alias an
identical grid when proved. When both axes are points, or the request is disabled,
the copy route preserves the input/output coverage contract without RGB transfer.

Closed graph facts authenticate source kind, operator, color context, table
levels and spans, query completion, formats, generations, actual reads and final
footprint provenance. Each output has one producer and distinct backing from its
reads. A table survives both its query and the next table build; a previous query
output survives its refinement. Old tables and query outputs release after their
exact last use. All table levels need not remain live simultaneously.

Dimensions depend on source/output extents, rather than radius-sized padding.
Huge positive scaling is consumed only by proved semantic bounds: full-period
queries, full-input dilation queries or always-exterior finite erosion. These
routes classify before integer narrowing; other unrepresentable reached
arithmetic refuses. Endpoint maps, shifts, query lengths, sample indices,
uploaded arithmetic, actual texture sizes and storage requests must qualify.
Actual dimensions remain subject to device and accounting limits without an
arbitrary public radius cap.

## Accounting, diagnostics and API impact

`prepare_morphology` records one central `RenderAttemptKind::Morphology` attempt.
It spends one `FilterOperationVisits` and each reached scalar inspection, reaching
two `CoordinateVisits` on success. A nonfinite value retains its inspected
prefix. Inline admission has no fictitious backing lease or output-transfer charge.
Reuse the [standalone context](reference.md#standalone-accounting) for subsequent
preparations and terminal reports.

Frame work uses the existing Ledger owners for geometry and repeated proofs,
graph nodes/edges, resource probes, retained facts, shader parameters, serialized
bytes, uploads, actual backing and native handles. Morphology adds no Gaussian coefficient allocation, coefficient
`KernelSampleVisits` or fabricated Kernel timing interval. Captured source
rasterization retains its own actual Kernel phases and any raster sample work.
`ScheduledFilterTexels` counts actual scheduled rebase, preparation, table, query,
copy and resolve target cells. `ScheduledMorphologySamples` counts scheduled
opportunities, independently of hardware fetch traffic:

| Stage | Opportunities per scheduled cell | Read resources |
| --- | ---: | ---: |
| Table build | 2 | 1 |
| Constant-length interval query | 2 | 1 |
| FirstQuery, including complete one-cell selection | 1 | 1 |
| Variable RefineQuery | 3 | 2 |
| Raw rebase, point copy, preparation or direct resolve | 1 | 1 |
| Periodic resolve | 4 | 1 |
| Analytic constant | 0 | 0 |

Refinement counts the previous output and two candidate table loads even for
shader-guarded nonmatching cells. Two table texel loads still read one graph
resource. `ImagePasses` counts real draws; omitted work incurs no fictional pass
or sample charge. Timings use existing host/backend phase intervals and make no
GPU timestamp claim.

`Error::morphology_diagnostic()` exposes `MorphologyPreparationError` for invalid
RadiusX/RadiusY, EmptyPeriod, InfiniteSourceBackdrop and reached numeric refusal.
Numeric phases distinguish RadiusGrid, EndpointMapping, TableLayout, GridIndex
and Sampling. Accounting, cancellation and device failures retain their existing
typed channels. Normalization checks both LayerFilter semantic support and
`PrimitiveOperation::GpuMorphologyFilterExecution`; the corresponding capability
reader is `FilterCapabilities::supports_gpu_morphology_filter_execution()`.

All intermediates remain provisional in the ordinary atomic frame transaction.
Refusal, cancellation or backend failure preserves prior published pixels and
successful statistics, releases provisional custody and permits healthy retry
with the original Scene and shared immutable Images. See
[frame accounting](reference.md#frame-accounting) and
[getting started](getting-started.md) for rendering and explicit readback.

The new request, operator, preparation, source, receiver and diagnostic APIs are
additions. Adding `GpuMorphologyFilterExecution` to the existing exhaustive public
`PrimitiveOperation` enum is source-breaking for downstream exhaustive matches,
including facade consumers. There are no compatibility aliases or changes to
that enum's exhaustiveness. Existing [Gaussian](gaussian.md), [offset](offset.md)
and [native shadow](native-shadow.md) retain their operation-specific semantics.

The semantic sources are the [morphology clause](https://drafts.csswg.org/filter-effects-1/#feMorphologyElement),
[primitive subregions](https://drafts.csswg.org/filter-effects-1/#FilterPrimitiveSubRegion)
and [filter color interpolation](https://drafts.csswg.org/filter-effects-1/#color-interpolation-filters-property).
The retained [Filter Effects edition](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md)
contains the matching clauses. Pinned WebKit
[`FEMorphologySoftwareApplier.cpp`](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb/Source/WebCore/platform/graphics/filters/software/FEMorphologySoftwareApplier.cpp)
supplies floor-discretization precedent; its destination caps and clipped erosion
neighborhood are not this native independent-domain contract.
