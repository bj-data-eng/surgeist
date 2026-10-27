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
no `rust-version` field. The existing integration guidance records Rust 1.97 as
the product integration floor; the [root manifest](../../../Cargo.toml) owns
the authoritative facade `rust-version`. Confirm that declaration when doing
integration work; this crate guide does not define a separate crate MSRV.

## Glyph admission

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
and retained logical custody. Charges include font bytes and conservative owner
headers, actual buffer capacities, fixed provider workspace and growth overlap.
These limits do not measure the allocator or bound CPU time. A failure discards
the entire unpublished result; the caller's inputs remain available for retry.
The authored `TextRun` route retains its existing provider behavior and supports
`Unspecified` bounds only on its direct route.

The optional `prepare_text_layout` adapter consumes the actual placed Text
snapshot without reshaping or choosing lines. Its complete `TextLayoutLimits`
budget also covers font copies, mapping buffers and decoration commands. Font
copies are shared within one invocation; the original Text storage remains an
input loan. The adapter consumes each run's retained orientation and physical
origin while preserving canonical glyph pens and offsets. Clockwise sideways
runs receive their exact right-angle basis before caller placement. Existing
horizontal synthesis operates in that canonical frame before rotation;
decorations are already physical and receive caller placement once. An
unqualified run orientation returns a typed `UnsupportedOrientation` refusal.

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
