# Reference

This page describes the current crate interfaces and local tooling. The
[manifest](../Cargo.toml), [public crate root](../src/lib.rs), and linked source
files own their current definitions.

## Package and features

| Item | Current value |
| --- | --- |
| Package / library | `surgeist-layout` / `surgeist_layout` |
| Version | `0.1.0` |
| Rust edition / minimum version | `2024` / `1.97` |
| Default features | None |
| Optional feature | `layout-golden-generate` |
| Generator binary | `surgeist-layout-generate`, requiring `layout-golden-generate` |

The optional feature enables the local `surgeist-generator` browser-corpus
engine and its Serde dependencies. The path/version requirement is in the
manifest; resolved dependencies are in the shared [Cargo.lock](../../../Cargo.lock).
The package excludes the separate `tools/surgeist-layout-audits` tree.

## Scalar precision

`surgeist-layout` keeps its default public API at browser-style coordinate
precision: `DefaultScalar` is `f32`, and `Scalar` aliases `DefaultScalar`.
Most applications should use the default aliases such as `NodeInput`,
`ComputeInput`, and `ComputeOutput`.

For applications that need more coordinate precision, scalar-bearing APIs also
provide generic `*Of<S>` forms, and `LayoutScalar` is implemented for both
`f32` and `f64`. Pick one scalar type for a layout tree and run it end-to-end:
do not mix `f32` and `f64` values inside one tree, cache, traversal, or layout
run.

Browser parity XML and its generator remain a default-precision fixture
boundary. Use those fixtures to check the default `f32` contract; add separate
crate-local tests when a behavior specifically needs `f64` coverage.

## Public API map

All public paths are exposed directly from the `surgeist_layout` crate root in
[`src/lib.rs`](../src/lib.rs).
Default-scalar aliases use `f32`; the corresponding `*Of` forms support either
public scalar lane.

### Computation and tree

Sources: [engine](../src/engine/mod.rs), [measurement](../src/measurement.rs),
[tree](../src/tree.rs), [cache](../src/cache.rs), and [errors](../src/error.rs).

- Tree layout: `compute_layout` and `compute_layout_invalidated`; checked inline
  flow with optional Text: `LayoutFlowInput`, `compute_layout_with_flow`, and
  `compute_layout_with_flow_invalidated`.
- Direct leaf measurement: `compute_leaf`.
- Host contracts: `Traverse`, `LayoutTree`, `WrapEdgeSource`, and `LayoutBatchSink`.
- Early native Text edges: `process_wrapping_source`, `ProcessedWrappingSource`,
  and `ResolvedWrapEdges`; [source admission](../src/inline_wrapping/source.rs).
- Cache and failure contracts: `CacheOf`, `LayoutResultOf`, and `LayoutErrorOf`.

### Node input

Source: [node input](../src/node_input/mod.rs) and its domain modules.

- Aggregate input: `NodeInputOf` and its `NodeInput` default-scalar alias.
- Tree participation: owned `LayoutInputOf` storage and borrowed `LayoutInputViewOf`, `Display`, `Position`, and `ItemOrder`.
- Flow and alignment: `WritingMode`, `Direction`, `AlignItems`, `AlignContent`,
  `TextAlign`, `TextAlignLast`, `TextJustification`, and `TextWrapStyle`.
- Inline and flex input: `TextParticipation`, `TextControlParticipation`, `TextParagraphBinding`,
  `InlineBoundaryInputOf`, `FlexDirection`, `FlexWrap`, and `FlexItemCollapse`.
- Grid input: `GridPlacement`, `GridAutoFlow`, and `GridFlowToleranceOf`.
- Scroll input: `ComputedOverflow`, `OverflowClipMarginOf`, `ScrollPaddingOf`, `ScrollMarginOf`, and `ScrollSnapType`.

`NodeInputOf::text_wrap_style` defaults to `TextWrapStyle::Auto`. `Balance`
selects among the same legal endpoints within each actual forced-break group,
using natural remaining inline space before justification. It preserves Auto's
formatted-line count in its bounded two-to-ten-line search; larger groups use
Auto. The preference does not narrow containing geometry, change tab or
percentage bases, or authorize conditional overflow breaks. Numeric atomic and
control-only runs use the same preference through their existing fitter.
Intrinsic sizing and forced-only nowrap retain their ordinary behavior. This is
an unfragmented, unclamped profile; the other CSS wrap styles are not exposed.

Prepared Text participation carries actual issued origin, atomic, control and scope
handles. Capture validates complete inventories against the same borrowed tree;
equal characters from another preparation cannot replace that basis. Layout
selects actual issued break opportunities and alternatives, then consumes Text's
finalized metrics, fragments, control slots, and atomic slots. At a discretionary
endpoint it tries alternatives in declaration order and uses the first that fits;
it keeps the first alternative when an indivisible candidate must overflow.
`TextControlParticipation` supplies the exact issued semantic control with no
allocated strut. Its direction, wrapping, and source-less alternative facts stay
Text-owned. Existing line breaks and inline boundaries instead supply their real
Layout strut metrics and must match the corresponding issued semantic role.
`LayoutFlowInput::try_new(tree, root, paragraphs, wrapping)` also admits
box/control-only flow with no Text paragraphs. `WrappingInput` supplies actual
scope declarations and generated child-order proofs. `WrappingScopeInput::NativePair`
declares genuine normalized endpoints; `Captured` declares an authentic inline
capture identity. `WrapInside::Avoid` ranks permitted fitting breaks through the
same validated scope forest. A captured scope without endpoints supplies semantic
ownership and no physical border or padding authority. Ordinary `compute_layout`
keeps automatic wrapping without a declared scope context.

Avoidance prefers a fitting cut outside an avoided box over one inside it, then
prefers an outer cut over a cut inside a nested avoided box. Incomparable sibling
chains retain the fitter's normal preference. The preference is noninherited and
does not create or remove opportunities: forced ends remain mandatory, necessary
internal breaks remain usable, and min-/max-content keep their ordinary opportunity
rules. Actual finalized spacing, hanging punctuation, line metrics and float bands
determine fit before preference is applied. Changed scope declarations require a
new checked flow; invalidation includes their actual endpoint and owner dependencies.

`NodeInputOf.wrap_edges` supplies before/after declarations for actual atomic
inline boxes and flex items. Non-atomic boxes carry the same `WrapEdges` in their
scope declaration. `Avoid` discourages both matching kinds of break;
`AvoidLine` and `AvoidFlex` affect only their respective kind. `Line` forces an
inline break; `Flex` partitions actual items only in a multiline flex container.
Applicability follows the normalized box role. Flex exterior forces add no empty
line, while collapsed items retain their real strut participation.

Numeric inline forces use genuine participant cuts, including leading/terminal
mandatory progress, and partition maximum-content groups. Propagation consumes
actual first/last in-flow child evidence; `InlineChildOrderInput` distinguishes
the paired box from its endpoints and declares prefix/suffix completeness.
Missing demanded incidence returns `MissingChildOrder`. Mixed inside/edge
avoidance uses the [independent-constraint policy](../../../references/text-wrap-edges--native-determination.md#independent-avoidance-constraints).
Avoidance adds no ordinary opportunities.

Text-backed soft edge avoidance selects one Text-issued original incidence of
the existing boundary, with authentic generated-box order and association.
Genuine empty captured boxes can have distinct before/after choices; one choice
governs every participating scope. Each material box-metric assignment is fitted
before preference selection. Missing correspondence returns
`UnsupportedIncidence`; see the [original-cut contract](../../../references/text-wrap-edges--native-determination.md#original-box-cut-incidence).
A Text-backed `Line` declaration requires source processing before preparation.
Ordinary `LayoutFlowInput::try_new` returns `SourceProcessingRequired` at this
late admission boundary. Computed mandatory boundaries already prepared by Text
retain their separate issued causes.

`process_wrapping_source` accepts a borrowed `WrapEdgeSource` projection and one
Text `ContextualSourceInput`. Source scopes and occurrence associations refer to
that input's actual inventories; the early projection needs no prepared handles.
The operation derives applicable native forces, preserves existing computed
requests, processes Text once, and authenticates the returned capture before
issuing `ProcessedWrappingSource`. Its `into_parts` returns the processed source
and an owned `ResolvedWrapEdges` receipt, ending the early host borrow.
An explicitly declared native pair retains its real endpoint-control identity
when the input has no matching inline scope.

After normal Text preparation and host binding, `LayoutFlowInput::try_new_with_edges`
accepts the current tree, actual paragraph bindings, final `WrappingInput`, and a
borrowed receipt. The tree also supplies a fresh `WrapEdgeSource` projection.
Admission checks current roles, declarations, relevant generated order and
source/cause associations against both the receipt and actual prepared inputs.
Changed semantic evidence returns `PlanMismatch`; invalid source associations
return `SourceAssociation`. A coherent new preparation from the retained capture
can reuse the receipt. The receipt authorizes no stale tree or foreign capture.

Used box metrics remain final `WrappingInput` facts. They are not resolved from
early source styles or frozen as absent by the receipt. A numerical change alone
can reuse its semantic proof; the normal metric/fit/cache owners consume the new
values. See [the source-admission guide](how-to.md#prepare-native-forced-text-edges).

Complete generated child order can include genuine inline scopes without physical
endpoints. Native spans establish the physical projection; canonical capture
facts establish the full parent/order and consumed prefix or suffix. An empty
in-flow child still determines first/last-child propagation. Admission rejects
contradictory or unavailable required proofs instead of dropping that child.
Atomic exterior edges require the same parent and consumed-side proof. A
continuing inline uses its actual parent-domain paragraph coverage; intervening
block content retains its own domain.

Both scope variants carry `metric_contribution`.
`Some(InlineBoxMetricContribution::Strut(...))` supplies actual used geometry and
alignment; `Some(NoContribution)` affirmatively supplies absence, and `None`
leaves the evidence unsupplied. Influential boxes need a real witness unless an
actually selected same-box control allocation already owns that contribution.
Glyph or descendant-font containment cannot establish the box's own strut.
Aliases share one supplied witness; contributions are not duplicated.
Missing, invalid and contradictory evidence return `MissingBoxMetrics`,
`InvalidBoxMetrics` and `ConflictingBoxMetrics`, respectively. See the
[metric ownership contract](../../../references/text-wrap-edges--native-determination.md#actual-box-metrics).

The accepted original incidence and box attribution continue into the receiving
line, actual allocation feedback, replay and cache identity. This custody is
independent of the receiving preparation's spelling boundary. Text combines
allocated box struts with its existing metric groups; numeric-only native flow
uses its existing metric owner. Equal glyph prefixes do not establish equal
box assignment or float clearance.
At paragraph completion, the outgoing frontier must close inline boxes whose
genuine exits precede that paragraph's exit. After an actual Forced selection,
Text's issued ParagraphEnd selection can complete remaining boxes within the
same actual scheduled paragraph. Layout retains this proof separately
from the preceding original pin, which remains unchanged. The proof carries the
genuine selected sequence, incoming box attribution and captured paragraph horizon
through measurement, replay and reuse. CSS phantom disposition is independent of
numerical metric completeness: a strut-only Paragraph receiver retains completion
custody but contributes no ordinary published line or block extent. See the
[completion and disposition contract](../../../references/text-wrap-edges--native-determination.md#paragraph-completion-and-retained-pins).
Missing demanded box metrics still return `MissingBoxMetrics`; missing completion
authority returns `UnsupportedIncidence`.

Box/control-only native runs also distinguish nominal metrics from formatted
line existence. Empty zero-decoration endpoints retain their metrics and
source-indexed descendant outputs with zero used height; they do not establish
a formatted ordinal, baseline, float band or adjoining-margin separator. A real
in-flow atomic, each individual nonzero supported inline border or padding, and
an actual Forced ending remain formatted even with zero extent. A terminal
strut-only receiver does not inherit the preceding Forced status. These native
results use the numeric owner directly and do not manufacture Text receipts.

Captured paragraphs whose only prepared participants are genuine computed
mandatory boundaries can execute without host child nodes. Layout proves their
unique native gap from the existing capture/domain/graph associations and orders
shared-gap paragraphs by canonical events. Empty, leading and trailing gaps use
the same block/float/fitter path as ordinary runs. Exact preparation and paragraph
identity prevent empty-vector lookup and result collisions through replay and
publication. Real Block interruptions retain their child-domain associations;
missing, foreign or ambiguous correspondence refuses. Strict zero-participation
AssociationOnly metadata stays nonrendering, and numerical struts alone do not
create work. Known-size and cache paths retain actual semantic baseline/metric
work; whole-paragraph placement applies the same phantom disposition above.

Native non-atomic endpoints currently carry their actual border and padding;
they do not represent signed inline margins. That remaining geometry capability
belongs to [fragmented inline decoration work](https://github.com/bj-data-eng/surgeist/issues/780).
An atomic's signed margins cannot stand in for a non-atomic margin-only witness.

Text-backed scopes use authenticated mixed-prefix and canonical source incidence,
including transformed, collapsed and removed content. A semantic captured scope
can span a whole typographic unit without physical endpoints. Physical paired
endpoints that cannot preserve genuine participant order across that unit remain
unsupported; Layout refuses the association rather than moving endpoints around
the glyph. `WrappingScopeError` identifies invalid or unprovable scope facts.
Contradictory numeric metric/band feedback returns `InlineFeedbackCycle`; the
Text-backed fitter retains `TextFeedbackCycle`. Neither publishes a partial batch.

The preparation retains its actual `ParagraphStrut` policy. `DefaultFont`
contributes the captured base font on each native line; `AllocatedContributions`
uses real selected text, boxes, controls, and any explicitly supplied Layout
`line_strut`. Missing metrics remain a typed refusal rather than a guessed font
or zero-size line.

For virtual phrase output, capture derives one association for each needed
canonical Text scope from its genuine captured endpoint controls and the actual
validated control-node bindings. Inline scopes retain their distinct normalized
Start/End node pair across paragraph cuts; domain scopes associate with each
paragraph's actual container. `TextScopeBinding` and `TextScopeAssociation` expose
these derived associations on retained fragments. Callers supply no separate
scope-pair inventory. Equal source offsets or another balanced pair cannot
establish identity. Scope Style stays Text-owned, boundary metrics stay
Layout-owned, and generated geometry remains local to its own prepared paragraph.

Flow proof covers each contributing inline owner's authentic extent in its own
domain, or the full domain for a contributing domain owner. Disjoint extents keep
their gap. Known ancestor endpoints establish ancestry without demanding unrelated
interiors. Capture compares Text's owner-issued flow projection with actual tree
and run order inside those extents. Atomic contents validate independently through
their exact nominated atomic node, so deferred capture after a later sibling does
not change ownership. Plain child Blocks inside a required extent need their real
container association. Missing, foreign, reused, crossed or mismatched facts
remain typed refusals. Needed scope, endpoint, container and tree dependencies
participate in cache and invalidation checks.

A genuine captured paragraph with no original, control, atomic, participant or
generated-contributor inventory can associate an empty domain through the existing
paragraph binding. Its supplied node and binding inventories must also be empty,
and `line_strut` must be absent. This association adds no Text run, line, strut,
fragment or placed Text output; ordinary Box geometry is unchanged. Removed-only
source and omitted real bindings do not qualify. Empty incidence must be authentic
and unambiguous, and an empty paragraph does not certify that its whole domain is
empty. Unknown inline topology does not erase genuine canonical paragraph/domain
facts; missing facts needed by an active proof still refuse. See the [Text/Layout contract](../../../docs/text-layout-contract.md) for
the shared ownership and placement boundary.

The native profile supports horizontal LTR and RTL text, signed indentation,
and the preparation's selected font-advance or CSS-stop tab operation.
For `TabPreparation::CssStops`, Text retains the actual nearest-block metric
reference; capture checks its used direction against the normalized container.
Finite used-layout candidates supply `CssStops` with that block edge relative
to the current unaligned paragraph start. Floats and retries change this
displacement; indentation changes the pen without restarting the grid. A
selected nonzero tab needs the actual used origin; zero or absent tabs do not.
Later alignment moves the completed line. Spacing allocations can be absorbed
by shorter tab shifts, so fit and placement consume actual finalized advances.

Unconstrained min-/max-content candidates select `IntrinsicCssStops`, using a
logical grid from the unindented paragraph start and the applicable line indent.
This operation is independent of block direction and incidental placement width.
Each measured line resets its logical pen; subsequent finite layout recomputes
tabs from the real block edge. Request and cache identity distinguish both
operations even when their numbers agree. A finite negative spacing-derived
interval retains its signed metric and uses the same forward grid with the
interval's magnitude as period. The [tab contract](../../../docs/text-layout-contract.md#preserved-css-tab-stops)
records these native policies, metric ownership and source boundaries.

`text_align` supplies normal line alignment;
`text_align_last` supplies paragraph-ending and forced-line alignment. Logical
Start/End follows the paragraph direction, while Left/Right remains physical.
Only the Legacy alignment variants also align ordinary block children.

`text_indent: TextIndentOf<S>` retains a signed length/percentage and the
`each_line` and `hanging` flags. Percentages resolve against the block's own inner
inline size, before float reduction; intrinsic sizing uses only the absolute
component. A 25% indent in an 80px block is 20px even when floats leave a 50px
span. Negative and oversized indents retain their actual overflow.

Default indentation follows the first formatted line. `each_line` instead affects
the first line of an actual block container and lines after genuine forced
breaks; `hanging` inverts that predicate. Only accepted lines advance this state,
including empty lines. Candidate retries and clearance do not consume it.
Text-bearing and box/control-only inline runs use the same policy.

Prepared Text hanging punctuation uses explicit First/Last formatted-container
roles. Layout supplies actual First state and validates demanded Last assumptions
against accepted Text, numeric, empty and Forced lines, including genuine later
in-flow children. Endpoint-based replay permits a terminal punctuation candidate
to absorb a former last line. Detached full-word/phrase trials retain prospective
entry and remaining-flow authority without committing line history. Intrinsic
and finite invocations keep separate chronology; geometry-only cache hits lacking
a demanded summary are recomputed. Text owns punctuation classes and signed
advance accounting, and Layout consumes its fit/alignment/intrinsic receipts.
See the [shared formatted-line contract](../../../docs/text-layout-contract.md#hanging-punctuation-and-formatted-line-roles).

`block_generation` distinguishes principal boxes from actual anonymous blocks.
For anonymous blocks, Layout derives first-line eligibility through the first
in-flow-child chain; an earlier empty in-flow block consumes that position.
Hidden, absolute and floated predecessors do not. The admitted anonymous profile
is in-flow BlockFlow, with the generating owner captured and matching inherited
computed indent values. Missing context and incompatible roles refuse explicitly;
these inputs do not authenticate live computed Style.

One prepared paragraph can span interleaved floats in its captured source order.
Layout queries Text's finalized prefix at each encounter, places the actual float
box, and rechecks the selected line against the resulting spans and metrics.
An encounter inside a shared text unit follows the
[shared-unit policy](../../../docs/text-layout-contract.md#float-encounters-inside-shared-text-units);
it does not split that unit or create a break opportunity. Earlier float tops
constrain later float placement, including floats preceding the capture.
Rejected candidates discard their child output, caches and pending Text results;
accepted candidates publish their actual float placements once.

Retained source participation determines whether an encounter has preceding
in-flow content, independently of zero or signed prefix advance. Hidden and
absolute participants retain their own traversal and static-position effects.
Separate paragraph bindings retain their genuine terminal receipts. A
strut-only Paragraph completion publishes no ordinary line; actual Forced
endings remain real. A discarded-only soft endpoint contributes no phantom
line. Minimum sizing includes the largest individual float contribution;
maximum sizing combines each actual line with its applicable float occupation
before the final finite placement pass. Clearance and expired float cohorts
remain distinct.

Automatic hyphenation consumes Text's exact
[en-US resource](../../surgeist-text/docs/hyphenation-resources.md) and issued
word/cut ownership. Finite fitting preserves admitted manual partitions while
measuring their residual portions through the same Text finalizer in the actual
otherwise-empty candidate frame. Those trials retain preceding source-float
encounters and never publish a line or child placement. Min-content instead
allows automatic refinement within manual partitions; max-content stays
unbroken. Neither operation infers fit failure from an absent finite width.
Ordinary indication-inclusive trials precede nonhyphenating emergency cuts;
automatic variant dispositions do not turn meaningful content into phantom flow.
`NodeInputOf::hyphenate_limit_lines` supplies a block-owned `NoLimit` or finite
count. Only successfully committed formatted lines update remaining capacity;
automatic cuts are suppressed at the cap, while admitted manual endings
(including empty indications) remain permitted and consume capacity. A genuine
nonhyphenated ending resets it. Trials, floats and administrative run splits do
not themselves change it; nested block containers own separate state. Min-content
uses only the static zero/nonzero policy, independently of actual line history:
zero retains manual partitions without automatic refinement, positive counts and
no-limit retain refinement. Max-content stays unbroken. See the
[successive-line contract](../../../docs/text-layout-contract.md#successive-hyphenated-lines-and-intrinsic-measurement).


`NodeInputOf::hyphenate_limit_zone` supplies a signed length/percentage, default
zero. For each automatic candidate with a genuine ordinary fallback immediately
before its word, Layout converges that fallback with no supplied spacing through
the same shaping, allocation, float replay and height loops. Its own natural
alignment measure and actual indentation determine the unused inline gap;
percentages use its full float-constrained line-box span before indentation.
Only a gap strictly greater than the used zone permits automatic hyphenation.
Manual endings, including empty indications, remain independent. No legal
fallback at a line start or inside a continued word makes this gate inapplicable;
an existing fallback that fails finalization propagates its error. Min-content
is zone-exempt, while max-content stays unbroken. These reference trials remain
unpublished, and changed policy, bands or encounters rebuild their private pass
basis. See the [zone contract](../../../docs/text-layout-contract.md#hyphenation-zone-and-the-ordinary-fallback-line).

`NodeInputOf::text_group_align` selects `TextGroupAlign`, defaulting to `None`.
Layout groups actual descendant formatted lines in the same block formatting
context, resolves their common padding from natural advances, then applies
ordinary alignment and justification in that reduced content frame. A child
`None` still participates in its ancestor's group. The
[native group-alignment contract](../../../references/text-group-alignment--native-determination.md)
defines nested residuals, membership and per-line direction.

`NodeInputOf::block_flow_context` supplies `BlockFlowContext::Independent` for a
used flow root. `Normal` retains the box's other context-establishing facts.
Independent contexts contain their own floats; ordinary shared blocks transfer
float exclusions through the enclosing context without including those floats
in their own auto height.

`TextJustification::Spaces` expands retained U+0020 slots using Text's finalized
ownership and edge decisions. Layout distributes the remaining line capacity
after subtracting Text's natural alignment measure to those slots, then finalizes
the same selected line with the complete allocations. It reconciles an over-budget
result against Text's actual allocated alignment measure; if that correction
still exceeds the span, `TextSpacingBudget` reports the available span and actual
alignment advance. Text numeric refusals remain Text errors.
`None` adds no spacing. Native fitting and intrinsic queries consume the matching
Text-issued measurements; an absent finite width is not an intrinsic-purpose
tag. Intrinsic allocation/report width is separate from the actual Text metric
frame and source-fragment geometry retained for scrollable overflow. A supplied
block-end claim is checked against known later in-flow content in the captured
container; unknown authority is never upgraded from numeric coverage.
This explicit profile does not implement CSS-wide justification methods,
BreakSpaces or styled inline hanging barriers. Vertical and sideways Text
orientations remain explicit unsupported operations. A complete canonical
frame, line allocation, ordered spacing allocations, and float-band basis governs
same-invocation retries.
An unchanged selection reuses its exact finalized line, including rigid
translation; a repeated conflicting basis refuses rather than looping.
The common physical line anchor uses nearest-CSS-pixel rounding while retaining
fractional Text geometry. A real float transition or explicit clearance imposes
a minimum position; Layout selects the earliest permitted anchor on that same
pixel grid and retains the constraint during feedback retries. Completed batches
own the exact placed Text layout
and authentic fragment handles; consumers can pass that layout directly to
Render. Bound Text and dependent ancestor geometry caches are rejected across
invocations, while unrelated box caches retain their ordinary behavior.
Native scroll geometry includes every logical line allocation, including
terminal empty lines, alongside the independently placed Text metric bounds.
Source fragments and atomic boxes contribute their own actual geometry; glyph
ink and glyph advance do not replace line allocations. The
[shared placement contract](../../../docs/text-layout-contract.md#canonical-boxes-and-coherent-line-placement)
distinguishes logical progression, physical anchors and their float constraints.

The box/control-only numeric fitter uses Text's canonical `BidiLevel` and
borrowed [`SuppliedLine`](../../surgeist-text/src/supplied_line.rs) for ordering.
Supplied levels include 126; values `127..=255` refuse. It accepts actual atomic
box/control metrics, not caller-shaped text segments. See the
[shared ownership contract](../../../docs/text-layout-contract.md).

### Sizing

Sources: [sizing](../src/sizing.rs) and [values](../src/value.rs).

- Property domains: `PreferredSizeOf`, `MinSizeOf`, `MaxSizeOf`, and `FlexBasisOf`.
- Validated calculations: `SizingCalculationOf` and `CalcSizeCalculationOf`.
- Finite values and resolution: `LengthPercentageOf`, `PercentageBasisOf`, `LengthOf`, `LengthAutoOf`, and `AvailableOf`.

### Geometry and scroll

Sources: [geometry](../src/geometry.rs) and [scroll](../src/scroll/mod.rs).

- Physical geometry and flow mapping: `Point`, `Size`, `Edges`, and `FlowAxes`.
- Canonical scroll geometry: `ScrollGeometryOf`, `ScrollRectOf`, `PhysicalScrollRangeOf`, and `FlowRelativeScrollRangeOf`.
- Read-only scroll details: `OverflowClipOf`, `ScrollbarGutterRectsOf`, and `ScrollTargetGeometryOf`.

### Output

Source: [output](../src/output.rs).

- Root request and transaction: `LayoutRootRequestOf` and `CompletedLayoutBatchOf`.
- Node and sizing results: `NodeOutputOf`, `ComputeInputOf`, and `ComputeOutputOf`.
- Inline output: `InlineFragmentOutputOf`, `InlineFragmentOutputEntryOf`, and
  `PlacedTextOutput`.
- Root context: `LayoutRootContextOf`, `FlexItemRootContextOf`, and `ContainingLayoutContext`.

`InlineFragmentOutputEntryOf::owner()` distinguishes `InlineFragmentOwner::Node`
from `Scope(TextScopeBinding)`. Scope output preserves the complete issued scope
and container or boundary-pair association at each fragment's actual placed
geometry. It has no single donated node, extra strut or duplicated endpoint
geometry. The endpoints identify ownership; their coordinates do not position
the generated text.

Grid and grid-lanes use canonical scroll-box reservations for intrinsic
contributions and final layout. Subgrid edge contributions include the child's
margin, border, padding, and effective gutter once at each inherited boundary,
with the child's writing mode and its own settled scrollbar state. Inherited-axis
Auto reservations participate in the current parent's intrinsic contribution.
Nested preparation stops at the corresponding physical-axis sizing point,
including reversed axis order for orthogonal subgrids. Grid-lanes stacking
placement is evaluated when its actual stacking-size dependency is reached or a
qualified child-layout observation needs the stacking range; a grid-axis-only
prefix does not run otherwise unused stacking measurements. Minimum and maximum
contributions are measured separately against that point's track basis. A changed
individual minimum contribution permits the source-defined once-only axis
revisit; that revisit replaces obsolete basis-dependent contributions. The final
row stage does not reopen columns. Final placement uses the actual used box,
including saturation when a maximum size shrinks its gutters.
An intrinsic Auto observation runs actual child layout in a disposable computation session over
the same immutable inputs and invalidation basis; only its geometry observation
returns. Probe node, inline-fragment, warm-cache, and cache-publication staging
is discarded. Fully known size-only requests retain their measurement fast
path. Gutter rectangles describe layout geometry; painting and live scrolling
remain consumer responsibilities. The supplied finite scrollbar thickness is
an input, including zero; Layout does not select a host scrollbar policy.

### Finite grid utilities

Sources: [grid](../src/grid/mod.rs) and [track values](../src/value.rs).

- Lane placement: `place_lanes`, `LanePlacementInputOf`, and `LanePlacementReportOf`.
- Intrinsic lane sizing: `lane_intrinsic_sizing`, `LaneIntrinsicSizingInputOf`, and `LaneIntrinsicSizingReportOf`.
- Grid computation reports: `GridComputationOf` and `GridComputationReport`.
- Finite track construction: `TrackSizingOf`, `TrackComponentOf`, `TrackComponentListOf`, and `track_sizing_components_of`.

## Modeling contracts

These contracts are defined in [node input](../src/node_input/mod.rs),
[sizing](../src/sizing.rs), [values](../src/value.rs),
[measurement](../src/measurement.rs), and [output](../src/output.rs).

`surgeist-layout` exposes layout-ready contracts rather than authored CSS syntax.
`LengthPercentageOf<S>` is a normalized finite affine value: px plus a percentage
coefficient. Resolve it only against an explicit `PercentageBasisOf<S>`;
`PercentageBasisOf::definite` rejects invalid values at construction. Resolution
reports `MissingBasis` for a required missing basis and `InvalidNumeric` for a
non-finite evaluation; no value is guessed.

Preferred size, minimum size, maximum size, and flex basis are distinct closed
property domains. Their role-valid keywords cannot cross property boundaries.
Direct `FlexBasisOf::MIN_CONTENT` and `FlexBasisOf::MAX_CONTENT` values retain
their distinct intrinsic measurement constraints through the public layout
front door; neither is normalized to the generic content basis.
`SizingCalculationOf<S>` combines finite affine leaves with nested `min`, `max`,
and `clamp` in a validated program that is evaluated iteratively. Percentages
remain symbolic until layout receives an explicit basis, and a required missing
basis remains unresolved.

`NodeInputOf::flex_item_collapse` is a normalized, layout-ready flex effect, and
`FlexItemCollapse::Normal` is its default. A collapsed in-flow flex item
participates through a finite cross-size strut replay, publishes zero committed
collapsed geometry, and hides its descendants. Root `surgeist` owns
computed-style lowering from a flex item's `visibility: collapse` to this
normalized state, while rendering owns painting. This leaf does not parse
authored CSS or provide a general visibility model.

Canonical layout-ready `calc-size()` input pairs a property-specific basis with a
validated `CalcSizeCalculationOf<S>` containing finite absolute-pixel,
percentage, and size coefficients. Track flex is separate: construct a finite,
non-negative `TrackFlexFactorOf<S>` through `try_new` and place it only in a
maximum track breadth. A valid sizing behavior unsupported by an algorithm is
reported through `LayoutUnsupportedCapability::SizingBehavior` with the exact
property, behavior, algorithm, and axis instead of an automatic fallback.

`LayoutTree::layout_input` returns a concrete `LayoutInputViewOf` borrowed from
immutable request storage. Box facts and actual Text participation handles
are read in place; small break and boundary records are copied. Existing owned
`LayoutInputOf` constructors remain storage choices and expose `as_view` for
borrowing. A box provider derives `node_input` and the box view from the same
stored value. Non-box canonical facts and role diagnostics still cross the
existing validation front door.

The private computation retains that immutable source loan separately from
mutable output and cache staging, so recursive child computation can keep
borrowed projections live. Intrinsic Grid probes use local minimum-suppression
flags without changing or cloning source inputs. Completed geometry, fragments
and cache entries own their results and remain readable after the source is
dropped. These access guarantees do not claim zero allocation for layout's
working state or a measured speed improvement.

`LayoutRootRequestOf<S>` validates root input for the public
`compute_layout` front door. A successful call returns a
`CompletedLayoutBatchOf<Node, S>` containing the staged layout and cache updates;
a `LayoutErrorOf<Node, S, M>` returns no partial public result. Recursive
algorithm modes remain internal.

`compute_leaf` is the direct, fallible leaf-measurement boundary. Its provider
receives non-negative content-space constraints, and invalid provider output or a
provider error becomes a typed layout error. Resolved padding and border must be
non-negative and finite, including their combined axis sums. Overflow in
layout-owned box arithmetic returns `LayoutInvalidInputOf::InvalidNumeric`;
invalid provider dimensions retain the distinct `MeasurementOutput` diagnostic.
Valid fully known sizing requests can complete without calling the provider.

Visible inline boundaries and line breaks must match their containing block's
`FlowAxes`. A mismatch returns `LayoutInvalidInputOf::InlineFlowMismatch` with
the expected and actual axes, the container and control node, and the
`ChildLayout` operation. Hidden line breaks retain their existing bypass.

`ItemOrder` is the layout-ready signed order value. `SourceIndex` is stable
source-sibling identity: outputs remain source-associated while flex, ordinary
grid, and grid-lanes consume one stable order-modified traversal sorted by item
order and then source index.

`item_is_replaced` is an independent box-generation fact. It is not inferred
from table role, measurement, aspect ratio, or stretch. Block and root sizing
use it to avoid ordinary auto-inline fill, flex uses it when selecting automatic
main-size suggestions, and grid and grid-lanes use it when resolving normal
alignment while preserving explicit stretch.

Participating replaced boxes establish independent formatting contexts: float
placement preserves their resolved size, and their own block margins cannot
collapse through the box. External adjacent sibling margins still follow the
ordinary collapse rules. The host leaf provider supplies finite content
measurements selected from actual natural dimensions or the applicable fallback
when the host treats natural dimensions as absent. A missing measurement provider
does not mean absent natural dimensions; a genuine zero measurement is zero-sized
content, not absence.

`ContainingLayoutContext` keeps the containing flow axes and
`ParentFormattingContext` role together as the complete containing context and
cache identity. Flex-item roots require explicit parent flow axes and keep the
host allocation in the root request separate from the viewport percentage
context in `FlexItemRootContext`.

## Inline metrics

Source: [inline input](../src/node_input/inline.rs).

`InlineMetricsOf<S>` is layout-ready line box data. Layout consumes it for inline
line construction and does not derive it from authored CSS or fonts. Integration
layers should provide metrics from computed style and text/font measurement
before constructing `LineBreakInputOf<S>`.

## Browser parity runtime

The feature-gated `surgeist-layout-generate` binary supplies a private layout
adapter to the local `surgeist-generator` workspace dependency. The shared crate
owns acquisition, browser processes, input protection, reports, and atomic
publication. Layout owns helper preparation, measurement decoding, unsupported
classification, and XML serialization. Default layout builds omit this tooling.

The corpus pins its Chrome-for-Testing version, launch settings, and report
inventory in [the corpus manifest](../tests/layout/browser_parity/corpus.toml). `generate` may acquire the
managed browser; `generate-existing` requires `SURGEIST_BROWSER_PATH` to name a
repository-relative executable below the declared cache and never fetches.
Source checkouts live in `tmp/surgeist-sources/<id>/<revision>/`; browsers live in
`tmp/surgeist-browser/`. Repository `tmp/` is ignored and survives `cargo clean`.

`check-corpus` validates the generic schema-4 report and persisted source-import
attestation without a browser or source checkout. Explicit `check-taffy-corpus`
verifies the existing pinned source cache; `import-taffy` may acquire that source
and publishes verified imports plus attestation. Full generation validates
accounting before publishing. Filtered generation does not rewrite full reports.
XML bytes remain layout-owned and comment-free; semantic parsing and comparison
remain in layout tests. See the [browser parity guide](../tests/layout/browser_parity/README.md)
for commands, cache behavior, and current provenance requirements.

## Local commands

Run these recipes from the crate directory with `just` already available. The
[Justfile](../Justfile), [Cargo task script](../scripts/run-cargo-task.sh),
[verification script](../scripts/run-verification.sh), and
[browser-parity task script](../scripts/run-browser-parity-task.sh) define them.
Setting `CARGO_NET_OFFLINE=true` keeps Cargo dependency resolution offline.

| Recipe | Observable operation |
| --- | --- |
| `just fmt` / `just fmt-check` | Format Rust / check Rust formatting |
| `just check` / `just test` / `just clippy` | Default Cargo check, tests, or strict Clippy |
| `just verify` | Default formatting, check, test, and Clippy sequence |
| `just generator-check` / `just generator-test` / `just generator-clippy` | Corresponding all-targets checks with `layout-golden-generate` |
| `just verify-generator` | Generator-feature check, tests, and Clippy sequence |
| `just parity-all` | Execute the ignored full checked-in XML parity test, clearing the parity filter |
| `just corpus-check` | Browser-free `check-corpus` with corpus and browser overrides cleared |
| `just taffy-check` | Validate the existing pinned source cache and imported Taffy corpus |

Clippy recipes use `-F unsafe-code -D warnings`; Cargo check, test, and Clippy
recipes use `--locked`. Browser commands and environment variables are described
in the [browser parity guide](../tests/layout/browser_parity/README.md).

### Unfragmented element-terminal hyphenation

`NodeInputOf::hyphenate_limit_last` selects `HyphenateLimitLast::None` (the
initial unrestricted operation) or `Always`. For finite layout, `Always` retains
trailing empty line geometry and prohibits selected Source, Control or Automatic
hyphenation, including empty indications, on the immediate Soft predecessor of
the owning completed block's last content line. Real Forced segments remain
separate. Intrinsic queries build no virtual terminal tail.

The supported producer authenticates a complete actual inline-flow sequence,
including Text, in-flow atomics, controls and transparent out-of-flow tails.
Branch-local reselection uses the same finalizer and float/indent/count owners;
rejected sequences publish nothing. Numeric runs retain their actual ending and
content observations. A cross-run or child-block interruption without necessary
continuation custody returns `LayoutMissingContext::TextBlockTail` when selected
hyphenation makes that missing relation material. Child-owned lines are not
flattened into the parent, and ParagraphEnd alone is not block completion.
Column, page and spread fragmentation remain unavailable. See the
[canonical unfragmented-tail contract](../../../docs/text-layout-contract.md#unfragmented-element-tail-hyphenation-restriction).
