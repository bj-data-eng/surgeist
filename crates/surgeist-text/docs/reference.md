# Reference

## Package and features

[Cargo.toml](../Cargo.toml) defines package `surgeist-text` version `0.1.0`, library
name `surgeist_text`, and Rust edition 2024. No `rust-version` minimum is declared.
The [owned engine](../src/engine/mod.rs) uses safe direct providers for Unicode
analysis, hyphenation resources, font resolution, shaping and font tables.
[Cargo.toml](../Cargo.toml) defines their exact versions;
[source provenance](owned-engine-provenance.md)
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
| Pre-font source processing | `process_source_whitespace`, `process_contextual_source`, `ProcessedSource`, issued `ProcessedParagraph` views and partial plaintext | [processing.rs](../src/processing.rs), [contextual capture](../src/processing/contextual.rs) |
| Style and fonts | `Style`, `Font`, `Brush`, decorations, flow settings | [style.rs](../src/style.rs) |
| Native inline endpoints | `InlineBoundaryAllocation`, `InlineBoundaryContextAllocation`, `ControlAllocation::boundary`, `LineSelectionRequest::boundary_context`, `PreparedText::boundary_context_demand`, `FinalizedControlView::inline_bounds` | [protocol types](../src/protocol/mod.rs), [selection](../src/protocol/selection.rs), [readback](../src/protocol/readback.rs) |
| Supplied mixed lines | `BidiLevel`, `InlineWhitespaceEdge`, `SuppliedLine`, edge decisions and borrowed visual order | [supplied_line.rs](../src/supplied_line.rs) |
| Paragraph layout | `Options`, `Alignment`, `Indent`, `ValidatedOptions` | [options.rs](../src/options.rs) |
| Original analysis and layout | `AnalyzedSource`, `AnalysisIdentity`, `System`, `Builder`, `SystemOptions` | [analyzed.rs](../src/analyzed.rs), [system.rs](../src/system.rs) |
| Font catalogue | `FontCatalogueMode`, issued `FontCatalogueIdentity`, `System::register_fonts` and `refresh_fonts` | [catalogue.rs](../src/catalogue.rs) |
| Font resources and metrics | `FontData`, `FontFace`, `EffectiveFontInstance`, `FontMetrics`, `StrikeoutMetrics` | [font.rs](../src/font.rs), [font/metrics.rs](../src/font/metrics.rs) |
| Borrowed readback | `RunView`, positioned `GlyphPlacement`, actual decorations | [views.rs](../src/views.rs) |
| Prepared hyphenation | `PreparedWord`, `PreparedHyphenationCut`, borrowed views, `PreparedWordPortion`, `HyphenationResource`, `HyphenationProvenance` | [hyphenation.rs](../src/protocol/hyphenation.rs), [resource support](hyphenation-resources.md) |
| Finalized logical prefixes | `TextSeam`, `FinalizedPrefixView`, `FinalizedSharedUnitView`, `FinalizedLine::prefix_at` | [prefix.rs](../src/protocol/prefix.rs), [composition guide](composed-lines.md) |
| Placed original-source fragments | `SourceRangeFragments`, `SourceRangeUnit`, `SourceRangePiece`, `SourceRangeFragmentError`, `Layout::source_range_fragments` | [range_fragments.rs](../src/range_fragments.rs), [composition guide](composed-lines.md) |
| Output and interaction | `Layout`, glyph runs, line/box facts, `PlacedPosition`, `PlacedSelection`, `PlacedSelectionPiece`, source cursors/selections and movement | [layout.rs](../src/layout.rs) |
| Ranges and edits | `Range`, `SourcePosition`, `SourceRange`, `Edit`, `TextEdit` | [range.rs](../src/range.rs), [edit.rs](../src/edit.rs) |
| Cache identity | `Key`, `SourceKey`, `StyleKey`, `OptionsKey`, `FontGeneration`, `Stats` | [cache.rs](../src/cache.rs) |
| Errors | `Error`, `ErrorCode`, `ErrorDetail`, `Result` | [error.rs](../src/error.rs) |

## Prepared hyphenation

`PreparedText::word_views` and `hyphenation_cut_views` expose preparation-owned
observations of complete lexical words, resource applicability, cut provenance,
counts and exact source pieces. The current automatic resource requires en-US
on every genuine word contribution, compared without ASCII case distinctions;
generic English and other locales have no automatic resource. See
[resource support and attribution](hyphenation-resources.md).

The [BreakAll owner contract](../../../docs/text-layout-contract.md#breakall-and-independent-ordinary-cuts)
suppresses Source/Control/Automatic hyphenation when that cut's actual owner has
BreakAll, including empty alternatives, while retaining truthful observations.
Independent ordinary after-source-SHY permission uses the following ordinary
owner, actual Unicode/typographic boundary and native NCA Wrap permission;
source-free Controls or equal source offsets supply no BA permission. An Allowed
cut can retain both its genuine nonhyphenating `None` and an admitted manual
alternative. See the [source determination](../../../references/text-break-all-hyphenation--determination-20261024.md).

`Style::hyphenate_limit_chars` accepts a computed `HyphenateLimitChars` triple
of `HyphenationMinimum` components. Native `Auto` components independently use
5/2/2; explicit counts are nonnegative minima, with no sum or word-length clamp.
The supplying style adapter owns specified-value expansion and checked integer
conversion. The actual raw standalone operation stays unconstrained. See the
[count contract](../../../docs/text-layout-contract.md#automatic-resources-word-custody-and-initial-limits).

Word portions retain only cuts with an actual matching admitted Source/Control
alternative. A retained cut/opportunity alone, including Allowed/None-only, does
not partition a word or advance a portion start; Allowed plus a manual choice
does. `PreparedTermination::Measurement` finalizes an issued portion through the
same selected-line pipeline and cannot be placed. Consumers use
`ConditionalDisposition` to account for earlier ordinary variants when requesting
conditional cuts; actual fitting remains consumer-owned. The
[composition guide](composed-lines.md) defines selection and placement use.

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
`DocumentSegmentBreaks`, returning immutable `ProcessedSource`. All six
`WhiteSpace` values execute before font selection:

| Mode | Actual document delimiter | Other SPACE, TAB and ordinary CR |
| --- | --- | --- |
| `Collapse` | Contextually becomes SPACE or disappears; following collapsible duplicate breaks disappear. | TAB/CR become SPACE; adjacent collapsible spaces collapse. |
| `Discard` | Deleted. | Deleted. |
| `Preserve` | One LF per delimiter. | SPACE/TAB retained; CR becomes SPACE. |
| `PreserveSpaces` | One SPACE per delimiter. | SPACE retained; TAB/CR become SPACE without collapse. |
| `PreserveBreaks` | One LF per surviving delimiter; consecutive LFs remain distinct. | TAB/CR become collapsible SPACE; adjacent collapsible spaces and runs around delimiters collapse. |
| `BreakSpaces` | One LF per delimiter. | SPACE/TAB retained; CR becomes preserved SPACE. |

A supplied document delimiter is one original occurrence; an ordinary CR outside
that inventory never invents a forced break. LF fallback and `Defined(Vec::new())`
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

SPACE collapsing applies to `Collapse` and `PreserveBreaks`; document-break
collapsing and contextual break transformation apply only to `Collapse`.
Discard deletion runs first. Eligible collapsible SPACE/TAB/CR runs around
surviving original document breaks are removed, including around preserved
breaks. Following duplicate Collapse breaks disappear; remaining Collapse breaks
become SPACE or disappear under the context rule below. PreserveBreaks retains
each surviving delimiter as LF, including consecutive delimiters; these yield
distinct Forced opportunities even under `Wrap::None`.

TAB/ordinary CR become SPACE, and following duplicate collapsible spaces are
deleted across eligible Collapse/PreserveBreaks spans. Each disappearing original
space retains its own source participant and actual containing style's Wrap
permission. The first surviving representative retains its original range/style;
bidi controls retain exact units and order. Preserve, PreserveSpaces and
BreakSpaces output spaces are not eligible duplicates. Leading and trailing
representatives remain: this phase does not perform selected-line trimming.

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
full document/line pipeline. All six whitespace modes execute in this
source-processing phase; actual cross-delimiter mode errors remain precise.

### Contextual source processing

`process_contextual_source(ContextualSourceInput)` consumes one complete captured
envelope. Its `SourceProcessingEvent` stream orders actual scopes, paragraphs,
origins, connections, controls and atomic occurrences; referenced tables provide
their facts. `Source::boxes` remains the atomic inventory. Inline, block,
atomic-block and out-of-flow-block roles are explicit. Lexical scopes can remain
open while a block inside an inline ends and resumes a paragraph.

Full effective paragraph styles, delimiter policy, direction, strut and
`WhiteSpaceTrim` facts are captured before trimming and the shared Phase I pass.
`IncompleteExterior` refuses when required surrounding context is absent. Genuine
exterior trimming can remove original PreserveBreaks delimiters before Phase I;
surviving LF then retains its separate forced-break role. These are normalized
producer facts, without authentication of a live DOM or Style tree.

Retain the result in `Arc<ProcessedSource>` and obtain views from `paragraphs()`.
`AnalyzedSource::from_processed_paragraph` consumes an issued view with its fixed
style and coverage; no later range or replacement style can rebind it.
`admit_paragraph` checks the actual processing owner, not equal input values.

Paragraph coordinates remain original UTF-16 positions. `source_range()` is the
minimal envelope of original owned pieces and genuine empty/control/atomic
occurrences, including text subsequently deleted by trimming or collapse.
`source_pieces()` and issued occurrences retain membership; a middle child hole
does not become parent text merely because it lies inside that envelope.
Preparation, placement, interaction and accessibility preserve those boundaries.
An out-of-flow child remains transparent to parent shaping without transferring
its internal source or styles to the parent.

Paragraph `plaintext_items` reads the same processed correspondence and original
owned contributions. It does not add presentation characters, allocated spacing or
hyphenation glyphs. Actual block/forced-break starts govern leading collapsible
space removal; soft wrapping does not create a source break. In-flow atomic
occurrences end leading-space eligibility, while out-of-flow occurrences remain
transparent. This remains a partial plaintext projection, not serialization of
a document or generated alternative content.

### Native mode opportunities and line edges

Captured/analyzed/prepared paragraph paths execute all six modes. PreserveBreaks
uses Collapse-equivalent Phase II removal of leading/trailing U+0020 and trailing
U+1680, retains leading U+1680, and hangs remaining eligible separator sequences
unconditionally, including under nowrap. NBSP remains ordinary content.

BreakSpaces offers after each eligible preserved SPACE/TAB or other separator at
its actual character-end cut, under genuine neighboring/NCA Wrap permission and
typographic-unit/glue constraints. It creates neither a before-first-space cut
nor an offer relocated past a combining continuation. U+2007/U+202F retain their
own GL protection; SPACE/TAB before GL use the adopted Unicode 17/revision 55
boundary, rather than a blanket next-GL veto. WJ/ZWJ protection and genuine
higher-priority ZW opportunities remain. See the [glue source reconciliation](../../../references/css-text-break-spaces-glue--source-reconciliation-20261024.md).

BreakSpaces keeps preserved spaces/tabs/separators in ordinary flow: no line-edge
removal, hanging or intrinsic omission at Soft, Forced or BlockEnd, including
nowrap. An ordinary BreakSpaces row stops preceding hanging groups even at zero
advance. Text's real opportunities and measurements feed Layout selection;
[the whitespace contract](../../../docs/text-layout-contract.md#native-whitespace-edges-and-hanging)
defines fit, alignment and intrinsic contribution custody. Native inline endpoints
supply their own used geometry through the selection contract below.

## Native inline endpoint geometry

Layout owns each Start/End endpoint's actual used border and padding on its own
inline side. [InlineBoundaryEdgeOf](../../surgeist-layout/src/node_input/inline.rs)
checks the nonnegative finite components and their sum in Layout scalar `S`;
`InlineBoundaryInputOf::new(kind, metrics, edge)` requires that explicit edge.
Its `ZERO` is affirmative used zero, not unavailable Style geometry. Lower
`edge.advance()` once to Text's `f64`; separately widening and adding `f32`
components would change the used sum and potentially a tab threshold.

| Text input | Admission and custody |
| --- | --- |
| `InlineBoundaryAllocation::try_new(advance)` / `ZERO` | Checked nonnegative finite total; negative zero normalizes to zero. No authored lengths or duplicated components. |
| `ControlAllocation::boundary` | Every selected InlineBoundaryStart/End requires `Some`, including zero; all other controls require `None`. Struts remain independent vertical metrics. |
| `LineSelectionRequest::boundary_context` | Only demanded off-selection endpoint facts; no selected flow, strut, fragment or placement. Selected endpoints belong only in `controls`. |

For each complete candidate request, `PreparedText::boundary_context_demand`
returns the next genuine off-selection `PreparedControl` needed by consumed
shaping context, or `None`. Append its actual `InlineBoundaryContextAllocation`
and repeat. The list must be the canonical first-demand prefix; a proved positive
barrier stops the outward walk, while zero can reveal a farther endpoint.
The query validates the request without shaping, fitting or querying fonts.
`select` also accepts a directly supplied complete list in the same order.

Missing selected rows retain `MissingControl`; a selected endpoint without a
boundary yields `MissingControlBoundaryGeometry`, and a boundary on another
control yields `InvalidControlGeometry`. Missing demanded context yields
`MissingBoundaryContext`; duplicates/selected overlap yield
`DuplicateBoundaryContext`, and wrong-role, out-of-order or surplus rows yield
`UnexpectedBoundaryContext`. Foreign preparation and numeric failures remain
typed. Missing geometry cannot be replaced with zero.

Positive inter-unit edges bound selection-local original and presentation joining
context. An edge inside one indivisible typographic unit keeps its own numeric
contribution without splitting the unit's shared provider positions. Selected
edges count once in actual flow, tabs, prefixes and all purpose measurements;
zero edges remain transparent to hanging, and positive edges stop it locally.

`FinalizedControlView::inline_bounds()` returns the checked physical left/right
interval; `slot().x` is its left end. The source-free
`FinalizedFragmentOwner::Control` retains that interval at actual line-metric
height. Layout separately publishes the endpoint node's width×0 footprint and
own-side border/padding. Virtual-separator Scope fragments retain only their
actual local separator geometry. These are distinct observations of genuine
occurrences; endpoint width is never added to every incident source Range.

The [endpoint contract](../../../docs/text-layout-contract.md#native-inline-endpoint-geometry)
defines soft-cut attachment, exact event-side positions, reuse and rollback.
This bounded native operation consumes explicit used facts; it does not provide
a live Style producer, authored percentage resolution, vertical Text or complete
inline-decoration envelopes/painting. Raw standalone whitespace, six-mode
processing and the separate #74/Cc/default-ignorable contracts retain their rules.

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
style winners once, without fonts. The post-Phase-I transform phase executes
before semantic Unicode analysis. `Style::word_space_transform` independently
selects `WordSpaceTransform::{None, Space, IdeographicSpace, SpaceAutoPhrase,
IdeographicSpaceAutoPhrase}`. The Space and IdeographicSpace choices, including
their automatic-phrase forms, replace original U+200B with U+0020 and U+3000 before casing;
None is neutral. The separator keeps its exact original/Phase-I range and
plaintext value. Replacement is suppressed next to actual post-Phase-I forced
breaks, ignoring inline boundaries and out-of-flow boxes. A collapsed LF is
not a forced break. Generic optional-break controls are not interpreted as wbr.
Actual disconnected joins bound forced adjacency. Contextual capture retains
the control's genuine connection side. If a flat paragraph leaves that side
ambiguous and it could change a requested original-separator replacement,
`MissingOriginalSeparatorBoundaryContext` identifies the original range, anchor
and connection. None, proven source-hard-break suppression and an intervening
actual character do not demand unused side facts. No generating control is
invented for an original character.
Separator analysis rejects contradictory empty-origin connection order with
`ProtocolError::InvalidConnection`, preserving the supplied occurrence identities.
Native wbr uses the [separate occurrence capture](#native-wbr-separators).
The automatic-phrase forms additionally request
[contextual phrase insertion](#native-automatic-phrase-separators).

`TextTransform::none()` is neutral for casing, width and kana.
`TextTransform::Set` holds a `TextTransformSet`. Its `.with_case` builder selects
`CaseTransform::{Uppercase, Lowercase, Capitalize}` for
full contextual casing with each component's effective language and operation.
Capitalize uses pre-casing native UAX29 words and the first effective
Letter/Number unit, preserving the word tail. Connected styles and occurrences
retain context. The [transformation contract](../../../docs/text-layout-contract.md#text-transformations-and-correspondence)
defines mixed-component policy, language/script resolution and exact source custody.
On the same set, `.with_full_width(true)` applies the 226 raw wide/narrow mappings from
[UnicodeData 18.0.0](../../../references/SOURCE-EDITIONS.md#text-full-width-mapping-data)
after casing. It maps original U+0020 only when actual Phase-I processing preserved
that space: a surviving collapsed space remains U+0020, while PreserveSpaces' converted
TAB or document-break spaces become U+3000. Raw standalone layout preserves its
actual spaces. SPACE generated by eligible separator replacement qualifies in
every mode, including Collapse and Discard, under the
[native source determination](../../../references/SOURCE-EDITIONS.md#separator-generated-space-and-full-width).
Phase I is not replayed over this output, so adjacent generated spaces stay
distinct. Phase II still applies to the actual emitted character and governing
style; width eligibility does not confer immunity from trimming or hanging.
Preserved TAB and line-break controls remain controls. Unrelated
compatibility characters remain unchanged; this is not NFKC or NFKD.
`.with_full_size_kana(true)` then enables all 58 mappings in the pinned
[CSS kana table](../../../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#small-kana).
Full-size-kana alone retains halfwidth forms; width plus kana maps halfwidth small
kana through its full-width small form to the full-size form.
Real transformed code points drive analysis, shaping and wrapping;
segmentation still uses a separately proven private projection for isolated units.
Original/source AX and plaintext retain their own original/Phase-I projections.

`TextTransform::MathAuto` is an exclusive computed alternative. Native paragraph
origins supply `text_node: Some(CompleteTextNodeContribution::new(text, range)?)`:
the actual complete original Text-node content and its node-relative contributed
UTF-16 range. The constructor validates bounds; capture validates length and
exact units against the origin's local source. Supplied facts always validate,
including empty contributions and styles without MathAuto. The capture does not
assign host node identity or infer it from Source IDs, origin slices or equal text.

Eligibility uses the complete original node's single listed scalar, then maps
the actual surviving Phase-I code point through the pinned
[MathML Core table](../tools/math-auto/README.md). A one-character slice of a
longer node stays ineligible. Original `SPACE c SPACE` stays ineligible after
Discard leaves `c`. Known complete non-key or multicharacter content succeeds
unchanged. An actual positive MathAuto-demanded contribution without a node
fact returns `TextTransformError::MissingTextNodeContext` before trim or Phase I;
invalid ranges and mismatched source content have distinct typed errors. Empty
origins and non-text controls create no positive text demand. Explicit computed
MathAuto has no inferred namespace gate; the actual MathML default-style adapter
remains separate. Raw standalone analysis/layout has no such capture and refuses
MathAuto with `RequiresCompleteTextNodeContext`.

Original, processed and presentation coordinates remain distinct: for `A𛄲C`,
original and Phase-I length are 4, with middle processed interval 1..3; transformed
presentation length is 3, with middle interval 1..2. `PreparedBoundary` and unit
processed readbacks remain Phase I. Selected `mapping_views` retain both exact
processed extents and line-local presentation extents. Source Range/carets keep
the original supplementary association and actual transformed glyph geometry.
For `ß → SS`, original and Phase-I lengths remain 1 while presentation length is 2.
An exact `PlacedPosition` retains the interior output boundary; its source cursor
is a lossy projection. `position_selection` returns `PlacedSelection`, with
geometry, occurrence-qualified original pieces, exact UTF-16 `source_text()` and
separate `plaintext()`. Selecting either S extracts its original ß; selecting both
extracts that occurrence once. Removed components retain ordered source coverage
without glyphs. Exact cuts determine collapse even when geometry coincides;
they do not create fractional original editing rights.

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

### Native wbr separators

Capture each genuine occurrence as `ParagraphControlKind::Wbr { style: Box<Style> }`
in direct or contextual paragraph controls, retaining its actual anchor order.
The complete computed Style owns replacement, subsequent transforms and metrics.
`WordSpaceTransform::None` and forced-break suppression leave the unexpanded
control; positive Space/IdeographicSpace produces a text contribution with its
actual control identity. It requires no duplicate control allocation.

Every genuine wbr separates original UTF-16 decoding before Phase I, including
None and suppressed requests. High/low surrogate units on opposite sides remain
isolated original units; provider-only replacement glyphs do not supply their
semantic properties. Later projections preserve that distinction and the exact
original code units. This native decoding rule does not disconnect whitespace,
casing, word/phrase, bidi or shaping context. Ordinary origin splits, inline
boundaries and out-of-flow boxes remain transparent to scalar formation.

Insertion uses the exact post-Phase-I scalar cut, before rebuilding graphemes.
It can therefore separate a base from a following combining mark without moving
to an old grapheme boundary. Deletions retain their actual mixed-event seam;
same-anchor occurrences retain distinct order and identities. Ordinary anchors
retain their documented affinity on the rebuilt units; equal projected positions
do not merge or reorder captured occurrences. A positive request inside a
multi-unit document-delimiter replacement returns
`TextTransformError::UnrepresentableSeparatorSeam`, identifying the control,
anchor and indivisible original interval. None and demonstrably forced-suppressed
requests do not demand that insertion capability. Contextual capture rejects
connections inside declared delimiters; the broader flat interface retains this
precise replacement-interior refusal.

Contextual capture also retains the actual event side of disconnected text joins.
A positive direct request coincident with such a join requires that execution
context: `MissingSeparatorBoundaryContext` identifies the control, source anchor
and connection when the flat capture cannot establish its side. Typography
affinity does not supply that fact. A numerically nearby forced break cannot
prove suppression across an unknown connected side. None and requests that do
not consume the missing fact keep their existing admission.

`PreparedUnitView::controls()` and `SourceRangeUnit::controls()` expose actual
contributors alongside original-source associations. A generated-only unit has
no invented original pieces. A generated separator and authored mark sharing one
actual Range unit retain both contributors and their real fragments: querying
the mark selects that indivisible unit, with original intersections restricted
to the mark. An empty original interval does not select a generated-only unit.
`PresentationMappingKind::GeneratedSeparator` and `separator_control()` distinguish
its empty source/processed anchor from its presentation extent. Exact placed
selection can traverse that geometry while source extraction and plaintext omit
generated characters. Bind those control contributors when lowering Text units
to Layout; origin-only ordering is insufficient for source-less contributions.

### Native automatic phrase separators

`SpaceAutoPhrase` and `IdeographicSpaceAutoPhrase` request detection before
separator replacement, casing, width and kana transformation. Text analyzes the
complete connected supported-language segment with the pinned BudouX 0.9.3
Japanese model. Font, origin and ordinary inline splits do not shorten that
context; actual disconnected, in-flow atomic and hard-break boundaries do.
Out-of-flow child interiors do not become text in the parent domain.

Initial language admission requires validated primary language `ja`, with script
absent or `Jpan`. Regions do not change that admission. Unknown languages,
`und-Jpan`, `en-JP` and other explicit scripts produce no automatic proposals;
Text never infers Japanese from glyphs. The private provider follows decoded
semantic positions, preserving supplementary scalars and isolated UTF-16 units.
The [model guide](../tools/automatic-phrase/README.md) identifies its exact source,
generated tables and qualification limits. This support is not a general
Japanese linguistic-quality or browser-equivalence guarantee.

Use `ContextualSourceInput` to capture the actual balanced scope stream. Domain
roots declare `SourceProcessingTopology::{Unknown, Complete}` independently of
the exterior-trimming envelope. Inline scopes capture their actual complete
`style: Option<Style>`; absence means unknown, not computed None. If a surviving
candidate needs missing topology or owner Style, Text returns
`MissingPhraseTopology` or `MissingPhraseScopeStyle`. Genuine no-output paths do
not demand unused metadata. Supplied Styles and structural facts still validate
when the request is inactive. Flat paragraph/source capture cannot infer a tree.

Detection and expansion ownership differ. Incident text requests detection; the
entire intervening inline-boundary strip determines the outermost owner,
including empty scopes. That scope's Space can expand a child-detected candidate
without itself requesting automatic detection. Its None suppresses expansion
without borrowing the child's Style. Text chooses the earliest actual seam at
which the winning scope is active and preserves genuine mixed-event order.

CSS eligibility excludes typographic-unit interiors, adjacent GL/WJ/ZWJ and
existing separators. The native recognized separator repertoire includes all Zs,
U+200B, the seven explicit CSS examples, U+103D0 and U+12470; it is not an exhaustive
Unicode word-separator property. Conservative punctuation attachment withholds
proposals beside unfamiliar punctuation and removes punctuation-only chunks.
The native policy emits internal model cuts, not exterior-domain separators.
True forced-break adjacency suppresses insertion before demanding owner facts.
A genuine wbr remains a separate occurrence: WbrNone can coexist with an emitted
virtual separator, and two independently expandable occurrences can emit two.
Generated replacement characters do not retrospectively suppress proposals.

`PreparedText::scopes()` issues only contributing, preparation-specific handles.
`PreparedScope::captured_scope()` identifies the canonical scope in the shared
immutable capture. `CapturedScope` and `CapturedControl` equality requires that
same capture and actual occurrence; equal ordinals or source offsets from a
different capture cannot substitute. Independently prepared projections of one
capture share canonical identity while retaining distinct prepared identities.

`PreparedText::captured_paragraph()` exposes authentic scope intersections and
mixed-event positions independently of generated output. Canonical Style and
endpoint facts remain optional when genuinely uncaptured; unused metadata is
not replaced by defaults. `ScopeRole` distinguishes inline and domain ownership.
`CapturedDomainDefinition` retains each domain's actual Block, AtomicBlock or
OutOfFlowBlock association, including the exact nominated atomic occurrence.
`CapturedScope::flow_events()` projects a domain's actual paragraph boundaries,
originals, atomics, controls and plain child-Block boundaries in capture order.
It exposes the original domain flow even when no generated output is selected.
Atomic child contents have their own projection; their capture schedule does not
interleave with parent flow. Unknown inline topology does not erase supplied
domain events.

An inline scope's global controls remain its genuine Start/End pair immediately
inside its enter/exit events, even across block-in-inline paragraph cuts.
`PreparedScope::view()` observes only controls local to that preparation; local
absence does not make the global pair incomplete. No paragraph cut manufactures
a continuation control or transfers Style, source membership or geometry.

`PreparedText::virtual_separators()` issues distinct occurrence handles with
their scope and exact empty source/processed anchors. Unit and Range views expose
`virtual_separators()` alongside origins and controls. Generated mapping readback
uses `separator_virtual()`; finalized geometry has
`FinalizedFragmentOwner::VirtualSeparator`. These contributors never acquire an
authored interval. Source extraction, plaintext, accessibility and automatic
hyphenation retain original text only, while placed selection can traverse the
real generated geometry. Shared grapheme units preserve every real contributor.
Selected-indication rebuilds retain the same scope Style and occurrence identity.

Layout binds each contributing domain to its actual paragraph container. It
derives one inline association registry from canonical endpoints and the real
control-to-node bindings across all supplied paragraphs. Paragraph bindings do
not author another scope-pair list. Remote endpoints and middle intersections
are validated against genuine capture order and actual tree structure; paragraph
vector order and source-offset sorting do not supply that authority. Atomic
content domains must use their exact nominated atomic's node.
Scope proof covers each contributing inline owner's complete authentic extent,
or a contributing domain's complete own-domain flow. Known ancestor endpoints
and nominee parent bindings establish structural dependencies without activating
unrelated content. Disjoint owner extents stay separate; no active extent may
discard an interruption merely because its association was omitted.

A genuine captured paragraph with no actual or supplied source/control/atomic
participation, generated contributors or run nodes can associate an empty domain
with its real Layout box. It requests no line strut and generates no Text result,
line or fragment. Ordinary box layout remains intact. Empty rendered output alone
does not prove this condition: removed-only originals still participate.

The complete association survives in `InlineFragmentOwner::Scope`, alongside the
fragment's own prepared/placed occurrence. A spanning scope emits only each
paragraph's real local separator geometry; boundary nodes never substitute for
that geometry. Each actual endpoint retains its separate
[control footprint](#native-inline-endpoint-geometry).
Truly missing endpoint facts/bindings, foreign capture identities, inconsistent
tree order and ambiguous required tree occurrences refuse precisely. No generated
text is donated to an original or duplicated onto both boundaries.

## Style support

[TextStyleFeature::ALL and support(phase)](../src/style_support.rs) enumerate the
current style-facing contract. This is a capability report, not a CSS parser or
style resolver.

| Status | Features |
| --- | --- |
| Supported | Family lists; named/numeric weights; basic/expanded stretch; normal/italic/oblique slant; feature and variation settings; font size; line height; letter/word spacing; concrete text color; locale. |
| Supported | Preserved whitespace; word-break, wrapping, and overflow-wrap controls; paragraph alignment; supported indent shapes; inline-box alignment facts; underline/strikethrough; decoration offset, thickness, and concrete color; concrete selection color. |
| Unsupported: paragraph direction control | Explicit text direction. Automatic direction still resolves to left-to-right or right-to-left in `Layout::direction()`. |
| Phase-specific | Collapse, Discard, PreserveSpaces, PreserveBreaks and BreakSpaces execute through explicit processed-paragraph capture; raw standalone layout requires Preserve. |
| Unsupported: font or text-flow policy | Non-normal font variants, text overflow behavior beyond the accepted default, last-line alignment behavior beyond the accepted default. |
| Unsupported: decoration policy | Overline and decoration styles beyond the exposed solid decoration contract. |
| Unsupported: color resolution | Symbolic text and decoration colors. |
| Unsupported: broader vertical alignment policy | General text vertical alignment. |

`TextStyleFeature::{Uppercase, Lowercase, Capitalize, FullWidth, FullSizeKana}` report the
executed operations as Supported for both standalone and processed-paragraph
analysis. These entries do not certify the entire CSS text-transform property.
`TextStyleFeature::MathAuto` reports support for ProcessedParagraph only;
Standalone reports `RequiresCompleteTextNodeContext`.
`TextStyleFeature::ExplicitWordSeparators` reports source-backed U+200B replacement
in both phases. `NativeWbr` reports processed-paragraph support; standalone layout
requires processed-source layout. `NativeAutomaticPhrases` reports contextual
Japanese phrase execution in ProcessedParagraph, with the capture and detection
limits above; standalone layout requires processed-source layout.

`TextStyleFeature::{WhiteSpacePreserveBreaks, WhiteSpaceBreakSpaces}` report
Supported for `ProcessedParagraph`. Raw standalone formatting refuses each with
`ErrorDetail::UnsupportedTextStyle`, its matching feature and
`UnsupportedTextStyleReason::RequiresProcessedSourceLayout`; it does not silently
switch to a processed profile.

`FontVariant::Normal`, `TextOverflow::Clip`, `TextAlignLast::Auto`, and
`TextTransform::none()` are the exposed default values. Their acceptance does not
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
fragments, explicit baseline placement, placement-bound positions and nonempty
original-source font-metric observations.
