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
an unfragmented, unclamped profile.

`Stable` selects ordinary line-local quality preference in both Text-backed and
numeric runs, without Balance's residual-quality search. It retains its distinct
normalized mode in selection and replay. Actual source legality, scope priorities,
conditional stages, float/height convergence and terminal constraints still apply.
For example, `hyphenate_limit_last: Always` can require revisiting earlier choices;
Stable does not freeze an old line sequence or promise unchanged geometry after
every suffix edit. Supply coherent current preparation and use ordinary changed-node
invalidation when changing the mode. See the pinned
[Text 4 selection rule](../../../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#valdef-text-wrap-style-stable).
`AvoidShortLastLine` attempts a bounded repair of the final two formatted lines
in the final genuinely Paragraph-ending group. It preserves the ordinary line
count, terminal occurrence and earlier accepted prefix, including geometry and
state. A final formatted Forced line followed by a nonformatted Paragraph receipt
is not a target; earlier Forced groups remain ordinary, and content never moves
across Forced. This Paragraph-only target is Surgeist policy.

Quality uses natural advances before justification and each trial's actual
usable capacity, accounting for indent once. The ordinary last line must be
less full than its predecessor. A repair must strictly increase the last line's
fullness, the weaker final-pair fullness, and the last line's natural advance.
Narrowing a float band without lengthening the natural tail is insufficient.
There is no fixed shortness ratio, word-count cutoff or degradation budget.
The greatest improved minimum wins; equal improved scores retain the existing
deterministic incumbent. The finite search is a heuristic, not a global optimizer
or a promise to find every possible repair; it has no Balance ten-line limit.

Only finite positive capacities and natural advances between zero and capacity
participate in this optional comparison. Valid hanging, signed, overflowing or
otherwise out-of-domain ordinary results remain valid ordinary results. Quality
ineligibility cannot hide a real provider failure or create a geometry error.
Real zero-size atomic participants retain their formatted-line authority;
nonformatted completion and metric-only receipts do not become scored lines.
The same scope priorities, conditional authority, float/height feedback and
terminal constraints apply before quality. These choices implement the
[pinned Text 4 bias](../../../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#valdef-text-wrap-style-avoid-short-last-line)
through an explicit native policy.

`Pretty` processes hard groups in source order. At each actual entry it establishes
an ordinary result, applies the same Tail preference when the group's final
formatted occurrence genuinely ends Paragraph, then attempts body improvement.
Body preserves this entry's ordinary formatted count and terminal occurrence;
it can change the group's interior cuts. Earlier selected groups stay fixed.
Changed line heights can change the next group's entry and ordinary count, so
Pretty does not promise the original whole-Auto line count.

For at least two formatted lines, body quality is the mean of
`(1 - natural_advance / usable_capacity)^2` over all but the final formatted line.
Each candidate must lower that mean without decreasing terminal natural advance,
terminal fullness or minimum final-pair fullness relative to the fixed post-Tail
reference. Equal body scores retain the incumbent. Tail can worsen body quality;
Body starts from that result and does not undo it merely to recover Auto's score.
The finite natural-measurement domain above also applies to Body. Real zero-size
atomics count; nonformatted receipts do not become scored lines. Body can improve
Forced-ended groups, while Tail remains Paragraph-only. The deterministic search
has no ten-line eligibility cap and promises local improvement in actual context,
not a global optimum. These objectives and their ordering are explicit Surgeist
policy under [Text 4 Pretty](../../../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#valdef-text-wrap-style-pretty).

A mandatory terminal constraint can require revisiting a provisional earlier
group, as with `hyphenate-limit-last: Always` and a trailing Forced followed by a
nonformatted Paragraph receipt. If the streamed optional program cannot close
that constraint, Pretty discards the entire unpublished attempt and runs the
ordinary whole-sequence solver once from the same invocation entry. This loses
the attempt's quality improvements while retaining Pretty request identity,
all constraints, observed dependencies and consumed work. Candidate-only
ineligibility retains its valid phase incumbent. Provider, resource, cancellation,
dependency and replay errors retain their existing failure semantics; they do
not authorize successful fallback. Ordinary failure remains failure. A changed
dependency frame uses normal invalidation/convergence, never a restart from
previously optimized output.

#### Numeric inline baseline references

`NodeInputOf::inline_baseline_context` selects the numeric line's alignment
reference. `InlineBaselineContext::Physical` is the default raw coordinate
operation. `Named(BaselineName::Alphabetic)` and `Named(BaselineName::Central)`
request matching named baselines. The caller resolves the name from its
typographic context; writing mode alone does not choose it. A parent's request
does not change the child's own alignment context.

Named coordinates use the child's border-box physical origin on an explicit
axis. `BaselineCoordinateOf` accepts finite signed offsets, including positions
outside the border box. `NamedBaselinesOf` retains separate first/last facts
for each name: `Unknown`, genuinely `Absent`, or `Available`. Generic
`BaselinesOf` projections do not establish named authority. A missing generic
scalar likewise cannot prove named absence.

Measurement returns both generic and named coordinates relative to the returned
border box. An indefinite sizing pass retains its accepted physical anchor and
used frame until the final extent is known. Completing an end-side frame rebases
that anchor by the extent change without snapping it again; an already canonical
local anchor stays local. A child resolves its own frame before parent placement.
These measurement facts do not establish canonical Text placement authority.

The child's non-inherited `NodeInputOf::baseline_source` selects its matching
set. `BaselineSource::Auto` selects last for an inline-block and first otherwise;
`First` and `Last` select their respective sets explicitly. Only that set is
consulted. If the selected fact is genuinely absent, Layout synthesizes
alphabetic at the line-under margin edge and central midway between line-over
and line-under margin edges. Signed used margins remain part of that geometry.
Unknown authority refuses the request; a coordinate on an incompatible axis
is an input error. First/last output facts follow the actual eligible line
and retain their name and physical reference through box translation.
Unrounded facts refer to the unrounded border origin. Final packet facts refer
to the published border origin and preserve the once-snapped global baseline
anchor, including when nested local positions round independently. Top/bottom
content without a named baseline contribution establishes extents, while its
named anchor remains unknown.

Atomic [Top/Bottom alignment](../../../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#valdef-baseline-shift-top)
places its margin box against the line's physical line-over/under edge. Block
progression does not rename those edges. Signed margins locate the border from
that edge and can place it outside the line box.

`VerticalAlignOf<S>::Shift(BaselineOffsetOf<S>)` matches the actual child
baseline through the same path as `Baseline`, then applies a finite signed
displacement toward `FlowAxes::line_over`. Positive values move up in
horizontal-tb, right in vertical-rl/lr and sideways-rl, and left in numeric
sideways-lr. Local child baseline and extent facts remain unchanged. The shifted
bounds participate in line fitting, float feedback and committed placement.
An explicit `Shift(0)` retains its request identity and baseline geometry.
Generic callers use the `Of<S>` types; `VerticalAlign` and `BaselineOffset`
select the default scalar lane.

The offset helpers consume supplied used values. `from_length_percentage`
accepts `LengthPercentageOf` and the child's own `PercentageBasisOf` line-height;
a missing basis is valid only for a zero percentage coefficient. `subscript`
and `superscript` accept a `NonNegativeFiniteOf` parent font size and use the
native [Inline 3 fallback policy](../../../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md):
minus one fifth and plus one third, without resizing the child or querying
optional font offsets. Numeric zero contexts are valid. Resolution reports
missing context, nonfinite arithmetic or a lost nonzero contribution through
`BaselineOffsetResolutionError`; exact cancellation is valid.

These operations describe actual box/control allocations. They do not discover
CSS ancestry or execute captured text-scope glyph alignment. Native Text
allocations use already used line-coordinate units, with no second preparation
scale; see Text's [allocated shift contract](../../surgeist-text/docs/reference.md).
Its existing sideways-lr refusal remains in effect.

For a block scroll container with `baseline_source: Auto`, the last set places
both names at its own signed block-end margin edge. Layout then clamps exported
available coordinates to that container's border interval. Explicit `First` or
`Last` uses content-derived facts, with the same clamp; a synthesized fallback
is also clamped when consumed. `Clip` alone does not create a scroll container
or invoke these rules. Classification follows the adopted overflow-pair model.
Each scrolling descendant exports its facts before its parent translates them.
These coordinates use Layout's canonical initial-placement frame; current scroll
offsets are not Layout inputs. See the pinned
[baseline export rules](../../../references/css-align-3--WD-css-align-3-20260130--7361249cbb73.md#baseline-export)
and [source selection](../../../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#baseline-source).

`InlineMetricsOf` constructors require an explicit reference for their control
strut. Baseline and shifted controls must match the line's reference; top/bottom
alignment uses their real extents. Context, source and metric-reference changes use
the ordinary changed-node invalidation contract, and cached computation
outputs retain their named facts.
References also survive forced-line metric carry. A control's reference becomes
required when a real successor uses its strut as a baseline contribution.

When mandatory atomic participation or a native box edge carries a top/bottom
atomic's strut, Layout retains the actual source box. A real formatted receiver
resolves its requested name against that box's selected first/last fact and
signed margins before metric allocation, float-band fitting and balance replay.
If needed, it requests the fact on the source's original child-computation basis.
The original top/bottom line keeps its geometry; phantom completion requests no
unused name and establishes no named anchor. Explicit control struts still use
their declared reference, because they have no atomic box from which to resolve
another one.

Numeric atomic-box flow supports both names. Native upright or Mixed Text in
`VerticalRl` or `VerticalLr` additionally admits `Named(Central)` when the actual
prepared paragraph has matching Upright or Mixed analysis and Vertical orientation.
Its first/last formatted lines establish Central facts on the physical x axis;
absent or nonformatted lines do not acquire authority from the requested name.
HorizontalComposition Text admits `Named(Alphabetic)` when its actual preparation
observes `Some(FontBaseline::Alphabetic)` and its Horizontal or clockwise Sideways
orientation matches the consuming frame. This includes clockwise preparation in
`VerticalRl` and `VerticalLr`. Text owns each actual font/script's conversion to
the shared reference; Layout uses the common line anchor for matching and
first/last export, independently of displaced per-font paint origins. Raw
preparations cannot establish Alphabetic authority from their orientation alone.
`Physical` remains a separate operation and leaves named Text facts Unknown.
Other named Text contexts retain `MissingContext(NamedInlineBaseline)`.
Generic Flex/Grid projections and font-design zero coordinates do not establish
named facts or substitute for a possibly nonzero alphabetic baseline.
A Flex/Grid container can remain named-Unknown while its child's independently
qualified facts remain available in that child's packet. Positioning and floating
likewise preserve child-owned facts. Published cache outputs retain the same
distinction between Unknown and Absent, including size-only computations.
A sizing query may leave unneeded named facts Unknown even when a performed
layout derives them. An actual named alignment consumer requests its needed
name and first/last set before child computation; that demand is part of cache
identity. It does not change the child's own alignment context or grant an
unsupported producer named authority.

#### Prepared Text and inline flow

Prepared Text participation carries actual issued origin, atomic, control and scope
handles. Capture validates complete inventories against the same borrowed tree;
equal characters from another preparation cannot replace that basis. Layout
retains one `TextParagraphBinding` per complete analyzed output. Its `nodes`
inventory contains all participating nodes in order, including native windows
separated by actual child blocks. Text-issued source segments identify those
windows and metadata-only gaps; raw member seams create no native run. Layout
completes the segments into one placed output while traversing child blocks
independently. Independent preparations cannot split one uninterrupted native
run; admission returns `RunMembership` for that unsupported association.

Layout selects actual issued break opportunities and alternatives, then consumes Text's
finalized metrics, fragments, control slots, and atomic slots. At a discretionary
endpoint it tries alternatives in declaration order and uses the first that fits;
it keeps the first alternative when an indivisible candidate must overflow.
`TextControlParticipation` supplies the exact issued semantic control with no
allocated strut. Its direction, wrapping, and source-less alternative facts stay
Text-owned. Existing line breaks and inline boundaries instead supply their real
Layout strut metrics and must match the corresponding issued semantic role.
`LayoutFlowInput::try_new(tree, root, paragraphs, wrapping, scope_alignments)` also admits
box/control-only flow with no Text paragraphs. `WrappingInput` supplies actual
scope declarations and generated child-order proofs. `WrappingScopeInput::NativePair`
declares genuine normalized endpoints; `Captured` declares an authentic inline
capture identity. `WrapInside::Avoid` ranks permitted fitting breaks through the
same validated scope forest. A captured scope without endpoints supplies semantic
ownership and no physical border or padding authority. Ordinary `compute_layout`
keeps automatic wrapping without a declared scope context.

The `scope_alignments` vector supplies `TextScopeAlignmentInputOf<Tree::Scalar>`
for authentic captured inline scopes. An empty vector supplies no alignment
requests. Admission checks the actual capture association and its early formatting
classification. Length/percentage normalization uses the scope's retained own
line height; native subscript/superscript normalization uses the actual parent's
used font size. Layout fits the selected scope contributions and passes complete
allocations to Text, including explicit zero, then consumes the executed frame
once. See [captured-scope alignment](../../surgeist-text/docs/scope-alignment.md)
for early producer facts, edge-subtree behavior and transaction resource limits.

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
accepts the current tree, actual paragraph bindings, final `WrappingInput`,
normalized scope alignments, and a borrowed receipt. The tree also supplies a
fresh `WrapEdgeSource` projection.
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
A native pair's source enclosure alone does not prove that it owns the enclosing
captured box. A continuation without a selected same-box endpoint needs that
box's explicit witness. A zero strut remains a contribution and differs from
`NoContribution`.
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

Native non-atomic endpoints carry their actual border and padding.
`InlineBoundaryInputOf::contribution(kind, metrics, edge)` supplies the existing
endpoint operation without a full-box claim. `box_edge(kind, metrics, edge,
parent_axes)` requests actual box-side geometry with a separately resolved
immediate-parent basis. The existing writing-mode/direction fields remain the
containing Block formatter and retain `InlineFlowMismatch` validation;
`parent_edge_axes()` is present only for BoxEdge. Neither a child's direction nor
its parent's endpoint formatter fields establish the parent's own used axes.

Layout authenticates the actual captured scope/native pair and parent relation,
then lowers checked Text contexts or uses the genuine numeric family. Selected
BoxEdges surround their actual indivisible packet or authentic empty gap on the
assigned sides. Flattened native containment, equal source cuts and bidi parity
do not establish this ownership. Original levels and non-edge payload order stay
unchanged; context-only controls add no flow. A failed BoxEdge request refuses
without becoming a Contribution. Captured Contribution controls remain genuine
Contribution operations, including zero.

These endpoints do not represent signed inline margins. That remaining capability
belongs to [fragmented inline decoration work](https://github.com/bj-data-eng/surgeist/issues/780).
An atomic's signed margins cannot stand in for a non-atomic margin-only witness.

Text-backed scopes use authenticated mixed-prefix and canonical source incidence,
including transformed, collapsed and removed content. A semantic captured scope
can span a whole typographic unit without physical endpoints. Paired endpoints
at admitted decoded atom seams retain their genuine captured chronology,
including inside a unit shared by same-font connected origins. Their used
border/padding contributes once without separating the shared glyphs or granting
an interior line cut. Layout validates that order and canonical pairing;
unrepresentable source seams remain precise Text refusals.
`WrappingScopeError` identifies invalid or unprovable scope facts.
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

The native profile supports horizontal LTR and RTL text and clockwise
`WritingMode::SidewaysRl`. `VerticalRl` and `VerticalLr` also consume authentic
`HorizontalComposition` analysis prepared with `LineOrientation::SidewaysClockwise`.
Block progression follows the containing writing mode while glyph outlines
remain clockwise and use horizontal text metrics and bidi. Selection and
placement retain that actual
preparation and its receipts. Paragraph bidi direction and container line-box
direction remain independent. RTL line-box alignment does not reverse the
already ordered local glyph geometry again.

Both profiles support signed indentation and the preparation's selected
font-advance or CSS-stop tab operation.
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

`TextJustification::InterWord` is a distinct uniform native paragraph method. Text
supplies the complete separator inventory, intrinsic geometry and actual alignment
response; Layout supplies complete nonnegative allocations. Direct opportunities
receive equal initial shares and correction against Text's actual measurement.
When only tab-dependent opportunities can stretch, Layout probes the last admitted
one in Text's tab traversal using the same immutable natural shape, brackets the
actual response and refines until exact fill or adjacent representable amounts
prove the target is skipped. Refused probes propagate as Text errors; they never
prove unexpandability. Shared provider geometry or deferred terminal authority
needed for expansion is demanded through the real Text operation, not inferred
from a zero width or unavailable feedback.

An InterWord result must exactly fill the actual alignment capacity after indent
and group insets, or return its numeric/spacing-budget refusal. Known unexpandable
content uses the actual last-line alignment, with Justify falling back to Center,
through a fresh selection. Hanging-only allocation is not a successful fill.
Natural and intrinsic trials add no allocation; authored word spacing remains
Text-owned. Replays retain the actual method, complete amounts and Text shape
basis; changed line, box, role, tab or float context rebuilds the real selection.

`TextJustification::InterCharacter` requests Text's qualified visual character
boundaries through the same positive response solver. The actual effective
Justify context selects the spacing-capable Text basis before any natural or
residual measurement. Effective non-Justify contexts, including terminal Auto,
use an ordinary selection. Intrinsic and chosen-cut trials retain their actual
receiving context and add no allocation when they are natural measurements;
they do not borrow an ordinary ligature width for a spacing-capable request.
Text owns the [native character-spacing profile](../../../references/text-character-spacing--native-determination.md),
provider constraints, authored tracking, boundary cells and raw glyph geometry.

Qualified Direct boundaries receive equal initial full extras. Independent
character boundaries do not inherit InterWord's zero-intrinsic exclusion.
Protected required/cursive pairs are not allocated. Constrained or deferred
candidates stay in the complete table: an exact fill through qualified responses
may leave them at zero, but otherwise their material demand executes through
Text before fallback. Tab-dependent boundaries use the same actual response
bracket and representable termination as InterWord.

An ordinary InterCharacter fallback is a fresh selection and may change the
optional-feature plan. Layout remeasures that result for fitting, float bands,
groups, chosen cuts and tails. Conditional search retains matching-method query
authority: it uses an actually executed pre-fallback trial, or executes a real
ordinary trial with the prospective Soft method while preserving its issued
range, termination and actual end role. That trial authorizes queries only;
the actual fallback result remains the fit and placement output. No conditional
capability is transferred across methods or recreated from source offsets.
A conditional cut that itself needs ordinary fallback obtains a separately issued
ordinary-method endpoint: Layout executes that query's real ordinary trials,
phrase measurements and tier receipts through the same fitter. The endpoint
must match both the strong boundary and alternative. If an earlier ordinary
barrier or tier prevents issuance, that fallback attempt is inadmissible; the
primary tier retains its actually executed primary receipt and the chooser
continues without publishing an underfilled result.

An atomic's full exterior tracking value belongs to its actual Text `InlineBox`
before preparation, with resolved zero as the default. Layout consumes that bound
source occurrence. Neither child interior Text nor neighboring style supplies its
exterior tracking, and no later box allocation patches an issued preparation.
Atomic border/content rectangles and supplied flow allocations retain their
meaning; Text reports their actually shifted origins and boundary cell geometry.

`None` adds no spacing. Native fitting and intrinsic queries consume the matching
Text-issued measurements; an absent finite width is not an intrinsic-purpose
tag. Intrinsic allocation/report width is separate from the actual Text metric
frame and source-fragment geometry retained for scrollable overflow. A supplied
block-end claim is checked against known later in-flow content in the captured
container; unknown authority is never upgraded from numeric coverage.
This explicit profile does not implement CSS-wide justification methods,
BreakSpaces or styled inline hanging barriers. Native whole-upright paragraphs
compose in `VerticalRl` and `VerticalLr` through Text's
[qualified native profile](../../surgeist-text/docs/reference.md#native-whole-upright-paragraphs).
Native Mixed paragraphs compose in both vertical flows through Text's
[Mixed profile](../../surgeist-text/docs/reference.md#native-mixed-vertical-paragraphs),
retaining ordinary bidi, per-run outline axes and the common Central reference.
Actual Text-backed `SidewaysLr` reports
`UnsupportedCapability(TextWritingMode(SidewaysLr))` at `TextCapture` on the
container; box-only flow retains its own support. Clockwise Sideways Physical
composition admits in-flow and positioned atomic boxes with their actual
physical geometry and independently owned content orientation. Text glyphs
retain Clockwise. Explicit used Alphabetic preparation also supports
[`Named(Alphabetic)` matching](#numeric-inline-baseline-references); raw
preparation retains its named-reference refusal. Genuine
incoming native floats outside the paragraph binding remain Layout constraints.
Physical Text baseline
coordinates alone do not satisfy a named font-baseline request.

Canonical frames, line allocations, ordered spacing allocations and float bands
govern same-invocation retries.
An unchanged selection reuses its exact finalized line, including rigid
translation; a repeated conflicting basis refuses rather than looping.
The common physical line anchor uses nearest-CSS-pixel rounding while retaining
fractional Text geometry. A real float transition or explicit clearance imposes
a minimum logical block position; Layout selects the earliest permitted anchor on
that same pixel grid and retains the constraint during feedback retries. For
clockwise SidewaysRl and native VerticalRl this is a maximum physical x,
snapped downward when the nearest anchor would violate it; VerticalLr instead
requires a minimum physical x. Actual canonical container origin, used size
and all four content insets govern placement, including automatic block size.
Intrinsic sizing and final canonical placement are distinct: a final used frame
can supply a finite selection span after an intrinsic root sizing request.
Completed batches own the exact placed Text layout
and authentic fragment handles; consumers can pass that layout directly to
Render. Bound Text and dependent ancestor geometry caches are rejected across
invocations, while unrelated box caches retain their ordinary behavior.
Native scroll geometry includes every logical line allocation, including
terminal empty lines, alongside the independently placed Text metric bounds.
Source fragments and atomic boxes contribute their own actual geometry; glyph
ink and glyph advance do not replace line allocations. The
[shared placement contract](../../../docs/text-layout-contract.md#canonical-boxes-and-coherent-line-placement)
distinguishes logical progression, physical anchors and their float constraints.

Native upright and Mixed placement keep three coordinate mappings separate. Flow columns
progress leftward in VerticalRl and rightward in VerticalLr, while Text's
line-over to line-under cell direction decreases physical x in both. For actual
allocated line height `H` and finalized local Central baseline `B`, the anchor
is `B` from the right block-start in VerticalRl or `H-B` from the left in
VerticalLr. A finalized local point `(u,v)` maps to
`(CentralX+B-v, inlineOrigin+u)`. Normal-flow allocations follow the actual
block progression; source and metric rectangles retain their own extents.
Containing direction maps the available interval once, independently of the
upright paragraph's used LTR direction and retained computed direction.
Mixed retains Text's ordinary bidi order. Per-run outline axes remain separate
from these line and cell mappings.

Native upright in-flow atomics retain their physical child size, border, content
and overflow geometry. The adapter converts those boxes to Text's local
inline/line-under cells before finalization, then maps the accepted geometry
back once. A physical 6×14 child therefore uses a local 14×6 cell without
rotating its content. Signed margins and flow advance remain separate from
occupied bounds. The adapter carries explicit CSS margin-box fitting authority
from actual used atomic margins and size. Text returns a separate
[margin-edge admission observation](../../surgeist-text/docs/atomic-fit.md);
finite, intrinsic and conditional fitting compose actual signed indent with
that result once, then compare against the full preindent band while preserving
raw occupied and rendered geometry. Block intrinsic inline demand follows the
accepted line report; actual border/content/scroll extents remain independent.
Native opaque requirements retain their
own authority even beside a CSS margin box. Relative offsets place the physical box without being added
to its signed flow advance; changed canonical box edges still reenter fitting.

Named Central alignment uses the existing child first/last resolver and its
Absent synthesis, Unknown refusal and axis checks. Numeric participants retain
their distance from flow block-start; Text needs distance from line-over, so
VerticalLr converts that reference through the child's physical width.
Baseline, Top and Bottom keep their existing alignment roles. Out-of-flow
upright occurrences retain zero line-flow contribution. Their static anchors
use the actual flow block-start edge, which differs from line-over in
VerticalLr. The ordinary absolute-box operation supplies final physical
border/content rectangles and a physical x baseline; it does not align that
child to the parent's Central baseline. Nonformatted completion retains its
zero-height anchor. Offset-valued alignment remains separate.

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
- Named baseline facts: `NamedBaselinesOf`, `BaselinePairOf`, `BaselineFactOf`
  and `BaselineCoordinateOf`; see [baseline references](#numeric-inline-baseline-references).
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
before constructing `LineBreakInputOf<S>`. Its explicit reference follows the
[numeric baseline contract](#numeric-inline-baseline-references).

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
