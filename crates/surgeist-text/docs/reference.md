# Reference

## Package and features

[Cargo.toml](../Cargo.toml) defines package `surgeist-text` version `0.1.0`, library
name `surgeist_text`, and Rust edition 2024. No `rust-version` minimum is declared.
The [owned engine](../src/engine/mod.rs) uses safe direct providers for Unicode
analysis, font resolution, shaping and font tables. [Cargo.toml](../Cargo.toml)
defines their exact versions; [source provenance](owned-engine-provenance.md)
records the adapted mechanisms and cached source identities. Parley is not a
production dependency.

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
| Pre-font source processing | `process_source_whitespace`, `ProcessedSource`, `DocumentSegmentBreaks`, processed positions/ranges and partial plaintext views | [processing.rs](../src/processing.rs) |
| Style and fonts | `Style`, `Font`, `Brush`, decorations, flow settings | [style.rs](../src/style.rs) |
| Supplied mixed lines | `BidiLevel`, `InlineWhitespaceEdge`, `SuppliedLine`, edge decisions and borrowed visual order | [supplied_line.rs](../src/supplied_line.rs) |
| Paragraph layout | `Options`, `Alignment`, `Indent`, `ValidatedOptions` | [options.rs](../src/options.rs) |
| Original analysis and layout | `AnalyzedSource`, `AnalysisIdentity`, `System`, `Builder`, `SystemOptions` | [analyzed.rs](../src/analyzed.rs), [system.rs](../src/system.rs) |
| Font catalogue | `FontCatalogueMode`, issued `FontCatalogueIdentity`, `System::register_fonts` and `refresh_fonts` | [catalogue.rs](../src/catalogue.rs) |
| Font resources and metrics | `FontData`, `FontFace`, `EffectiveFontInstance`, `FontMetrics`, `StrikeoutMetrics` | [font.rs](../src/font.rs), [font/metrics.rs](../src/font/metrics.rs) |
| Borrowed readback | `RunView`, positioned `GlyphPlacement`, actual decorations | [views.rs](../src/views.rs) |
| Output and interaction | `Layout`, glyph runs, line/box facts, cursors, selections, movement | [layout.rs](../src/layout.rs) |
| Ranges and edits | `Range`, `SourcePosition`, `SourceRange`, `Edit`, `TextEdit` | [range.rs](../src/range.rs), [edit.rs](../src/edit.rs) |
| Cache identity | `Key`, `SourceKey`, `StyleKey`, `OptionsKey`, `FontGeneration`, `Stats` | [cache.rs](../src/cache.rs) |
| Errors | `Error`, `ErrorCode`, `ErrorDetail`, `Result` | [error.rs](../src/error.rs) |

## Supplied mixed-line processing

`BidiLevel` represents supplied resolved levels `0..=126`, including implicit level 126;
explicit embedding depth 125 is a different phase. See the [metadata-only UAX9
reference](../../../references/uax9--18.0.0--rev52-metadata.md). `SuppliedLine` borrows
the complete logical text/atomic/boundary sequence and applies supplied edge
policies and L2 ordering. Forced breaks delimit inputs; discarded text remains an
ordering slot. It does not analyze Unicode, apply L1, choose breaks or place boxes.

`reorder_into` initializes exact-length caller scratch. A typed length error leaves
scratch unchanged; success borrows both backing slices and iterates local logical
ordinals paired with their actual participants. Edge decisions retain the same
input loan, including overlapping discard cuts for an entirely discarded line.
These operations allocate no storage. Layout consumes this Text boundary directly
while retaining fitting, scalar width/metric reduction, float feedback and physical
placement. This front supplies no preparation/finalized-line identity and does not
replace the standalone paragraph path.

## Borrowed paint readback

`Layout::run_views` and `decoration_views` traverse actual positioned output
without allocating owned run/glyph vectors. `FontFace` identifies the actual
font resource and collection face. `EffectiveFontInstance` exposes the canonical
shaping size and normalized coordinates in that face's axis order, including
validated all-zero default axes omitted by the provider. Malformed instance
metadata and unresolved faux-bold/skew return typed failures. These views retain
no independent layout or preparation identity; their lifetime comes from the
original immutable Layout.

## Source-local whitespace processing

`process_source_whitespace` consumes an actual `Source`, base `Style` and explicit
`DocumentSegmentBreaks`, returning immutable `ProcessedSource`. It currently
executes effective Collapse, Discard, Preserve and PreserveSpaces. A document delimiter is one original occurrence: Preserve replaces
it by one LF, PreserveSpaces by one space, and Discard deletes it. Outside those
occurrences, Preserve and PreserveSpaces convert CR to space; PreserveSpaces also
converts TAB to space, while Preserve retains TAB and ordinary spaces. Discard
removes SPACE, TAB and CR. LF fallback and `Defined(Vec::new())`
remain distinct. Last applicable original span wins; empty spans remain records.
A delimiter crossing a change of effective mode refuses rather than splitting
the occurrence; same-mode span-origin changes admit. Intrinsic validation is
shared with layout, but unrelated font and style fields are retained without
admission by this phase.

The result retains original UTF-16, identity, spans, boxes and document facts.
Processed positions/ranges use a separate numeric domain, not preparation IDs.
`source_to_processed(at, affinity)` maps copies linearly and deletions to one
position. A replacement maps its original endpoints to output endpoints; a strict
original interior maps to output start Before or end After. Inverse mapping
collects incident original boundary candidates and chooses minimum Before or
maximum After. Replacement edges recover original endpoints, not contraction
interiors: this is not the inverse fiber of either forward projection. For
deletion-only inputs the original complete-preimage extrema law is preserved.

`ProcessedSpan::range(affinity)` projects both endpoints with the same choice, so
empty spans remain collapsed. This numeric projection is neither unique computed
style/font ownership of the normalized unit nor conservative painted coverage.
Span/box views still borrow exact original records; editing/extraction retains
original precision and affinity independently of projected coordinates.

`plaintext_items` visits the same processed slices and actual box occurrences in
original anchor order, with original vector order for ties. InFlow boxes remain
atomic adjacency barriers; OutOfFlow boxes are transparent even where enumeration
cuts a text slice. A supplied delimiter crossing an InFlow anchor refuses.
`ProcessedInlineBox::position(affinity)` and `plaintext_items(affinity)` expose the
same projection choice: an OutOfFlow anchor strictly inside a contraction appears
before its normalized unit for Before, after it for After. Its exact original
anchor remains unchanged; no new document order or adjacency barrier is inferred.
Text slices partition the sole processed backing exactly once under either query.
No
separator, alternative content, U+FFFC or opportunity is invented. This is partial
source-local plaintext, not a document serializer or a line-analysis result.
The explicit composed paragraph constructor consumes this same correspondence.
Standalone Preserve retains its identity correspondence and does not silently
normalize delimiters. General trim, opportunity rules and writing-mode profiles
remain bounded by their actual admitted capabilities; see [composed lines](composed-lines.md).

### Lexical Collapse

Collapse runs before wrapping or bidi placement. Discard deletion is applied
first. Eligible Collapse SPACE/TAB/CR runs around surviving original document
breaks are removed, including around preserved breaks. Following duplicate
Collapse breaks are removed. Remaining Collapse breaks become SPACE or disappear
under the context rule below, TAB/CR become SPACE, and following duplicate
Collapse spaces are deleted. The first surviving representative retains its
original range/style; retained bidi controls keep their exact units and order.
Preserve/PreserveSpaces output spaces are not eligible duplicates. Leading and
trailing representatives remain: this phase does not perform line trimming.

The selected Surgeist UA rule removes a break next to U+200B; otherwise it removes
one between two scalars whose actual Unicode East_Asian_Width is F/W/H and whose
Script is not Hangul. Other contexts produce SPACE. Actual Bidi_Control scalars
and OutOfFlow occurrences are transparent for adjacency; InFlow occurrences are
barriers. NBSP, WJ/ZWJ, marks and other format controls are real context. Isolated
surrogates are opaque and cannot become property lookup scalars by deleting or
ignoring intervening original units. Only actual contiguous original pairs are
decoded, and an InFlow occurrence or document delimiter can separate them.
The compiled ICU properties are version 2.2.0; no font or repaired text is used.
This is a selected UA policy under the current standard's contextual discretion,
with [historical precedent](../../../references/css-text-4-character-alignment-grammar--WD-css3-text-20121113--cc1cba0eebc9.md#line-break-transform),
not a claim of whole-language wrapping or browser equivalence.

Private map records distinguish Discard, around-break, duplicate-break,
contextual-break and duplicate-space deletion. Equal causes may coalesce;
different causes remain distinct even at the same processed anchor. All existing
forward/inverse affinity and borrowed original span/box laws still apply. These
are actual lexical outcomes, not computed soft opportunities or preparation IDs.
Source-local input does not provide ordinary host box edges, NCA wrap permissions,
trim/block facts or selected lines. Those producers remain necessary for the
full document/line pipeline. All four current whitespace modes execute in this
source-processing phase; actual cross-delimiter mode errors remain precise.

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
glyph policy. These are catalogue capabilities. The private owned engine retains the selected
resources through actual preparation, selected-line finalization and placement;
the public mixed-line protocol lends those actual phases without copying glyph packets.

## Glyph-free associated-font metrics

`System::registered_font_faces()` lends accepted registrations in request order,
then original collection-face order. It does not enumerate discovered fonts,
resolve a style or shape a sample. `FontFace::to_owned()` retains the actual Blob,
collection index and catalogue publication without copying font bytes. The same
canonical resource owner is used by layout readback. `FontData` equality retains
its existing resource identity plus collection index semantics; catalogue
qualification is separate. Refresh can therefore produce resource-equal handles
with different authentic publications.

`FontData::unsynthesized_instance(size, normalized_coords)` validates the actual
face and complete canonical coordinates. Its borrowed `metrics(expected_catalogue)`
query first requires the resource's exact publication. A historical resource
continues to work with its original publication after refresh or System disposal;
a later or foreign publication refuses. Queries neither shape glyphs nor change
layout/cache statistics. The result borrows the actual resource and coordinate
storage, rather than a temporary instance view.

This domain supports unsynthesized static and variable instances. A variable
face without MVAR keeps its base global metrics even when gvar changes outlines.
MVAR version1.0 requires real fvar axes, complete canonical coordinates, strictly
ordered eight-byte value records, a complete format1 Item Variation Store and
canonical region-coordinate domains. Both normal and LONG_WORDS delta encodings
are supported; this does not execute COLR. A larger record stride is precisely
unsupported. Present malformed tables, indexes and complete referenced rows
refuse even at default or inactive coordinates. Declared null data slots and
exact FFFF/FFFF no-variation indexes are valid zero adjustments; an out-of-array
outer index or inaccessible nonnull slot is an error.

Region interpolation uses the full actual coordinate vector. Peak-zero,
misordered and crossing-zero axes have the specified neutral factor; they are
not rejected as malformed tuples. Integer deltas accumulate with checked i64
arithmetic and one final rounding (half ties toward positive infinity), then
widen exactly into f64. No per-region rounding, permissive failed-read zero or
wrapping 16.16 metric conversion is used. Shared data offsets and rows are
qualified once; immutable aliases are permitted. Scratch capacity/refusal belongs
to the font metric operation and cannot publish partial metrics or cache changes.

Line deltas follow the actual stored-field origin selected from the original
validated tables: `hasc/hdsc/hlgp` adjust chosen OS/2 typo fields; hhea fields
receive no inferred line tags. Windows fallback uses `hcla/hcld`, with descent
adjusted in its positive stored domain before negation. Its retained hhea line
gap (or initial zero) receives no line adjustment. `stro/strs` adjust only an
actual OS/2 strike; they cannot create an absent decoration. Unknown/unreturned
tags still require structurally valid indexes and storage, but do not execute
another field. The [selection provenance](font-metrics-provenance.md) identifies
the retained provider branch and the separate owned qualification.

Metrics use f64 geometry at the actual f32 instance size: nonnegative ascent,
signed nonpositive descent, and finite signed leading. Required signs are checked
after applying actual-instance deltas. A usable resolved strike top and positive
thickness become `center = top - thickness / 2`. Otherwise fallback uses the
resolved `center = ascent / 2`, `thickness = size / 18`. At horizontal baseline
`b`, strike top is `b - center - thickness / 2`. This query does not change the
paragraph decoration policy. The owned engine reads this same instance metric
owner during shaping; the query itself adds no font matching, glyph coverage,
outline metrics or complete text-engine guarantee.

## Defaults and validation

`Options::default()` has no width constraint, scale `1.0`, start alignment,
quantization enabled, `TextOverflow::Clip`, `TextAlignLast::Auto`, and zero indent.
The indent defaults to first-line scope. A nonzero indent with `each_line = true`,
`first_line = false`, and `hanging = false` is unsupported by the standalone indent policy.

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

`AnalyzedSource::new(Source, Style)` captures the original source and effective
style winners once, without fonts. Raw original Unicode properties drive bidi;
segmentation uses a separately proven private projection for isolated units.
The glyph projection substitutes U+FFFD only for an original isolated unit;
source associations remain exact and deletion cannot manufacture a new scalar.
Span/box boundaries inside a valid pair refuse with `UnsupportedShapingBoundary`.

The standalone Preserve profile retains its established per-unit mandatory
breaks: CR, LF, U+2028 and U+2029 each end a line, including a terminal empty
line. CRLF therefore ends two lines here. This is the characterized standalone
profile, not inferred HTML delimiter policy or document normalization. The
separate source-processing `DocumentSegmentBreaks` contract remains unchanged.
An empty placed paragraph retains a real admitted-font strut for caret and line
metrics, with zero intrinsic width and no fabricated source character or glyph.

`System::layout_analyzed` retains that exact analysis identity. The convenience
`System::layout` uses the same analysis, preparation, chooser, finalizer and
placement pipeline. Renderable source without a usable selected face refuses;
an empty registered-only catalogue remains valid for source analysis. Every
actual shaping result must report completion before glyph extraction or cache
publication. Refusal preserves prior layouts, catalogue identity and cache stats.

Logical geometry, spacing, line-height and glyph pens use `f64`. Font size and
actual provider glyph offsets/advances remain canonical `f32`. A glyph's composed
position adds its run origin, logical pen and canonical local offset. Logical
spacing and justification do not rewrite the provider advance. Concrete faux
synthesis is resolved once on the actual font instance, without changing line
metrics or advances; Render owns outline execution.

Cursor and selection requests retain their exact endpoints. Geometry projects
interior carets to the preceding/following legal logical stop by affinity before
bidi placement. Noncollapsed partial-unit selections cover complete typographic
units; collapsed selections remain collapsed. Offset-taking interaction methods
return `Result`, and invalid endpoints refuse rather than clamp.

The [owned AccessKit projection](../src/accessibility.rs) retains the exact analysis
identity and source maps. Source-only updates need no point and omit font, glyph,
line and all geometry fields. Placed updates enrich the same semantic projection
from actual finalized rows, and replacement clears obsolete optional fields. Updates are fallible and publish complete state only.
Host values and character lengths use UTF-8 bytes; an indivisible unit longer
than 255 bytes is unsupported. Mixed-style units preserve all source/geometry
and publish only uniform node metadata. Optional one-dimensional geometry is
omitted when it cannot truthfully describe the fragments. Retained child focus
is valid on repeat updates; parent/child collisions and ID exhaustion refuse.
Queries require the exact analyzed or placed binding that produced the state; a
foreign identity or wrong phase returns `AccessibilityBindingMismatch`. Original
ill-formed UTF-16 cannot form a host string and refuses before state publication.

## Style support

[TextStyleFeature::ALL and support(phase)](../src/style_support.rs) enumerate the
current style-facing contract. This is a capability report, not a CSS parser or
style resolver.

| Status | Features |
| --- | --- |
| Supported | Family lists; named/numeric weights; basic/expanded stretch; normal/italic/oblique slant; feature and variation settings; font size; line height; letter/word spacing; concrete text color; locale. |
| Supported | Preserved whitespace; word-break, wrapping, and overflow-wrap controls; paragraph alignment; supported indent shapes; inline-box alignment facts; underline/strikethrough; decoration offset, thickness, and concrete color; concrete selection color. |
| Unsupported: paragraph direction control | Explicit text direction. Automatic direction still resolves to left-to-right or right-to-left in `Layout::direction()`. |
| Phase-specific | Collapse, Discard and PreserveSpaces execute through explicit processed-paragraph capture; raw standalone layout requires Preserve. |
| Unsupported: font or text-flow policy | Non-normal font variants, text overflow behavior beyond the accepted default, last-line alignment behavior beyond the accepted default, and text transformation. |
| Unsupported: decoration policy | Overline and decoration styles beyond the exposed solid decoration contract. |
| Unsupported: color resolution | Symbolic text and decoration colors. |
| Unsupported: broader vertical alignment policy | General text vertical alignment. |

`FontVariant::Normal`, `TextOverflow::Clip`, `TextAlignLast::Auto`, and
`TextTransform::None` are the exposed default values. Their acceptance does not
mean the broader feature is implemented; `Clip` does not add a clipping stage to
the layout pipeline. Explicit `Style.direction` and non-Preserve raw standalone
whitespace still refuse. Composed base direction is a separate captured paragraph
fact, and `TextStylePhase::ProcessedParagraph` reports its executed whitespace modes.

Inline-box `VerticalAlign` values are validated, cached, and preserved in output
facts. The [standalone engine](../src/engine/layout/mod.rs) retains box kind, source
anchor, width and height but does not execute these alignment requests.
`BaselineShiftFact::Requested` records an authored shift;
`BottomOnBaseline` describes the actual placement fact used for other values.
Neither means that a requested middle, super, or shifted placement was performed.

## Focused verification

[src/tests.rs](../src/tests.rs) owns the crate's test cases, including feature-gated
render and accessibility cases. [AGENTS.md](../AGENTS.md) owns the repository
command inventory. [Getting started](getting-started.md) gives the focused default
layout proof; enabling an optional feature requires its declared dependencies to
be available locally as well.

## Composed lines

[The composed-line guide](composed-lines.md) describes actual preparation-issued
handles, selected spacing, feedback, authentic original/control/replacement
fragments, explicit baseline placement and placement-bound positions.
