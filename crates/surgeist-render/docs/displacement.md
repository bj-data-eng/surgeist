# Native displacement

Native displacement copies a premultiplied first-input texel selected by two
straight-byte map channels. Prepare the parameters and native map through
`RenderAccountingContext`, then pass both values and an authentic first source
to `Scene::displacement_layer`. The receiver replaces the Layer's exclusive
foreground effect while retaining its transform and outer composition.

## Prepare a native map

```rust
use std::sync::Arc;
use surgeist_render::{
    accounting::RenderAccountingContext,
    displacement::{
        DisplacementChannel, DisplacementInput, DisplacementSource, prepare_displacement,
        prepare_native_displacement_map,
    },
    effect::{FilterColorSpace, FilterDomains},
    geometry::{Rect, Size},
    paint::Color,
    scene::{Layer, Scene},
};

fn displaced_scene() -> surgeist_render::Result<Scene> {
    let bounds = Rect::try_new(0.0, 0.0, 4.0, 1.0)?;
    let mut accounting = RenderAccountingContext::default();
    let effect = prepare_displacement(
        DisplacementInput::new()
            .with_scale(4.0)
            .with_channels(DisplacementChannel::R, DisplacementChannel::G),
        FilterColorSpace::Srgb,
        &mut accounting,
    )?;
    let map = prepare_native_displacement_map(
        Size::try_new(1.0, 1.0)?,
        Arc::<[u8]>::from([255, 128, 0, 255]),
        bounds,
        &mut accounting,
    )?;
    let mut input = Scene::new();
    input.fill(bounds, Color::BLACK);
    let mut scene = Scene::new();
    scene.displacement_layer(
        Layer::new(), effect,
        DisplacementSource::finite(
            input, FilterDomains::try_new(bounds, bounds, bounds)?,
        ),
        map,
    )?;
    Ok(scene)
}
```

At unit density this map selects horizontal offset+2 and vertical offset0.
Source addresses outside the declared input are transparent. The input Scene
moves into the existing iterative Layer child owner; map clones share immutable
accounted pixels rather than introducing another Scene tree.

`DisplacementInput` defaults to scale0 and A/A. Its signed scale must be finite;
finite large values remain admitted until actual frame arithmetic is reached.
`PreparedDisplacementMap::source_graphic()` and `source_alpha()` refer to this
receiver's actual input and select semantic pass-through. `suppressed()` requests
conservative pass-through without a pixel payload.

Native bytes declare a caller-authored value. They carry no document-origin or
filter-taint certificate. There is no implicit eligible-map conversion from
`Image` or `Scene`, and no clean flag. Document input resolution and its
source-identity-bound, transitive taint producer must establish document map
eligibility separately. This native interface cannot determine whether manually
supplied bytes were copied from document content.

## Independent domains and clipping

S declares the finite first input, M positions the native map, F is the common
input/filter clip, and P is the primitive result. O=P intersect F. A finite input
has D=S intersect F; a declared constant or fundamental-period source has D=F.
All first sources are clipped to F before lookup. Constant and periodic input
therefore retain transparent exterior beyond F; the Gaussian infinite-source
exception does not apply here.

Map lookup uses the undisplaced O-cell location. Source realization may retain
only W, the source cells reachable from O under the checked offset envelope.
That allocation window does not add another coverage clip, and source texels
outside O remain available when an offset reaches them. Input D coverage is
applied once before lookup, then output O coverage once afterward. Those are
independent image operations, including when their fractional boundaries agree.

A map cell intersects its complete raster-cell footprint with M intersect F.
Partly covered O cells retain that full footprint for map evaluation, so a map
can influence the cell even when the continuous rectangles M and O merely touch.
Map coverage changes map channels; it does not attenuate output color directly.

The scalar density comes from the existing full-affine capture qualification.
Final placement retains the complete outer transform. Periodic input captures
its actual tile with per-axis mapping and scoped context, reconstructs its seam
onto the common lattice, and is then clipped to F. Displacement itself copies
one integer source texel. Finite same-layer backdrop joins the source once;
constant/periodic input rejects same-layer backdrop before appending a command.
Masks, opacity and author clips apply after the effect. Bounded operation
coverage remains separate from source alpha for destructive Porter-Duff
operators, Plus override and singular transforms.

## Native map bytes and addressing

Pixels are straight numeric-sRGB RGBA8. M independently scales and positions
that finite image. For each O column and row, Render computes an integer native
pixel index and coverage in ordered binary64. A partially covered cell clamps
its center to M before choosing the containing native cell. Ties choose the
higher cell; the far endpoint chooses the final cell. Fully exterior map cells
supply zero channel data. No Image extension or quality policy is inherited.

Raw alpha0 suppresses hidden RGB. Positive raw alpha and positive unquantized
coverage preserve each selected straight RGB byte, followed by map-only color
conversion. Alpha uses `floor(raw_alpha_byte * coverage + .5)`, clamped to a byte.
It has no color transfer. A covered alpha byte can round to0 while RGB remains
nonzero. This policy preserves low-alpha straight bytes through native upload;
it does not reproduce an intermediate premultiplied8-bit map roundtrip.

Srgb map RGB is unchanged. LinearSrgb uses the existing color conversion owner
and half-up byte quantization. A single256-value conversion table is folded into
RGB address tables when required. First-input RGB is never transferred by this
primitive.

At frame density rho, each selected byte c uses these separately ordered
binary64 operations, without fused multiply-add:

```text
S = scale * rho
coefficient = S / 255
half = .5 * S
bias = .5 - half
product = coefficient * c
value = product + bias
integer_offset = trunc(value)
```

Thus scale+4 maps bytes0/255 to offsets−1/+2; scale−4 gives+2/−1. This specifies
native evaluation precision in addition to the selected byte-copy profile.
Reached nonfinite arithmetic is a typed refusal. Finite enormous offsets proved
outside D remain transparent without an enormous integer cast or capture.
Actual asymmetric extrema come from the table, rather than a symmetric radius.

The first host traversal classifies finite truncated offsets and derives W.
The second emits512 final address entries, each containing W-relative `q0:i32`
and an outside flag with8-byte storage stride. Every reached `q0+output_index`
is qualified before execution. Map records have16-byte stride and are computed
once per O column/row. Positive uploaded fractions and their smallest positive
product must remain normal f32. Shader addressing uses these integers, not a
second float nearest/floor calculation. Alpha quantization uses the actual f32
coverage multiplication/addition; ordinary rounding at its threshold is part of
the native policy.

## Omission and resource custody

Scale0 and closed semantic/suppressed maps select pass-through before importing
or retaining unused map pixels into normalized/frame payloads. The separately
requested native-map preparation still owns its constructor and retention cost.
An image with either zero dimension requires an empty payload; an empty M also
supplies zero map data. These valid zero maps remain active and can shift input
through byte0. They omit raw upload and map records while retaining active
address-table execution. Empty O has no operation; provably empty input avoids
unneeded map/address work while preserving bounded transparent composition.

Actual source images, raw Rgba8Unorm maps and address/record storage buffers have
distinct typed roles, generations, read counts and final uses. The existing
ResourceManager, frame leases and Generation ledger own their physical backing.
Raw upload preserves exact immutable-image identity and unique CPU import
custody, independently of the source texture. Arrays live in accounted shared
payloads rather than Layer or frame future fields. Native uniform backing shares
the graph's [parameter-buffer custody](reference.md#device-generation-custody)
and stays with the pending transaction after logical consumption. Rejection, cancellation
and device failure preserve previous successful pixels/statistics; identical
input can retry after a recoverable refusal.

Submission assembly keeps the real encoder or completed command buffer with
its pending capture and prepared-frame owners through every fallible readiness
and extraction step. Native commands dispose before their graph resources on
abort. A private device-limit snapshot qualifies reached table/record lengths
and binding counts before known-impossible backing, then actual allocation
checks remain active at the resource owner.

## Accounting and diagnostics

Native-map preparation observes actual identity bytes through
`DisplacementMapByteVisits`. Coordinate metadata uses `CoordinateVisits`;
`DisplacementMapRecordVisits` counts constructed axis records, and
`DisplacementTableVisits` counts512 final address entries plus the256-entry
linear RGB lookup when reached. Cache/proof byte and record comparisons retain
their existing separately metered owners. Serialization, uploads, native
allocations, retained storage, graph/resource work and phase timings use the
central Ledger.

For N scheduled output cells, active displacement records these fixed
opportunities, including conditional/exterior opportunities:

| Counter | Native map | Valid zero map |
| --- | --- | --- |
| `ScheduledDisplacementMapRecordReads` | 2N | 0 |
| `ScheduledDisplacementAddressReads` | 2N | 2N |
| `ScheduledDisplacementMapSamples` | N | 0 |
| `ScheduledDisplacementSourceSamples` | N | N |

These counts describe scheduled work, not observed hardware instructions.
Pass-through/source realization uses existing filter-texel/image-pass accounting
and records0 in all four columns. Missing cached assets and all graph parameter
buffers, including displacement uniforms, are checked together at the generation
root, then at their real cache and External/PassBuffers scopes. Actual allocation
checks remain active after preview; retained idle backing remains charged.

`DisplacementPreparationError` distinguishes invalid scale, map size/payload,
map domain, reached mapping/table/index arithmetic, empty periods and rejected
infinite backdrop. Runtime raw-format refusal identifies
`RuntimeOperation::DisplacementRendering`. Capability and central resource,
work, cancellation and device failures retain their owning typed diagnostics.

The new opaque native types are additions. Adding
`PrimitiveOperation::GpuDisplacementFilterExecution` to the exhaustive enum and
the seven required `RenderWorkFields<T>` fields is source-breaking for
exhaustive matches and struct literals. `RenderAttemptKind` additions remain
non-exhaustive. There are no compatibility aliases.

See [reference](reference.md), [offset](offset.md) and
[morphology](morphology.md) for adjacent contracts. The
[Filter Effects displacement clause](https://drafts.csswg.org/filter-effects-1/#feDisplacementMapElement)
defines inverse lookup and map-only color conversion; the selected pinned byte
profile is recorded in the retained
[WebKit witness](../../../references/webkit-filter-backface-witnesses--73aa6c89e2cb--5a2ab9f34073.md).
Interpolation and ordered native precision above are explicit native policy.
