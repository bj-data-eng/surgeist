# Reference

## Package and features

[Cargo.toml](../Cargo.toml) defines package `surgeist-text` version `0.1.0`, library
name `surgeist_text`, and Rust edition 2024. No `rust-version` minimum is declared.
The implementation uses Parley `0.9.0` in [src/system.rs](../src/system.rs), with
Fontique `0.9.0`, Skrifa `0.42.1`, and Linebender resource handle `0.1.1` called
directly for catalogue registration and admission.

| Feature | Default | Effect and dependency |
| --- | --- | --- |
| `text-accessibility` | Off | Enables owned `Accessibility` and layout-to-AccessKit methods; selects AccessKit `0.24.1`. |

The default feature set is empty. Owned accessibility projection is implemented in
[src/accessibility.rs](../src/accessibility.rs). Render owns its optional standalone
paint adapter under `render-text` and consumes this crate's borrowed layout/font
readback directly; Text has no dependency on Render.

## Public surface

All public exports are defined by [src/lib.rs](../src/lib.rs); its modules are
private implementation organization.

| Area | Main types and entry points | Source |
| --- | --- | --- |
| Authored text | `Text`, `Source`, `Span`, `InlineBox`, `SourceIdentity`, `SourceRevision` | [text.rs](../src/text.rs), [source.rs](../src/source.rs) |
| Composition | `source`, `compose`, `Composer`, `Mark` | [composer.rs](../src/composer.rs) |
| Style and fonts | `Style`, `Font`, `Brush`, decorations, flow settings | [style.rs](../src/style.rs) |
| Paragraph layout | `Options`, `Alignment`, `Indent`, `ValidatedOptions` | [options.rs](../src/options.rs) |
| Layout construction | `System`, `Builder`, `SystemOptions` | [system.rs](../src/system.rs) |
| Font catalogue | `FontCatalogueMode`, issued `FontCatalogueIdentity`, `System::register_fonts` and `refresh_fonts` | [catalogue.rs](../src/catalogue.rs) |
| Borrowed readback | `RunView`, `FontFace`, `EffectiveFontInstance`, `GlyphPlacement` | [views.rs](../src/views.rs) |
| Output and interaction | `Layout`, glyph runs, line/box facts, cursors, selections, movement | [layout.rs](../src/layout.rs) |
| Ranges and edits | `Range`, `SourcePosition`, `SourceRange`, `Edit`, `TextEdit` | [range.rs](../src/range.rs), [edit.rs](../src/edit.rs) |
| Cache identity | `Key`, `SourceKey`, `StyleKey`, `OptionsKey`, `FontGeneration`, `Stats` | [cache.rs](../src/cache.rs) |
| Errors | `Error`, `ErrorCode`, `ErrorDetail`, `Result` | [error.rs](../src/error.rs) |

## Borrowed paint readback

`Layout::run_views` and `decoration_views` traverse actual positioned output
without allocating owned run/glyph vectors. `FontFace` identifies the actual
font resource and collection face. `EffectiveFontInstance` exposes the canonical
shaping size and normalized coordinates in that face's axis order, including
validated all-zero default axes omitted by the provider. Malformed instance
metadata and unresolved faux-bold/skew return typed failures. These views retain
no independent layout or preparation identity; their lifetime comes from the
original immutable Layout.

## Font catalogue publications

`SystemOptions::default()` selects `FontCatalogueMode::System`; `RegisteredOnly`
excludes system discovery/fallback. `register_fonts(Vec<u8>)` consumes a whole
file and admits all its faces or leaves the current state unchanged. Refresh
constructs a new collection and source cache and replays accepted registered
bytes. The opaque catalogue identity retains its actual System issuer and
checked generation; equal generations from different Systems are distinct.
`Key::from_validated` requires this authentic identity, and `Key` is Clone rather
than Copy. Neither a numeric generation nor a font-content hash grants catalogue
authority. Historical layouts keep their old identities and selected resources.

Generation exhaustion is `FontGenerationExhausted`; malformed containers/faces
and valid faces the catalogue cannot admit are distinct `FontLoadFailed` details.
No operation publishes partial registration or clears a cache on refusal.
Registration does not define unregister, family replacement, or a new missing-
glyph policy. These are catalogue capabilities, not prepared text or selected-
line finalization; the scalar Parley engine remains in use.

## Defaults and validation

`Options::default()` has no width constraint, scale `1.0`, start alignment,
quantization enabled, `TextOverflow::Clip`, `TextAlignLast::Auto`, and zero indent.
The indent defaults to first-line scope. A nonzero indent with `each_line = true`,
`first_line = false`, and `hanging = false` is unsupported by the backend mapping.

`Style::default()` requests a 16-unit font with normal weight, width, and slant;
the empty family list maps to generic sans-serif. It uses metric-relative line
height `1.0`, zero letter/word spacing, opaque black, no decorations, automatic
direction, preserved whitespace, word wrapping, normal word-break and
overflow-wrap, and no text transform.

`Text` owns immutable exact UTF-16 units, including isolated surrogates.
`Text::from_code_units` takes owned units; string conversions encode them
losslessly. `code_units()` lends the original units. Ranges, cursor indices,
glyph/cluster ranges, and inline-box anchors use UTF-16 code-unit offsets.
`Range::new` does not validate ordering. `SourcePosition::try_new` and
`SourceRange::try_new` validate bounds/order against specific `Text`; pair
interiors are valid source endpoints. Layout construction validates source spans, style values, options, inline
box dimensions, and unique inline-box IDs before cache lookup. Font size and
scale must be finite and positive; layout width and box dimensions must be finite
and nonnegative; brush channels must be finite and within `0..=1`.

`TextEdit::apply_to` validates the target range and advances the source revision
once before returning the edited source. At `u64::MAX` it returns
`SourceRevisionExhausted`, including that revision in `ErrorDetail`, before
changing text, spans, or inline-box anchors. Retained sources and layouts remain
unchanged. `SourceRevision::checked_next` returns `None` at exhaustion.

Before cache/font work, layout refuses isolated surrogates with
`IsolatedSurrogateLayout` and span/box boundaries inside a valid pair with
`UnsupportedShapingBoundary`. Source storage and edits still support them.
Scalar source uses one private lossless UTF-8 projection consumed by Parley;
all returned offsets map back to source units. No original-surrogate Unicode
analysis or shaping completion guarantee is added by this interim engine.

Cursor and selection requests retain their exact endpoints. Geometry projects
interior carets to the preceding/following legal logical stop by affinity before
bidi placement. Noncollapsed partial-unit selections cover complete typographic
units; collapsed selections remain collapsed. Offset-taking interaction methods
return `Result`, and invalid endpoints refuse rather than clamp.

The [owned AccessKit projection](../src/accessibility.rs) retains the same layout
identity and source maps. Updates are fallible and publish complete state only.
Host values and character lengths use UTF-8 bytes; an indivisible unit longer
than 255 bytes is unsupported. Mixed-style units preserve all source/geometry
and publish only uniform node metadata. Optional one-dimensional geometry is
omitted when it cannot truthfully describe the fragments. Retained child focus
is valid on repeat updates; parent/child collisions and ID exhaustion refuse.
Mapping a state through another layout returns `AccessibilityLayoutMismatch`.

## Style support

[TextStyleFeature::ALL and support()](../src/style_support.rs) enumerate the
current style-facing contract. This is a capability report, not a CSS parser or
style resolver.

| Status | Features |
| --- | --- |
| Supported | Family lists; named/numeric weights; basic/expanded stretch; normal/italic/oblique slant; feature and variation settings; font size; line height; letter/word spacing; concrete text color; locale. |
| Supported | Preserved whitespace; word-break, wrapping, and overflow-wrap controls; paragraph alignment; supported indent shapes; inline-box alignment facts; underline/strikethrough; decoration offset, thickness, and concrete color; concrete selection color. |
| Unsupported: backend direction control | Explicit text direction. Automatic direction still resolves to left-to-right or right-to-left in `Layout::direction()`. |
| Unsupported: source-range policy | Whitespace collapse. |
| Unsupported: font or text-flow policy | Non-normal font variants, text overflow behavior beyond the accepted default, last-line alignment behavior beyond the accepted default, and text transformation. |
| Unsupported: decoration policy | Overline and decoration styles beyond the exposed solid decoration contract. |
| Unsupported: color resolution | Symbolic text and decoration colors. |
| Unsupported: broader vertical alignment policy | General text vertical alignment. |

`FontVariant::Normal`, `TextOverflow::Clip`, `TextAlignLast::Auto`, and
`TextTransform::None` are the exposed default values. Their acceptance does not
mean the broader feature is implemented; `Clip` does not add a clipping stage to
the layout pipeline. Explicit direction and collapsed whitespace produce typed
`UnsupportedFeature` errors during validation.

Inline-box `VerticalAlign` values are validated, cached, and preserved in output
facts. The current [Parley projection](../src/system.rs) passes box kind, source
index, width, and height but does not apply these alignment requests.
`BaselineShiftFact::Requested` records an authored shift;
`BackendBottomOnBaseline` describes the backend fact used for other values.
Neither means that a requested middle, super, or shifted placement was performed.

## Focused verification

[src/tests.rs](../src/tests.rs) owns the crate's test cases, including feature-gated
render and accessibility cases. [AGENTS.md](../AGENTS.md) owns the repository
command inventory. [Getting started](getting-started.md) gives the focused default
layout proof; enabling an optional feature requires its declared dependencies to
be available locally as well.
