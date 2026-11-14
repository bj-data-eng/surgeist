# Native image-space offset

`prepare_offset` admits a resolved layer-local translation and reconstruction
intent. `Layer::with_offset` selects that operation in the exclusive foreground
effect slot. It captures ordinary Image, vector and nested Layer children,
includes a supported same-Layer backdrop, and executes through the GPU graph.

```rust
use surgeist_render::{
    FilterDomains, ImageRendering, Layer, OffsetInput, Rect,
    RenderAccountingContext, prepare_offset,
};

# fn example() -> surgeist_render::Result<()> {
let source = Rect::try_new(0.0, 0.0, 40.0, 20.0)?;
let region = Rect::try_new(0.0, 0.0, 64.0, 32.0)?;
let mut accounting = RenderAccountingContext::default();
let offset = prepare_offset(
    OffsetInput::new()
        .with_dx(3.5)
        .with_dy(-1.0)
        .with_rendering(ImageRendering::Smooth),
    FilterDomains::try_new(source, region, region)?,
    &mut accounting,
)?;
let layer = Layer::new().with_offset(offset);
# let _ = layer;
# Ok(())
# }
```

Both displacements default to zero; the reconstruction hint defaults to Auto.
Preparation rejects nonfinite dx or dy. The admitted value is immutable and
inline: it needs no coefficient allocation or shared-payload import lease.

## Domains and placement

`FilterDomains` keeps independent source **S**, hard filter **F**, and primitive
result **P** rectangles. All coordinates, nonnegative extents and derived maxima
are finite; empty rectangles are valid. Invalid domain rectangles use
filter-labelled `InvalidValue` diagnostics. Convolution also consumes this shared
value directly.

The input image is **D = S ∩ F**, and the output is **O = P ∩ F**. Each captured
source cell receives its fractional D coverage once before reconstruction.
Sampling outside the finite input is transparent. Each output cell receives its
independent fractional O coverage afterward. A half-covered source cell shifted
half a texel contributes alpha one quarter to each neighboring output cell under
bilinear reconstruction; clipping half of one output cell reduces that cell to
one eighth.

For local output point p, sample input point `p - (dx,dy)`. Translation does not
move F or P. The complete outer affine transform subsequently places p; layer
clip, mask, opacity and composition retain their existing meanings. Output may
extend beyond S. A wider completed backdrop is copied onto the finite D capture
lattice before sampling, without adding source coverage or a second resampling
stage.

Empty D or O produces transparent effect output without capturing an excluded
input. Transparent-source composition still applies: operators such as Copy can
replace the parent with transparent pixels. Empty children alone do not exclude
a same-Layer backdrop.

## Reconstruction policy

Offset samples the captured premultiplied numeric RGBA directly. It performs no
color-space transfer conversion or unpremultiplication.

| Actual mapping | Reconstruction |
| --- | --- |
| Proven aligned lattices and integral texel translation, every hint | Exact one-cell copy with transparent exterior. |
| Fractional Auto, Smooth or Pixelated | Four-tap bilinear. Pixelated's translation-only enlargement factor is one. |
| Fractional CrispEdges | Nearest, with ties selected by `floor(source_texel + 0.5)`. |
| Fractional HighQuality | Sixteen-tap separable Mitchell–Netravali, B=C=1/3. |

HighQuality retains signed cubic contributions until reconstruction ends, then
clamps alpha to [0,1] and RGB to [0,alpha]. It does not silently degrade to
bilinear. The exact-copy rule prevents cubic smoothing at zero or integral
translation. These concrete kernels, ties and clamp rules are native policy;
they are not a claim of universal browser pixel equivalence.

## Qualification and accounting

Planning derives the inverse mapping from actual source/result capture lattices
in f64, then uploads rebased texel coordinates. It qualifies reached f32
arithmetic, coverage, integer conversion and the complete sampling neighborhood
before execution. Huge finite displacements may produce a proved transparent
result; otherwise unrepresentable mapping or addressing receives typed refusal.
Ordinary WGSL f32 rounding and intermediate subnormal flushing are permitted.
Final layer composition independently rebases its source-texel and
mask-normalized affine maps from the actual capture descriptors before narrowing.
Large logical origins therefore do not erase their separate texel phases;
authored clip coordinates retain their existing owner.

`Error::offset_diagnostic()` distinguishes invalid parameters from numeric
GridMapping/GridIndex refusals. Device, capability and accounting failures retain
their existing diagnostic channels. Refusal, cancellation and execution failure
preserve the previous complete pixels and successful statistics.

Central Ledger reports standalone `RenderAttemptKind::Offset` preparation and
actual frame resources, serialization, upload and scheduling. `ScheduledFilterTexels`
includes each executed offset and required raw-rebase output cell.
`ScheduledOffsetSamples` counts scheduled cells times the selected fixed footprint
(1, 4 or 16), including transparent exterior taps and one fetch per raw-rebase
cell. It does not estimate unpredictable driver fetches or charge coefficient
visits for this coefficient-free operation. See [frame accounting](reference.md#frame-accounting).

Scenes own their authored layer metadata separately from child command storage.
Normalized layer metadata uses the existing central immutable backing owner;
its real allocation, refusal cleanup and final release remain accounted.
Command clones share that immutable metadata, while qualification copies it
before mutation. This container backing is separate from the inline offset value.
