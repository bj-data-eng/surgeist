
Text owns an occurrence-qualified nonempty original-source range query over the
actual placed result. Its ordered observations retain selected source
intersections, expanded typographic-unit coverage where applicable, finalized
line/fragment and placement identities, advance edges and associated font-metric
extent. Native `Layout::source_range_fragments` implements this observation
boundary for admitted Native placements; the complete web consumer remains
separate. Piece rectangles use the actual physical placed basis. Ordinary line
pieces expose directed advance edges in physical inline coordinates and a
baseline in the physical cross coordinate. Ordinary font ascent/descent retain their unrotated before/after distances,
interpreted through the query's actual Layout orientation.
Generic source slots and caret affinities are distinct.
The selected evidence is the [current CSSOM View §9 capture](../references/cssom-view-range--editor-capture-20261022.md),
checked with the [bounded CSSWG and browser-test audit](../references/range-geometry-csswg-tests--20261022.md).
That audit found no adopted resolution for the remaining endpoint choices in
the discussions inspected; participant proposals and collapsed-only tests do
not establish nonempty deleted-source behavior.

Range units use extended grapheme clusters as their basis, with operation-specific
tailoring under [CSS Text's character-unit definition](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#typographic-character-unit).
They are distinct from float-prefix units: an optional Latin `AA` ligature can
share one provider cluster while containing two Range units; a base and combining
mark remain one grapheme-based unit. Partial selection expands the actual Range
unit, not every provider cluster touching it.

A combined Cell retains its actual post-edge semantic EGC partition for Range,
rather than becoming one opaque unit or a one-em rectangle. Partial original
selection closes only its actual EGC; selected source endpoints remain exact and
expanded coverage retains all original members. Removed-only selection units
retain metadata and no geometry. Font-backed pieces use actual retained
horizontal metrics through the executed compression, centering and once-applied
scope/placement frame. Their common metric reference differs from the
font-design paint origin by the retained font baseline offset; do not apply that
offset twice. Native CSS TAB pieces instead consume their executed spans and
real container-referenced em band, without a body font or glyph baseline. Ordinary
line, Cell font and Cell native-layout geometry remain explicit variants.

Preserve authentic per-unit advance positions and allocated adjustments first.
For an admitted monotonic horizontal shaped cluster containing multiple complete
Range units without a finer retained advance partition, allocate its otherwise
unpartitioned shaped advance equally among those units in logical order, mapped
through the actual run direction. A 12.5px `AA` shaped advance gives each unit
6.25px before separately owned adjustments. Compose letter/word spacing,
justification and boundary adjustments at their actual owners exactly once;
do not equally redistribute the whole finalized allocation. This is an explicit
Surgeist metric-allocation policy supported by the
[bounded Blink mechanism](../references/blink-formatting-unit-range--daf70f9ddee4.md#range-unit-advance-allocation),
not a CSS-mandated fraction, independently shaped letter width or glyph-ink clip.
Keep shaping unchanged. Ambiguous or nonmonotonic correspondence, unsupported
unit/font boundaries and unrecoverable adjustment ownership return a precise
capability error. Routine optional Latin ligatures need no font caret metadata
for this qualified fallback. Refusal does not complete unsupported-input work.

For a nonempty range selecting an independently retained zero-advance formatting
unit, Surgeist selects a zero-width rectangle at that unit's actual finalized
post-L1/L2 position. Its block extent uses the unit's actual retained resolved
font ascent/descent around the same placed baseline. Missing required metric
association returns a precise capability error; neither a neighboring glyph,
requested family, line-height strut nor outline ink supplies a replacement.
This forbids query-time font guessing, not valid font selection or fallback
during preparation. Using the actual prepared unit's resolved font is Surgeist's
association policy; CSS does not identify a unique font for every control.
This is an explicit policy for a gap in the pinned
[CSSOM View Range rules](../references/cssom-view-range--editor-capture-20261022.md).
The [bounded Blink witness](../references/blink-formatting-unit-range--daf70f9ddee4.md)
does not establish a universal alternative endpoint convention.

Collapsed ranges retain their separate endpoint semantics. Selecting part of an
actual indivisible typographic unit includes that whole unit. Do not relocate a
selected formatting unit to a neighboring visible run, matching isolate end,
pre-L1 position or chosen caret.
Keep zero-width rectangles; bounding-rectangle aggregation is separate.

For valid, fully covered nonempty selections with no eligible retained presentation
attributable to the selected source, executed trim/discard/segment-break deletions
and Phase-II removed edge spaces yield no geometry observations. Apply the same explicit no-observation
policy to suppressed collapsed duplicates, while preserving their distinct
source anchors and opportunity facts. A surviving space representative supplies
geometry only when its own original contribution is selected. Carrying deleted
or duplicate custody alongside it does not select its advance. Preserved spaces
with zero allocations and retained formatting units remain distinct.
Mixed queries report eligible retained units in content order and account for
the complete requested source coverage; absent or ambiguous mapping is an error,
not a successful empty result. A wholly removed origin needs no synthesized font
or endpoint rectangle. These are Surgeist query dispositions for gaps in the
selected Range rules, using the executed
[CSS Text whitespace distinctions](../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-processing).
These dispositions can differ from Blink's zero-boundary candidates for some
deleted subranges; whole-document fixture agreement does not prove their parity.

A selected replacement with a validated nonempty `Source` definition is eligible
when the requested original occurrence/range positively intersects that defining
interval. Endpoint contact, an insertion anchor or a `Control` origin alone is
insufficient. Admit that replacement's actual presentation Range units; an empty
replacement adds none. Preserve the selected original intersections, defining
source relation, alternative ownership and expanded units separately. Deleted
source custody adds no second rectangle. This is explicit Source-replacement
attribution policy, not a consequence mandated by source-preserving hyphenation.
An eligible original and a computed contribution sharing one actual Range unit
select that unit with truthful contributor provenance. Sharing only an optional
multi-unit ligature does not select the computed neighbor's share. Automatic
hyphens without a replaced authored interval require a typed relation
to their generating selected source boundary/span before a consumer can claim
their geometry; control anchors do not supply that missing capability.

For an automatic insertion, preparation retains its immediately preceding
source-backed grapheme in the same lexical word, with all exact ordered source
pieces, separately from the generating mixed boundary. A nonempty request selects
the indication when it positively intersects any of those pieces under the actual
original occurrence identity. Endpoint contact alone does not qualify. This is
Surgeist's automatic-insertion attribution policy; the
[Range rule](../references/cssom-view-range--editor-capture-20261022.md) supports
partial-unit expansion but does not specify this membership predicate. The relation
consumes no source and is not a replacement interval or a word/style/provider-cluster
envelope. A grapheme crossing origins retains each piece; selected
intersections and attribution remain separate observations. Empty origins and
source-less controls supply no relation. Cross transparent events only under
authentic continuity; never search across a disconnection or skip an authored
zero-width formatting grapheme because its advance or count is zero. Missing or
ambiguous attribution is a precise capability failure. After eligibility, use
the existing presentation Range-unit closure once, preserving shared indivisible
units and separately attributable optional-ligature shares. An empty indication
adds no units. Existing Source and Control replacement rules remain unchanged.

A selected preserved authored newline contributes one zero-inline-advance
font-metric rectangle at its actual finalized position on the line it terminates.
Use that item's retained resolved font and placed baseline, including newline-only
and consecutive-newline lines; missing required metrics or placement refuses.
Keep this zero rectangle in a larger query. Where the admitted processing mode
maps CRLF to one forced-break unit, selecting either original half or both yields
that unit once, with selected intersections separate from expanded unit coverage.
Do not introduce CRLF normalization into other modes or DOM offsets: HTML input
normalization precedes the DOM, and a direct DOM CR is not automatically a CSS
segment break. A terminal newline adds
only its terminating-line rectangle; the following empty line has no additional
selected authored unit. A source-less `br` has no Text-source rectangle. This
explicit newline policy uses neither a chosen caret nor the editing-selection
marker, and does not assert exact browser equivalence. The inspected browser
newline test supports relative vertical ordering; it does not prove this policy's
width, font height, terminal-line exclusion, CRLF mapping or rectangle count.

Layout supplies committed placement and node/occurrence associations. Its
[#860 CSSOM View consumer](https://github.com/bj-data-eng/surgeist/issues/860),
under [#850](https://github.com/bj-data-eng/surgeist/issues/850), validates actual
DOM Range membership/endpoints, maps selected Text portions to those occurrences,
includes required selected-element border boxes in content order, and applies
ancestor/viewport transforms and scroll/zoom once. The complete consumer also
handles collapsed ranges, detached ranges' empty result, and unrendered Text
without a layout occurrence. Those valid DOM cases are distinct from missing
mapping within an admitted placed occurrence. Attached shadow-tree Text cannot
be rejected merely because its ordinary root is not the Document; the inspected
shadow test's cross-root setter case does not define a composed range. Suppressing
descendants' element boxes under the selected-element rule does not suppress
their independently selected Text contributions. Excluding source-less `br`/controls
from Text-source eligibility does not exclude selected-element border geometry.
Native observations may be finer than web rectangles; the specification does
not require one rectangle per grapheme, so consumer grouping and bounding-rectangle
aggregation remain separate. A fixture adapter can request the same Text
observations from explicit captured inputs without claiming the full live join.

The retained RTL PDI case therefore keeps native x=24, while the authenticated
Chrome Range reports x=72. These remain explicit strict oracle differences,
not passing or unsupported cases, and neither coordinate is claimed uniquely
required by CSS. Qualify font versus line metrics, independent zero units versus
partial shared units and collapsed ranges, exact source/placement admission,
LTR/RTL, zero-rectangle retention and failure atomicity. No bidi-ordering change
or browser-number substitution follows from this policy.

