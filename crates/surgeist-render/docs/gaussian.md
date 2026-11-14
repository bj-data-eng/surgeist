# Native Gaussian blur

Prepare independent resolved standard deviations, declare an actual source, and
append it with `Scene::gaussian_layer`. The ordinary Renderer, Surface and atomic
publication transaction execute the operation on the GPU. The source can contain
Images, vector draws and nested layers. SVG syntax, unit resolution and named
filter graphs belong to the caller.

## Build a finite source

```rust
use surgeist_render::{
    Color, FilterColorSpace, FilterDomains, GaussianInput, GaussianSource, Layer,
    Rect, RenderAccountingContext, Result, Scene, prepare_gaussian,
};

fn blurred_scene(accounting: &mut RenderAccountingContext) -> Result<Scene> {
    let domain = Rect::try_new(0.0, 0.0, 32.0, 24.0)?;
    let operation = prepare_gaussian(
        GaussianInput::new().with_standard_deviations(2.0, 1.0),
        FilterColorSpace::LinearSrgb,
        accounting,
    )?;
    let mut input = Scene::new();
    input.fill(Rect::try_new(8.0, 6.0, 16.0, 12.0)?, Color::BLACK);
    let source = GaussianSource::finite(
        input, FilterDomains::try_new(domain, domain, domain)?,
    );
    let mut output = Scene::new();
    output.gaussian_layer(Layer::new(), operation, source)?;
    Ok(output)
}
```

`GaussianInput::new()` and `Default` request zero on both axes.
`with_standard_deviation(s)` sets both axes; `with_standard_deviations(x,y)`
sets them independently. Preparation inspects both values in X/Y order and
rejects nonfinite input. Either negative value disables filtering. Both zero
select identity, and one zero omits that axis's kernel and passes. Signed zero
is zero. Finite positive values have no CSS blur cap, but the actual grid,
coefficient buffer, device and accounting limits must qualify when rendered.

The receiver replaces the layer's exclusive foreground effect. Its transform,
outer clip, mask, opacity and composition remain outside Gaussian evaluation.
Nest layers to apply consecutive foreground effects. Source commands move into
the existing iterative scene child owner; metadata never retains a second Scene.
Clone, inspection and destruction keep the existing bounded-stack behavior.

## Finite, constant and periodic inputs

`GaussianSource::finite(scene, domains)` selects transparent `None` extension.
`finite_with_edge(scene, domains, edge)` chooses `FilterEdgeMode::Duplicate`,
`Wrap`, `Mirror` or `None`. On each axis, duplicate clamps to the boundary cell,
wrap uses Euclidean remainder, and mirror reflects a `2N` period with repeated
boundary cells. Empty sources have no samples, including one-zero-axis domains.

For finite sources, `FilterDomains` keeps source S, filter F and result P
independent. Input D = S intersect F receives fractional raster-cell coverage
once, before filtering. Result O = P intersect F receives independent coverage
after filtering. A smaller P does not crop contributing input. The horizontal
intermediate retains every row required by vertical support, including rows
outside P. Both axes address the original prepared D; vertical filtering reads
the computed horizontal halo rather than extending a cropped intermediate.
Identity and disabled finite input are clipped copies without edge extension or
color transfer. Empty O schedules no output. Empty D or ink remains transparent
within the finite output operation footprint, so non-preserving composition can
still clear that footprint. Gaussian carries explicit per-cell O coverage,
separately from source alpha, through final composition. Under the full affine,
that coverage follows the same bounded clamp-to-edge reconstruction as the output
image. Destructive Porter-Duff operations retain uncovered destination pixels;
source alpha, masks and opacity do not redefine the operation footprint. Authored
clips still cover the whole resulting operation. Singular outer affines have an
empty bounded Gaussian footprint. Storage quantization and hardware sampling
rounding permit ordinary format-dependent error; reconstructed coverage and
sampled source alpha are not promised bitwise equality.

Finite same-layer backdrop joins the children before Gaussian. If backdrop
capture enlarges the group, a checked raw GPU rebase restores D's lattice before
preparation. This copy applies neither fractional coverage nor color conversion.
The complete outer affine, including reflection, skew and unequal scale, remains
in final composition; capture density does not replace that affine.

`GaussianSource::constant(color, filter, result)` declares a constant
unbounded input. Gaussian evaluates it analytically: the same numeric
premultiplied color, with only final O coverage and outer layer semantics. It
allocates no kernel and performs no transfer round trip. A finite constant fill
with `None` extension instead darkens near D's boundary.

`GaussianSource::periodic(tile_scene, period, filter, result)` owns an actual
fundamental period T with positive finite dimensions. At selected density rho,
it captures an integer canvas `Nx=max(1,ceil(T.width*rho))` by
`Ny=max(1,ceil(T.height*rho))`. Its separate axis scales are Nx/T.width and
Ny/T.height. Nested content is captured into that texel canvas through a scoped
frame context; following siblings and later periods recover their own context.
No unrelated global-grid partial-cell attenuation is introduced at the seam.
Filtering wraps the full period, and output uses manual four-neighbor wrapped
bilinear reconstruction. Distant output phase is reduced on the host before
narrowing the local map. Zero or negative requests use the same actual periodic
reconstruction while omitting kernels and transfer conversion.

Declared infinite inputs use their specific constant or periodic extension during
evaluation; F and P clip the published result. This native precedence resolves an
overlap between the common input hard-clip rule and the specific infinite-source
clauses. It is an explicit native policy, not a claim of normative consensus or
browser equivalence. Infinite input is never inferred from a large finite shape.
The fallible receiver rejects same-layer backdrop for constant and periodic
sources before appending, because backdrop could invalidate that declaration.
Nested finite content inside a tile retains its supported local backdrop rules.

## Approximation, color and resources

Native Gaussian uses normalized sampled coefficients `exp(-k²/(2*s²))` with
integer support R = ceil(4*s) on each actual axis grid. It serializes center and
signed offsets through the existing Gaussian kernel owner as `FullNearest`
records. CSS blur and shadow retain their existing scalar, 2.5-sigma paired
sampling contracts.

This is a sampled and truncated Gaussian, not exact continuous integration.
For R positive, omitted unnormalized mass is bounded by
`T_tail <= (2*s²/R)*exp(-R²/(2*s²))`. If Z is retained unnormalized mass, the L1
difference from the infinite sampled normalized distribution is at most
`2*T_tail/Z`. This bounds truncation only; it does not bound sampling
discretization, texture quantization or every WGSL operation. Rounded or flushed
subnormal coefficient mass is at most `tap_count*f32::MIN_POSITIVE`, with no
subsequent divisor amplification.

For `LinearSrgb`, preparation guardedly unpremultiplies captured numeric-sRGB,
decodes the transfer and repremultiplies once. Every batch and axis operates in
premultiplied linear space. After final reconstruction, resolve encodes once and
repremultiplies. `Srgb` operates directly on numeric premultiplied components.
Alpha below `f32::MIN_POSITIVE` is effective zero before division. Final alpha
is clamped to [0,1] and RGB to [0,alpha] before result coverage. Ordinary f32
rounding and flush-to-zero are permitted; no universal real-arithmetic error
bound or subnormal-preservation guarantee is implied.

At most 32 actual coefficient records execute in one accumulation draw. Larger
axes carry sums through distinct `Rgba32Float` images with no intermediate
transfer conversion, clamp or half-float storage. The shared float32 capability
requires render/sample/copy usages without hardware filtering. Constant and
pass-through paths do not require it. There is no CPU production fallback.

Exact geometry, record count, serialized bytes, addressability and actual device
buffer limits are checked before allocating or generating a known-unusable
kernel. Grid qualification checks reached uploaded-f32 products, additions,
indices, halo endpoints and reconstruction neighbors. Refusals retain typed
`Error::gaussian_diagnostic()`, accounting or runtime capability diagnostics;
no refused active filter silently becomes identity.

## Accounting and migration

`prepare_gaussian` records a central Gaussian preparation attempt with reached
parameter checks and no invented coefficient payload. Kernel generation,
normalization, serialization and comparisons use the existing Kernel phase and
`KernelSampleVisits`. Actual backing, uploads, resources and leases stay in the
same Ledger and provisional frame transaction as other native filters.
`ScheduledFilterTexels` counts each scheduled preparation, rebase, batch, axis and
resolve cell, including finite halo rows. `ScheduledGaussianSamples` counts
scheduled source and previous-partial sample opportunities; periodic resolve has
four per output cell. Transparent-guarded taps remain scheduled opportunities.
Zero axes and analytic constants incur only work they execute. Optional phase
timings are host interval observations, not GPU timestamp measurements.

Failure or cancellation preserves previous pixels and successful statistics,
releases provisional resources and permits retry with the original authored
scene and shared immutable Image handles. See [frame accounting](reference.md#frame-accounting)
and [getting started](getting-started.md) for Renderer and explicit readback setup.

This pre-v1 API directly replaces `ConvolutionEdgeMode` with `FilterEdgeMode`,
and exposes `supports_filter_float32` and `FilterFloat32Unavailable` for shared
native-filter precision. There are no compatibility aliases. Convolution retains
its operation-specific duplicate default and diagnostic identity; Gaussian finite
input defaults to none.

The [Gaussian clause](https://drafts.csswg.org/filter-effects-1/#feGaussianBlurElement),
[primitive subregions](https://drafts.csswg.org/filter-effects-1/#FilterPrimitiveSubRegion)
and [filter color interpolation](https://drafts.csswg.org/filter-effects-1/#color-interpolation-filters-property)
provide the semantic sources. The retained [Filter Effects edition](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md)
records the matching clauses. [Native resolved drop shadow](native-shadow.md)
reuses this Gaussian owner in a private alpha-only route, retaining the original
for its final colored merge. SVG document adaptation remains separate.
