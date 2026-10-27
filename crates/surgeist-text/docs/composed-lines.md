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
   Capture `terminal_edge` as `ParagraphTerminalEdge::Unknown`, `BlockEnd`, or
   `NonBlockBoundary` from the actual normalized block relation. A complete
   paragraph/source range alone does not prove block completion. Contextual
   views derive this fact from the sole validated scope/paragraph event stream,
   retaining occurrences inside the final paragraph separately from later ones.
   `paragraph_strut: ParagraphStrut::DefaultFont` requests an actual base-style
   font strut on every native line, even when body spans select other styles.
   `AllocatedContributions` omits that base strut; actual body and supplied
   allocations determine metrics. A selected empty line still needs a genuine
   contribution, including a terminal empty line. Explicit zero differs from absence.
   Supply connections, ordered anchor occurrences with original endpoint affinity,
   controls, wrapping facts and any actual computed discretionary replacements.
   Capture genuine wbr as `ParagraphControlKind::Wbr { style: Box<Style> }` with
   its complete computed Style and actual ordered control occurrence. An expanded
   wbr contributes text and retains its control handle; it needs no additional
   control participant or allocation. Bind its contributing control node when
   lowering to Layout, including units with no original text pieces.
   `AnalyzedSource::from_paragraph` validates this complete capture and retains the
   executed `ProcessedSource`; original code-unit ownership remains available
   through the processed correspondence. Contextual casing selected with
   `TextTransform::Set` accepts a `TextTransformSet` built with `.with_case`.
   `Style::word_space_transform` replaces eligible original U+200B before casing
   with `WordSpaceTransform::Space` or `IdeographicSpace`. Forced-break adjacency
   suppresses replacement in actual post-Phase-I order. Casing follows, then width when `.with_full_width(true)` is
   enabled on that set, then full-size-kana with `.with_full_size_kana(true)`.
   Width maps original SPACE only when actual Phase-I processing preserved it;
   separator-generated SPACE qualifies in every mode. No extra collapse pass runs
   over generated output. Later line-edge rules retain their actual style and
   character semantics. These operations run before semantic
   analysis. Original/Phase-I offsets remain distinct
   from actual transformed presentation offsets; plaintext excludes generated
   separator characters. See [native wbr admission](reference.md#native-wbr-separators)
   for exact scalar seams and indivisible-input refusals.
   For Japanese automatic phrases, use contextual capture with actual scope
   topology and Styles, then `SpaceAutoPhrase` or `IdeographicSpaceAutoPhrase` on
   the requesting text. The winning scope can differ from that requesting text.
   Bind each issued contributing scope to the actual Layout container or complete
   inline boundary pair, retaining source-less virtual identity in fragment output.
   See [phrase capture](reference.md#native-automatic-phrase-separators) for native
   language admission, missing-context refusals and conservative detection limits.
   For the exclusive `TextTransform::MathAuto`, supply each demanded origin's
   complete original Text-node content and node-relative contribution through
   `ParagraphOrigin::text_node`. Eligibility uses that original complete content;
   execution maps its actual Phase-I survivor. Missing demanded context refuses
   before trimming or deletion. An origin slice alone supplies no node authority.
   See [transformation and source custody](reference.md#defaults-and-validation).
2. Call `System::prepare(&analysis, PreparationOptions)` with the actual scale and
   `TabPreparation::FontAdvance` or `TabPreparation::CssStops` demand.
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
   `Style::hyphens` defaults to `Manual`; `None` suppresses both bare SHY and
   explicit hyphenation groups and automatic points. `Auto` also permits the
   exact [en-US resource](hyphenation-resources.md) over complete qualified
   lexical words, with retained source-exception and correction provenance.
   Native preparation consumes each cut's actual `Style::hyphenate_limit_chars`
   computed triple, including empty indications. Each symbolic `Auto` minimum
   independently resolves to total/before/after 5/2/2; explicit counts, including
   zero, compare against the same complete-word map. Text supplies no CSS
   specified-value expansion; see the canonical
   [count-unit policy](../../../docs/text-layout-contract.md#automatic-resources-word-custody-and-initial-limits).
   Raw `System::layout` and `layout_analyzed` retain their distinct unconstrained
   count operation. Borrow `word_views()` and `hyphenation_cut_views()` for
   actual lexical/resource/count and source-piece observations. Automatic
   alternatives carry `PreparedAlternativeDefinition::Automatic`, consuming no
   authored interval. Their cut retains the preceding original EGC separately
   from the generating mixed boundary and computed presentation. A word's
   `ordinary_fallbacks(start)` exposes genuine Allowed/no-alternative endings
   immediately before its first influencing text occurrence. It preserves mixed
   custody and does not nominate synthetic empty endings or make fit decisions;
   Layout owns the natural fallback trials used by its automatic hyphenation zone.
   `ParagraphReplacement::style_driven` resolves the actual defining style;
   `resolved_native_logical_before` captures one nonempty ordered list (individual
   strings may be empty). A discretionary control owns its complete boxed Style.
   There is no replacement style anchor or independent wrapping flag.
   Exact `hyphenate_character` text wins over convention text. Auto uses an
   applicable `NativeHyphenationConvention`, otherwise probes the first available
   font among the owning style's families/resolved generics and actual attributes.
   This distinct probe has no script fallback or glyph-coverage filter: raw native
   catalogue registrations have unrestricted unicode ranges, and a space glyph is
   not required. It chooses U+2010 when present, otherwise U+002D; normal shaping
   then resolves the chosen text. LogicalBefore and LogicalAfter use independent
   indication-local bidi attached to authenticated original levels; generated
   strings never change original paragraph UBA. BothSides emits the same exact
   resolved text once Before and once After, using each side's actual attachment
   and fresh joint shaping; it remains one selected ending for line-count policy.
   A nonempty LogicalAfter or BothSides ending
   lends `HyphenationContinuation` through `hyphenation_continuation()`. Supply
   that exact choice in the next `LineSelectionRequest.incoming`; its indication
   is shaped on the receiving line using its own owning style and actual receiver
   direction. `PreparedWordPortion::incoming()` retains the corresponding
   hypothetical manual choice for measurement. Conditional queries retain that
   incoming identity too. No emission is pending for an empty indication.
   Missing primary fonts, missing receiving authority, unsupported hard separators
   inside an indication and incompatible local/joint
   shaping order return precise errors. Actual replacement TABs retain the chosen
   CSS-stop or FontAdvance operation with local bidi-S and line L1 handling.
   Combined query units require exact font-match witnesses
   retained by preparation; covering glyphs or two partial matches cannot replace
   one complete query. Font witnesses never impose shaping boundaries or retain
   final glyphs. Suppressed or empty indications do not consume unused capabilities.
   Native conventions bind
   the complete locale, including variants/extensions. Indication conventions
   have no built-in language table; broader hyphenation resources or CSS font-face
   unicode-range support is not claimed.
   Enabled choosers prefer admitted manual partitions before finite automatic
   fallback. Measure exact `PreparedWordPortion`s through
   `PreparedTermination::Measurement` in the actual otherwise-empty candidate
   frame; these are not break offers and cannot be placed or used as phantom
   receipts. Finite decisions retain actual float/indent/tab/resource context.
   Min-content instead permits automatic refinement of those portions; maximum
   remains unbroken. Conditional queries require complete earlier variant
   dispositions, keeping actual phantom receipts separate from automatic
   priority, fit-exhaustion, intrinsic and successive-line-limit dispositions.
   `AutoLineLimit` covers only the exact automatic variant, before a suppressed
   trial: finite Overflow is the consumer's exhausted-capacity decision and
   MinContent accepts only static zero. It does not advance the phantom frontier,
   suppress a manual sibling, or certify consumer counter/fit arithmetic. Text validates their
   owners and roles; the consumer owns actual fit and recomputes on replay.
4. `PreparedText::select` returns `SelectedLine`. Its slots identify actual selected
   presentation contributions, including replacement spaces. Finalize first with
   `SpacingAllocations::None`, or provide a complete slot table including zeros.
   Foreign, missing and duplicate slots refuse. Actual feedback carries advances,
   occupied/hanging extents, metrics and atomic slots; the consumer decides fit.
   Raw provider glyph advances remain `f32`, separate from logical `f64` pens.
   Line-height changes preserve compatible joint shaping: each actual component
   retains its own style's height resolved against the shared font instance.
   Native line struts, preliminary unit heights and source-fragment bounds consume
   those contributions without assigning every unit the run's maximum height.
   Borrow `FinalizedLine::measurements()` for native fit, alignment and the
   min/max-content contributions of this exact selection. These signed flow
   observations exclude indentation; their ordered occupied intervals are in
   the prealignment line frame. The consumer supplies indentation once and
   still decides fit. `metric_bounds()` observes the actual Text metric frame,
   independently of a consumer's intrinsic allocation.
   `edge_units()` distinguishes natural eligibility and terminal grouping from
   final allocated exclusion and physical outward overlap, with genuine issued
   participant/alternative owners and original pieces. It issues no break or
   replay capability. A hanging contribution can lie physically inside a wide
   frame, and a reverse contribution is not an absolute width.
   `UnconsumedTerminal` observes a neutral row whose terminal authority was not
   consumed; it certifies neither an ordinary classification nor a block end.
   Natural classification retains that row and its exact earlier-group
   dependencies through spacing. If allocation makes one material, finalization
   requires the real terminal fact atomically; unrelated allocations stay valid.
   An eligible preserved terminal sequence consumes missing block-end context
   only when its actual measures need the distinction; genuine Soft/Forced
   endings, nowrap and independent ordinary content retain their own semantics.
   See the [native whitespace contract](../../../docs/text-layout-contract.md#native-whitespace-edges-and-hanging)
   for current modes, supplied-spacing remapping and remaining styled barriers.
   `LineContext::formatted_line` separately carries actual First/Last container
   roles. Unknown facts are demanded lazily by eligible hanging punctuation;
   `formatted_line_usage()` reports consumed roles even when their supplied value
   was false. An ending enum or source offset cannot supply those roles.
   A detached measurement obtains prospective authority from its actual owner,
   accepted entry and issued measurement-flow observations. Those observations
   expose retained participation, not generated-line proof or placement rights.
   See [hanging punctuation](reference.md#hanging-punctuation).
5. Supply `PlacementRequest` to `Layout::from_finalized_lines`. Coverage is the
   actual whole paragraph or issued boundary range. Lines must cover it contiguously
   under the same preparation. Whole coverage requires an empty first incoming,
   exact outgoing/incoming pairing and no unfulfilled final obligation. Range
   coverage may retain an outgoing choice or begin with its authentic incoming;
   it makes no claim about an omitted predecessor. Internal pairs still match.
   Incoming ownership never extends original source-query coverage.
   Each `LineAnchor` identifies physical local x=0 on
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
presentation extent, actual original `source_pieces`, every actual `alternatives()` owner and
independently checked contribution. `source_range()` is only an enclosing extent;
it does not claim text in contextual coverage holes. The prefix uses the actual
logical metrics-fold receipt, including signed advances and spacing, rather than
glyph ink or subtraction of cumulative values. A unit-local numeric failure
refuses its interior query without invalidating ordinary finalization or unrelated
prefix queries. These observations create no caret or break opportunity and do
not move the original seam. Layout owns float placement and replay under the
[shared-unit policy](../../../docs/text-layout-contract.md#float-encounters-inside-shared-text-units).

`has_retained_flow_participation()` reports participation before the original
seam, independently of whole-unit advance charging. Retained zero/signed text,
in-flow atomics and allocated control struts count; empty or discarded occurrences
add nothing. A later part charged by an interior seam does not become preceding
content. This observation does not establish visibility or a generated line box.

`Layout::source_range_fragments(&origin, local)` observes a nonempty original-local
`Range` of an issued `PreparedOrigin` in that placement. It validates the entire
request before returning borrowed `SourceRangeFragments`. A valid fully removed
range can return no units; foreign origins, absent coverage and unsupported
partitions return typed errors. Empty requests are separate from collapsed DOM
Range semantics and refuse here.

`units()` visits actual grapheme-based Range units in content order. Each unit
retains its finalized line, fragments, presentation range and selected original
intersections separately from expanded unit coverage. `alternatives()`,
`defining_sources()` and `hyphenation_cuts()` expose all genuine contributing
owners/relations, including a shared incoming/outgoing mark EGC. `same_unit` compares actual placement and unit identity,
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
Expansion interiors also retain output subpositions that a source cursor cannot
recover. Several exact positions can share a projection. A raw source query applies its
source-affinity eligibility rules and need not recover the same physical edge;
retain the exact handle when continuing physical interaction.
Raw source geometric queries refuse outside coverage or where their source endpoint
is genuinely ambiguous. Original extraction and edits retain their separate source
coordinate domain. Original cursor movement uses the original legal source
stops; executed whitespace deletions do not turn returned endpoints into
processed offsets. Word movement follows actual neighboring source-associated
rows across deleted intervals. Source-only accessibility remains bound to analysis; placed
accessibility enriches the same original semantics with exact placed geometry.

`position_selection` returns `PlacedSelection`. Its `geometry()` covers the selected
presentation units; `source_pieces()` retains normalized original intervals with
their actual `PreparedOrigin` occurrence. `source_text()` extracts exact original
UTF-16 once per covered interval, while `plaintext()` applies the processing owner's
separate policy. With `ß → SS`, selecting only the receiving S has receiving-only
geometry and extracts original ß. Selecting both S pieces does not duplicate it;
equal text in distinct original occurrences remains distinct. Reversal preserves
coverage, and `is_collapsed()` compares exact relation cuts. Removed components
can have distinct cuts and source coverage at the same geometry, without creating
glyphs or additional wrapping opportunities.

Native composition uses horizontal shaping and logical line measurements.
Explicit placement retains a Horizontal or clockwise Sideways basis;
[sideways preparation and physical readback](reference.md#clockwise-sideways-placement)
describe its feature settings, axis coordinates and capability limits. Tab
origins and spacing requests remain in the local inline frame.
`TabContext::FontAdvance` executes the
retained-font U+0009 glyph advance, which can be the actual `.notdef` advance; it is
not CSS tab stops. `NoTabs` requires actual selected absence. CSS preparation
uses `TabPreparation::CssStops` and a captured `ParagraphTabReference` containing
the real nearest block Style and used direction. It readies every selectable
original and replacement tab under the exact catalogue and scale. The line
request supplies `TabContext::CssStops { origin }`: the physical block start
relative to unaligned paragraph start, or `None` for a genuinely unknown phase.
Nonzero tabs consume that phase; zero tabs retain custody without unused font
probes. Hypothetical character metrics use the native isolated Common-to-Latin
profile with actual settings, coordinates and locale: SPACE follows the captured
block direction; ZERO retains its even/LTR level. This does not borrow a source
run script or claim arbitrary script-sensitive metric conformance. The shared
finalizer orders the mixed line, resolves glyph-free tab
advances with actual spacing/atomic contributions, then builds prefix, metrics,
paint and interaction rows. Re-finalization recomputes stops; alignment follows.
Fontless tabs cannot invent font metrics for Range observations. A nonzero signed
spacing-derived interval retains its actual SPACE and ZERO resources. Both
nonzero signs use the same forward grid with the interval's magnitude and exactly
one threshold bump; this is an explicit native policy. Consumed unknown intrinsic
phase remains a precise capability refusal. Native whitespace-edge measurements follow the captured
terminal role and actual line ending; callers do not supply numeric hanging.
Other whitespace modes, styled boundary barriers and upright/mixed typography remain
outside this contribution.
See the [owning contract](../../../docs/text-layout-contract.md) for phase ownership
and coherent placement policy, and [reference](reference.md) for supported styles.

`FinalizedLine::has_in_flow_content()` observes retained actual text/presentation
and in-flow atomics, including zero or signed contributions. Hard-separator-only
rows, control/default struts, discarded text and out-of-flow roles do not add
content. This differs from prefix retained-flow participation, which also
observes separator and allocated-control roles; neither predicate proves block
completion. Layout owns the completed unfragmented terminal-tail policy.

For finite branch-local reselection, `ConditionalDisposition::ConsumerHyphenationPolicy`
can discharge an exact rejected hyphenated ordinary trial. Text authenticates the
trial's preparation, unchanged start, alternative and context, not the consumer's
policy verdict. Meaningful no-alternative, Forced and Paragraph variants remain
barriers; these receipts neither advance phantom source nor admit intrinsic use.

The [indication bidi contract](../../../docs/text-layout-contract.md#hyphenation-indication-bidi)
defines the explicit local operation and its remaining capability limits. There
is no language-convention table, independent per-side chosen strings, or general
guarantee that arbitrary local control strings compose with joint provider
units/runs.
