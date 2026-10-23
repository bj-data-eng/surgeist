# Compose selected lines

The composed front shares the existing source, analysis, font and finalized-row
owners. It does not construct another independently wrapped paragraph. The
standalone `System::layout` convenience retains its raw Preserve behavior and
uses the same selected-line and placement machinery.

1. For captured box/paragraph context, call `process_contextual_source` with the
   actual ordered envelope and retain its result in `Arc<ProcessedSource>`.
   Obtain an issued paragraph from `paragraphs()` and analyze it through
   `AnalyzedSource::from_processed_paragraph`. Its style and original coverage
   were fixed before trim and Phase I. Child domains remain separate, including
   where a parent glyph shapes across an out-of-flow child hole. See
   [contextual source processing](reference.md#contextual-source-processing).
   For one complete native paragraph, construct `ParagraphInput` from the actual
   aggregate `Source`, base `Style`,
   document delimiter policy, explicit base direction and original occurrences.
   `paragraph_strut: ParagraphStrut::DefaultFont` requests an actual base-style
   font strut on every native line, even when body spans select other styles.
   `AllocatedContributions` omits that base strut; actual body and supplied
   allocations determine metrics. A selected empty line still needs a genuine
   contribution, including a terminal empty line. Explicit zero differs from absence.
   Supply connections, ordered anchor occurrences with original endpoint affinity,
   controls, wrapping facts and any actual computed discretionary replacements.
   `AnalyzedSource::from_paragraph` validates this complete capture and retains the
   executed `ProcessedSource`; original code-unit ownership remains available
   through the processed correspondence.
2. Call `System::prepare(&analysis, PreparationOptions)` with the actual scale.
   Preparation resolves resources and preliminary measurements without choosing
   breaks. Public preparation of `AnalyzedSource::new` also requests the genuine
   native base-font strut. For an allocated-only fontless box/control paragraph,
   use explicit `ParagraphInput` with `ParagraphStrut::AllocatedContributions`.
   `System::layout` retains its actual standalone metric policy;
   there is no additional public preparation switch.
   Borrowed views expose actual rows; their `handle()` methods retain the
   issued capability. Equal text prepared independently has different authority.
   Native atomic opportunities use the captured ancestor wrapping permission and
   actual mixed neighbors, including atomic items after the authored text ends.
   NBSP keeps its atomic exception; other GL, WJ and ZWJ characters suppress the
   adjacent atomic opportunity. A collapsing space that disappears at the break
   retains its separate character-owned opportunity under its containing style.
   Transparent occurrences add no permission: existing internal cuts borrow the
   same relation, while semantic beginning and end do not acquire soft breaks.
3. Supply a `LineSelectionRequest` with issued start/end boundaries, every selected
   mixed participant in order, an actual terminating opportunity, complete context
   and occurrence-qualified atomic/control allocations. Signed atomic flow and
   canonical border/content geometry remain distinct. `LineBoxAlignment` captures
   actual baseline/top/bottom metric participation for in-flow atomics and controls;
   their final slot and baseline relation are returned by the finalizer. Out-of-flow
   occurrences contribute only their captured anchor before placement; their actual
   canonical rectangle is supplied by the consumer's absolute-placement owner.
   A control strut or per-line `LineStrut` is allocated
   Layout geometry, not an invented font metric. Neutral formatting controls carry
   explicit absence. `line_strut` can allocate a genuine line contribution without
   inventing a control or a font. A taken discretionary break requires its own issued alternative;
   an empty replacement is valid, and a natural end does not imply a hyphen.
4. `PreparedText::select` returns `SelectedLine`. Its slots identify actual selected
   presentation contributions, including replacement spaces. Finalize first with
   `SpacingAllocations::None`, or provide a complete slot table including zeros.
   Foreign, missing and duplicate slots refuse. Actual feedback carries advances,
   occupied/hanging extents, metrics and atomic slots; the consumer decides fit.
   Raw provider glyph advances remain `f32`, separate from logical `f64` pens.
5. Supply `PlacementRequest` to `Layout::from_finalized_lines`. Coverage is the
   actual whole paragraph or issued boundary range. Lines must cover it contiguously
   under the same preparation. Each `LineAnchor` identifies physical local x=0 on
   the baseline. Text maps `(x,y)` as `(anchor.x+x, anchor.y+(y-local_baseline))`
   once; it does not independently snap origins or boxes. Physical atomic rows must
   match their exact finalized flow/box relation. All consumed geometry is qualified
   before the immutable layout is exposed.

Finalized run/glyph views borrow actual arrays and retained resources. Fragment
views distinguish original, native-control and alternative ownership. Replacement
presentation never becomes authored characters. Same-instance outline readback is
optional and can refuse independently of valid shaping; its conservative Text metric
bounds are not Render's positioned canonical paint bounds.

`FinalizedLine::prefix_at` observes the logical advance at an actual occurrence
seam. Use `TextSeam::Origin` with its issued `PreparedOrigin` and a
`SourcePosition` in that origin's original `Source`, or query before/after an
issued mixed participant. Foreign preparations, positions outside the admitted
origin and occurrences outside this selected line refuse. The returned
`FinalizedPrefixView` borrows this exact line and retains the original seam
independently of the descriptor's lifetime.

An interior seam counts the whole smallest unit closed under both selected
graphemes and actual provider-cluster associations. `interior_unit()` lends its
presentation extent, actual original `source_pieces`, selected alternative and
independently checked contribution. `source_range()` is only an enclosing extent;
it does not claim text in contextual coverage holes. The prefix uses the actual
logical metrics-fold receipt, including signed advances and spacing, rather than
glyph ink or subtraction of cumulative values. A unit-local numeric failure
refuses its interior query without invalidating ordinary finalization or unrelated
prefix queries. These observations create no caret or break opportunity and do
not move the original seam. Layout owns float placement and replay under the
[shared-unit policy](../../../docs/text-layout-contract.md#float-encounters-inside-shared-text-units).

`Layout::source_range_fragments(&origin, local)` observes a nonempty original-local
`Range` of an issued `PreparedOrigin` in that placement. It validates the entire
request before returning borrowed `SourceRangeFragments`. A valid fully removed
range can return no units; foreign origins, absent coverage and unsupported
partitions return typed errors. Empty requests are separate from collapsed DOM
Range semantics and refuse here.

`units()` visits actual grapheme-based Range units in content order. Each unit
retains its finalized line, fragments, presentation range and selected original
intersections separately from expanded unit coverage and a selected alternative's
defining source relation. `same_unit` compares actual placement and unit identity,
so equal offsets or rectangles do not establish identity. A partial surrogate or
combining sequence selects the complete unit. An optional multi-grapheme ligature
retains its separate Range units under the
[Range allocation policy](../../../docs/text-layout-contract.md#nonempty-text-range-fragments).

Each unit's `pieces()` lends actual font-metric rectangles, advance edges,
baseline, effective instance and contributing finalized cluster indices. It keeps
distinct font pieces and genuine zero-width rectangles. Shared provider advance
uses the qualified allocation policy while retaining each unit's actual letter,
word and selected spacing contributions. Queries neither reshape text nor choose
fonts. A preserved authored newline belongs to its terminating line; a terminal
empty line adds no selected authored unit. Deleted custody and source-less control
anchors do not invent rectangles. These are native Text observations; DOM
membership, element boxes, collapsed ranges, transforms and web rectangle grouping
remain consumer responsibilities.

`Layout::bounds` reports admitted physical placement bounds. `positions()` issues
placement-bound `PlacedPosition` values from actual interaction occurrences.
`position_geometry`, `move_position` and `position_selection` validate that same
placement. A physical `Hit::Text` retains its exact position. `source_cursor()` is an
explicit original-source projection; it cannot distinguish every source-less line.
Several exact positions can share that projection. A raw source query applies its
source-affinity eligibility rules and need not recover the same physical edge;
retain the exact handle when continuing physical interaction.
Raw source geometric queries refuse outside coverage or where their source endpoint
is genuinely ambiguous. Original extraction and edits retain their separate source
coordinate domain. Original cursor and selection endpoints use the original legal
source stops; executed whitespace deletions do not turn returned endpoints into
processed offsets. Word movement follows actual neighboring source-associated
rows across deleted intervals. Source-only accessibility remains bound to analysis; placed
accessibility enriches the same original semantics with exact placed geometry.

The initial composed profile is horizontal. `TabContext::FontAdvance` executes the
retained-font U+0009 glyph advance, which can be the actual `.notdef` advance; it is
not CSS tab stops. `NoTabs` requires absence, and requested stop/hanging profiles
without implementations refuse precisely. This contribution does not claim all
Text, CSS whitespace, line-opportunity, writing-mode or Layout integration criteria.
See the [owning contract](../../../docs/text-layout-contract.md) for phase ownership
and coherent placement policy, and [reference](reference.md) for supported styles.
