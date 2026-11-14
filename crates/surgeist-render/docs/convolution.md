# Native convolution

Prepare a resolved convolution once, install it on a `Layer`, and render the
layer's child `Scene` through the ordinary Renderer and Surface APIs. The child
may contain vector draws, immutable Images, text or nested layers. Rendering
uses GPU capture, convolution and composition; completed pixels become visible
to the CPU only through explicit headless readback.

Render receives numeric parameters and three rectangles in the receiving
layer's local logical units. The caller resolves SVG syntax, units, resource
references and computed working color before preparing the operation. This
receiver implements one foreground convolution. It does not parse SVG or expose
a generic filter graph.

## Prepare and draw

```rust
use surgeist_render::{
    Color, FilterDomains, ConvolutionInput, FilterColorSpace, Layer, Rect,
    RenderAccountingContext, Result, Scene, prepare_convolution,
};

fn sharpened_scene(accounting: &mut RenderAccountingContext) -> Result<Scene> {
    let domain = Rect::try_new(0.0, 0.0, 32.0, 32.0)?;
    let domains = FilterDomains::try_new(domain, domain, domain)?;
    let kernel = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    let operation = prepare_convolution(
        ConvolutionInput::new(&kernel).with_preserve_alpha(true),
        domains,
        FilterColorSpace::LinearSrgb,
        accounting,
    )?;
    let ink = Rect::try_new(8.0, 8.0, 16.0, 16.0)?;
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_convolution(operation), |children| {
        children.fill(ink, Color::BLACK);
    });
    Ok(scene)
}
```

The omitted order is 3×3 and the kernel sum supplies divisor 1. The operation
owns admitted coefficients; it does not borrow the local array after preparation.
Cloning a `PreparedConvolution` shares its immutable backing. A clone still
qualifies against the actual device and execution grids when rendered.

The layer has one exclusive foreground-effect slot. `with_convolution` replaces
an existing foreground filter choice, and the existing foreground filter setters
replace convolution. Nest layers to compose successive unary effects. The
receiving layer's clip, mask, opacity and composition apply after its foreground
effect. See [getting started](getting-started.md) for Renderer, Surface and
explicit readback setup, and [layer composition](reference.md#layer-composition-and-isolation)
for the surrounding operation.

## Resolved parameters and recovery

Order values truncate toward zero and must produce positive representable
integers. The omitted pair is 3×3; an omitted second value copies the first.
Omitted targets independently use `floor(order / 2)`. Supplied targets must be
integers inside their corresponding order dimension. Kernel storage is original
row-major order. There is no semantic 3×3, 9×9 or 32-tap maximum.

After domain, order and order-product validation, a coefficient-count mismatch
prepares a successful pass-through operation. It samples the primary input
unchanged within the source and result domain clips. It bypasses kernel
resampling, color conversion, coefficients, divisor and bias. Unused invalid
coefficient, target or bias values cannot turn this recovery into another effect.
Invalid dimensions still fail before the count test.

For matching counts, all reached coefficients and parameters must be finite.
An omitted or explicit zero divisor requests the checked `f64` row-major kernel
sum; zero sum selects divisor 1. Any other finite signed divisor is used directly.
An explicit nonzero divisor does not calculate an unnecessary raw kernel sum.
Overflow in a requested default sum is a typed numeric refusal.

Bias defaults to zero, `preserveAlpha` defaults to false, and edge mode defaults
to `FilterEdgeMode::Duplicate`. The caller supplies `FilterColorSpace::Srgb`
or `FilterColorSpace::LinearSrgb`; Render does not infer an SVG default from a
CSS-filter setting. A consumer implementing SVG's default selects linear-sRGB.

Missing `kernelUnitLength` uses one actual offscreen capture pixel on each axis.
`with_unit_length(x, None)` requests equal x/y spacing. If either explicit spacing
is nonfinite or nonpositive, the pair falls back to the source lattice. A valid
positive pair that cannot be represented on the execution grid produces a typed
refusal instead of falling back to another effect.

## Domains and the two resampling stages

`FilterDomains::try_new(source, filter, result)` keeps three rectangles:
source image domain **S**, hard filter region **F**, and primitive result **P**.
Their origins, extents and maxima must be finite; extents may be zero and cannot
be negative. Allocation padding and the final Surface viewport are separate.
Convolution domain arguments use this shared filter value, with the same
three-rectangle constructor and independent `source()`, `filter()` and `result()`
accessors. Invalid domain rectangles produce filter-labelled `InvalidValue`
diagnostics through `Error::invalid_value_diagnostic()`.

The captured input image is clipped to **D = S ∩ F** before resampling. Pixels
whose footprints partially intersect D retain fractional boundary coverage,
applied once to the rasterized input image. Pixels outside the finite source
image cells do not contribute. The first bilinear rescale clamps to that finite
clipped capture's boundary cells. It does not use the convolution edge mode to
invent extra source cells, discard partially intersecting cells by their center,
or apply source coverage twice.

The kernel lattice has boundary origin **o** equal to the actual capture lattice
boundary origin **c**, and independent positive axis steps **h = (dx, dy)**.
On each axis, its finite source cells are exactly

```text
I = [floor((D.min - o) / h), ceil((D.max - o) / h))
```

Empty D has no source cells on either axis, even when its other extent is large.
It uses a bounded transparent 1×1 GPU input without qualifying unused source
coordinates. The first resampling stage stores nonempty finite I in
premultiplied selected working color. A source cell whose footprint intersects D
is retained even when its center lies outside D.

A supported backdrop on the same Layer joins the children before convolution.
When the completed group's capture extends beyond D, a GPU copy rebases it onto
the finite D capture lattice before preparation. This copy preserves channel
values and applies no color transfer or fractional coverage; preparation still
applies source coverage exactly once. Pixels outside D cannot move the explicit
kernel grid's origin. Empty D excludes the backdrop input as well as the children.

Convolution evaluates a separate finite result grid **J** with enough halo cells
for bilinear reconstruction of the native output pixels intersecting **P ∩ F**.
Each helper cell in J samples only edge-extended I. J may extend beyond I and P;
its temporary halo is not published. The second stage bilinearly resamples the
completed premultiplied result, alpha included, back to the ordinary capture
grid. It then converts color back to numeric sRGB and applies fractional P/F
coverage. A small P therefore never removes contributing source pixels outside P.
P may also extend beyond S.

The edge modes address finite I independently on each axis:

| Mode | Addressing outside I |
| --- | --- |
| `Duplicate` | Use the nearest boundary cell. |
| `Wrap` | Repeat with Euclidean modulo over the cell extent. |
| `Mirror` | Reflect with period `2N`, repeating boundary cells; fold `t >= N` to `2N - 1 - t`. |
| `None` | Return transparent black. |

Empty input supplies zero samples for every mode without modulo or division by
zero. With `preserveAlpha = false`, nonzero bias can still create output from an
empty captured scene. Empty P/F intersection produces no result.

A distinguishing resampling example uses a two-cell premultiplied source
`(1,0,0,1), (0,0,0,0)` over x = [0,2], capture centers .5 and 1.5, an identity
1×1 kernel, `preserveAlpha = true`, and dx = 2. The sole prepared source cell at
x = 1 is `(.5,0,0,.5)`. Duplicate, wrap and mirror produce final alpha .5/.5 at
the original centers. None produces .375/.375 because the identity-convolution
halo outside I is zero. Restoring alpha 1/0 from the original capture after the
second resampling stage would change this selected native behavior.

## Pixel equations and working color

For output kernel index `(x,y)`, the exact semantic convolution samples
`(x - targetX + j, y - targetY + i)` with coefficient
`kernel[(orderY - 1 - i) * orderX + (orderX - 1 - j)]`. Reversing both coefficient
axes distinguishes convolution from correlation.

Let `S` denote an unnormalized sum and D the nonzero divisor. With
`preserveAlpha = false`, first calculate

```text
rawA = Sa / D + bias
rawC = Sc / D + bias * rawA
A = clamp(rawA, 0, 1)
C = clamp(rawC, 0, A)
```

RGB bias uses raw alpha before clipping. For opaque black, kernel `[2]`, divisor
1 and bias .25, the completed premultiplied result is
`(.5625,.5625,.5625,1)`. Applying outer layer opacity .5 subsequently gives
`(.28125,.28125,.28125,.5)`.

With `preserveAlpha = true`, each prepared working-grid sample is guardedly
unpremultiplied, straight RGB is convolved, bias is added, straight RGB is clamped
to [0,1], and the corresponding edge-extended kernel-grid source alpha is
preserved. The result is premultiplied before output resampling. Alpha is neither
convolved nor biased in this branch, and is not restored from the original
capture afterward.

For linear-sRGB, the source capture's numeric-sRGB RGB is unpremultiplied,
decoded with the standard sRGB transfer, and premultiplied before the first
rescale. The completed result is encoded back after output resampling. sRGB
operation skips these transfer conversions. Alpha never passes through an RGB
transfer function.

## Local axes and outer transforms

Let A be the complete layer-local-to-surface affine, ρ the chosen capture density,
c the capture boundary origin, and o = c the kernel boundary origin. Componentwise,

```text
capture center(i) = c + (i + .5) / ρ
kernel center(j)  = o + (j + .5) * h
capture coordinate of local q = (q - c) * ρ
kernel reconstruction coordinate of local p = (p - o) / h - .5
surface point = A * p
```

The actual surface-to-device scale applies after A. Capture density can depend
on A's largest singular value, but A retains its full nonuniform scale, rotation,
shear and translation for final composition. That scalar density cannot replace
A or collapse unequal dx/dy into one spacing. Rank-deficient outer transforms
follow the existing zero-area destination behavior. Concrete coordinate,
index-precision and device limits still apply.

## GPU resources, precision and refusal

Preparation computes normalized weights `w = k / divisor` in checked `f64`, then
admits zero or finite normal `f32` weights and bias. A nonzero quotient that
underflows in host division or becomes zero/subnormal when narrowed is refused.
Raw coefficient and divisor magnitudes alone do not determine admission:
`[2^-130] / 2^-130` and `[2^130] / 2^130` both produce weight 1. The GPU does not
divide these normalized sums by the divisor again.

Execution uses fixed batches of at most 32 taps in deterministic original
coefficient order. All admitted taps execute. Prepared source cells, signed
partial sums and completed result-grid cells use separate `Rgba32Float` images,
manual texture loads and bilinear interpolation. No blending or intermediate
clamp destroys negative values or cancellation. The ordinary working format is
used after primitive normalization. Reduced working precision does not substitute
UNORM accumulation for the required signed float resources.

Ordinary WGSL `f32` rounding and intermediate flush-to-zero are permitted in
conversion, interpolation, accumulation and finalization. This contract promises
neither universal subnormal preservation nor a global absolute-error bound
against ideal real arithmetic. Before any unpremultiplication, alpha below
`f32::MIN_POSITIVE` (2^-126) is effective zero and yields zero RGB; source-alpha
preservation carries that same effective-zero decision. Tiny RGB can also flush.
Consequently even significant straight color at extremely small alpha can be
lost, including color that convolution would otherwise transfer to an opaque
neighbor. This is not a bounded final-alpha-error guarantee.

Overflow admission is separate. With N taps, B batches, L1 the checked sum of
absolute admitted weights and b the admitted bias, preparation calculates

```text
R  = (1 + 2^-21)^(2*N + 2*B + 16)
U  = R * L1
Ua = R * (U + abs(b))
Uc = R * (U + abs(b) * Ua)
```

All bounds must be finite and at most `f32::MAX / 4`. This conservative margin
can refuse a mathematically finite ill-conditioned kernel. Grid construction
also checks signed addressing, finite f32 mapping, distinguishable neighboring
coordinates, representable allocation extents and actual device capabilities.
The mapping proof includes composed sampling coordinates before finite-input
clamping and the signed neighboring indices used by bilinear reconstruction.
Unrepresentable arithmetic refuses with `GridMapping` or `GridIndex`; empty
input skips only expressions its execution branch does not evaluate.
Required float attachment/sampling support and texture, buffer and binding limits
must qualify before execution. There is no CPU production fallback.

Preparation uses central `RenderAccountingContext` limits and shared admitted
backing. Frames import that backing through existing renderer custody. Ledger
meters reached coefficient, coordinate, planning, serialization and upload work,
plus actual resource capacity. `ScheduledFilterTexels` includes scheduled
input rebasing, preparation, accumulation, finalization and output-resampling cells, including
halo and padding. `ScheduledConvolutionTaps` records actual scheduled result-grid
cells × batch taps separately from CPU coefficient visits. Source and temporary
resources remain leased through their validated last uses.
Spatial-route classification reads a constant-size convolution-presence fact
derived and validated during the existing accounted graph traversals.

Invalid resolved parameters and numeric limitations expose
`Error::convolution_diagnostic()` with `ConvolutionPreparationError` and the
relevant parameter or numeric phase. Accounting and runtime capability failures
retain their existing typed diagnostics. A failed frame preserves the previous
complete pixels and successful statistics; it never substitutes identity pixels
for a refused executable convolution. See [frame accounting](reference.md#frame-accounting)
and [capabilities and errors](reference.md#capabilities-and-errors).

## Contract sources

The [Filter Effects convolution clause](https://drafts.csswg.org/filter-effects-1/#feConvolveMatrixElement)
and [retained Filter Effects edition](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md)
provide the primitive equations, defaults, domains and resampling requirements.
Mirror follows the [adopted CSSWG resolution](https://github.com/w3c/fxtf-drafts/issues/527).
The finite I/J grids, capture-aligned origin, bilinear resampling, batch order and
numeric admission above are explicit native execution policies. They do not
assert convergence with every browser implementation. WGSL's
[floating-point evaluation rules](https://www.w3.org/TR/WGSL/#floating-point-evaluation)
permit the stated rounding and flushing behavior.
