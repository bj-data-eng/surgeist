# Native resolved drop shadow

Prepare a resolved shadow, declare its actual source and append it with
`Scene::drop_shadow_layer`. The ordinary Renderer/Surface GPU transaction captures
the source once, filters its alpha, displaces that finite image and merges the
unchanged original above the colored shadow. This receiver is separate from
CSS `FilterDropShadow` and shape/box `Shadow`.

```rust
use surgeist_render::{
    Color, DropShadowInput, DropShadowSource, FilterColorSpace, FilterDomains,
    Layer, Rect, RenderAccountingContext, Result, Scene, prepare_drop_shadow,
};

fn shadowed_scene(accounting: &mut RenderAccountingContext) -> Result<Scene> {
    let domain = Rect::try_new(0.0, 0.0, 48.0, 32.0)?;
    let shadow = prepare_drop_shadow(
        DropShadowInput::new()
            .with_dx(3.0).with_dy(2.0)
            .with_standard_deviations(2.0, 1.0)
            .with_flood_color(Color::try_rgba(0.0, 0.0, 0.0, 0.5)?)
            .with_flood_opacity(0.75),
        FilterColorSpace::LinearSrgb,
        accounting,
    )?;
    let mut input = Scene::new();
    input.fill(Rect::try_new(8.0, 6.0, 20.0, 12.0)?, Color::BLACK);
    let mut output = Scene::new();
    output.drop_shadow_layer(Layer::new(), shadow,
        DropShadowSource::finite(input,
            FilterDomains::try_new(domain, domain, domain)?))?;
    Ok(output)
}
```

## Admission and source ownership

`DropShadowInput::new()` and `Default` request dx=dy=2, sigma=[2,2], opaque
black flood, opacity1 and `ImageRendering::Auto`. Set both deviations together
or independently. Preparation validates finite dx/dy, finite X/Y deviation, then
finite resolved flood opacity in [0,1]. Either negative deviation disables only
blur; zero axes omit their kernels. Displacement, flood and original merge remain
active. Positive finite sigma has no CSS cap; actual grids, coefficients and
device limits must qualify during rendering. `PreparedDropShadow` is immutable
inline data with no coefficient allocation or native handle.

`DropShadowSource::finite(scene, domains)` uses transparent extension of its
finite source. It exposes no finite edge selector. `constant(color,F,P)` declares
an authentic unbounded color; `periodic(scene,T,F,P)` declares an actual positive
fundamental period. These declarations delegate the existing
[Gaussian source owner](gaussian.md#finite-constant-and-periodic-inputs), including
its integer periodic canvas, separate axis densities and scoped frame context.
Infinite sources are never inferred from large finite shapes. The fallible
receiver rejects same-layer backdrop for infinite declarations before append.
Finite sources include supported backdrop before capture; a larger completed
backdrop is rebased to the exact raw finite source lattice.

The receiver replaces the Layer's exclusive foreground effect. Outer transform,
clip, mask, opacity and composition remain after the native effect. Source commands
move into the existing iterative child owner; there is no second Scene or cloned
source tree. When the alpha branch executes, the retained original texture has two actual
consumers: alpha preparation and final merge. Original-only routes read it only
at merge. Its registered last use is the merge. Constant
originals are descriptors and require no original texture.

## Stage domains and coverage

For finite input, D=S intersect F and final O=P intersect F. Each reached stage
uses its own fractional raster-cell coverage:

1. Capture the raw outward D lattice once. Alpha preparation applies D cell
   coverage once and projects black RGB with unchanged alpha.
2. Evaluate [native Gaussian](gaussian.md) into **F**, including vertical halo
   support, and resolve a finite F-covered alpha image. A smaller outer P cannot
   crop contributing alpha before displacement.
3. Displace that already covered image with the existing
   [offset reconstruction](offset.md). Its exterior is transparent, including
   when the original was constant or periodic. The offset does not multiply
   source F coverage again; destination coverage is deferred.
4. Fuse flood-in, unchanged-original-over-shadow and final O coverage in one GPU
   merge. Apply O after the merge, once.

Original finite input receives D coverage at its merge read. Constant/periodic
originals instead receive the common finite F input coverage. Only Gaussian
uses the specific infinite-source extension exception: its resolved AlphaF is
still finite before offset. This phase-specific precedence and the fused
internal coverage are selected native raster policies, not claims of exact
continuous geometry or cross-browser pixel consensus.

For example, constant blue alpha.5, F=O=[0,.5] on a unit grid, zero displacement
and blur, and red flood alpha.5 produce AlphaF=.25 and original alpha.25. Shadow
alpha is.125, merged alpha is.34375, then O gives premultiplied
RGB(.046875,0,.125), alpha.171875. Repeating F coverage at offset or clipping
both branches by O before merge gives a different result.

Empty O has no operation. Empty D/source remains transparent in a nonempty
bounded O so destructive composition still sees that operation footprint.
Transparent flood skips shadow-only work while retaining original merge.
Singular outer affines have an empty bounded footprint. Unused kernel and offset
arithmetic is not required by these eliminated routes.

## Color, reconstruction and precision

Public `Color` channels are straight numeric-sRGB. Let b be displaced blurred
alpha and a_f=Color.a*resolvedOpacity. Shadow alpha is h=b*a_f, with that product
applied once. In the selected merge space its premultiplied RGB is floodRGB*h.
For original I, the actual merge is `M=I+H*(1-I.a)`; the original RGB stays sharp
and unshifted. `LinearSrgb` guardedly decodes original/flood at this merge and
encodes the merged result once. Alpha receives no transfer conversion. Gaussian's
consumed alpha mode performs no RGB transfer or unpremultiplication.

The Gaussian owner supplies normalized sampled 4-sigma FullNearest coefficients,
32-record nonaliasing `Rgba32Float` accumulation and the existing effective-zero
policy. No intermediate axis/batch transfer or clamp is added. Offset retains
proved aligned copies, nearest ties, bilinear and Mitchell reconstruction for
all resolved ImageRendering hints; signed cubic alpha is clamped there before
flood. CSS blur/shadow retain their separate approximation and shader contracts.

The complete outer affine is preserved. Maps are rebased before narrowing, and
actual reached f32 expressions and signed neighbor conversions qualify before
runtime. The existing signed i32 device-origin lattice still bounds captures;
large absolute coordinates are supported when that lattice and all reached maps
qualify, not through an arbitrary new origin cutoff. Refusals retain owning
Gaussian/offset causes or `Error::drop_shadow_diagnostic()` for shadow input and
merge-map failures. Active float32 capability refusal identifies
`RuntimeOperation::NativeDropShadowRendering`; skipped kernels do not require it.
There is no CPU production fallback.

A typed final O footprint follows the actual merge producer separately from
source alpha. It reconstructs per-cell operation coverage through the same full
map and bounded clamp-to-edge sampling as final composition. Clear, Copy,
SourceIn, SourceOut, DestinationIn and DestinationAtop preserve uncovered parent
pixels; Plus overrides retain their existing effective route. Outer mask/opacity
attenuate source, and author clip covers the whole completed operation. Ordinary
working-format quantization and hardware sampling error remain permitted;
footprint coverage and stored source alpha are not promised bitwise equality.

## Accounting and failure

`prepare_drop_shadow` records one central `DropShadow` attempt, one filter
operation and each reached scalar check (five coordinate visits on success),
without nested public Gaussian/offset attempts or fictional payload backing.
Actual shared pass facts, immutable preparation plan, images, kernels, uploads,
GPU resources and native handles retain their existing Ledger custody.

`ScheduledGaussianSamples` and `ScheduledOffsetSamples` describe their actual
reused stages. `ScheduledShadowSamples` counts merge sample opportunities: one
per finite original, four per periodic reconstruction, zero for a constant
original, plus one for a present shifted-alpha image, per scheduled merge cell.
`ScheduledFilterTexels` includes real rebase, preparation, batches, resolves,
offset and merge cells, including halo rows. Skipped projection/flood textures
are neither created nor charged. Phase timings are actual host/backend intervals,
not invented GPU timestamp durations.

All work remains provisional in the ordinary atomic frame transaction. Refusal,
cancellation and backend faults preserve prior published pixels and successful
statistics, release provisional custody and allow healthy retries using original
scenes and immutable images. See [frame accounting](reference.md#frame-accounting)
and [getting started](getting-started.md) for setup and explicit headless readback.

The semantic sources are the [drop-shadow clause](https://drafts.csswg.org/filter-effects-1/#feDropShadowElement),
[primitive subregions](https://drafts.csswg.org/filter-effects-1/#FilterPrimitiveSubRegion)
and [color interpolation](https://drafts.csswg.org/filter-effects-1/#ColorInterpolationFiltersProperty).
The retained [Filter Effects edition](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md)
contains the matching clauses. SVG syntax, units, names and document adapters
remain caller responsibilities.
