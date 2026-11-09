# Reference

This page describes the current crate interfaces and local tooling. The
[manifest](../Cargo.toml), [public crate root](../src/lib.rs), and linked source
files own their current definitions.

## Package and features

| Item | Current value |
| --- | --- |
| Package / library | `surgeist-layout` / `surgeist_layout` |
| Version | `0.1.0` |
| Rust edition / minimum version | `2024` / `1.99.1` |
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

`CompletedLayoutBatchOf::node_layout` and `node_layouts` distinguish
`NodeLayoutRefOf::SingleBox`, `Fragments` and `NonBox`. A single box exposes
its unrounded and final `NodeOutputOf` and opaque `LayoutOutputEntryOf` custody.
Fragment collections expose typed actual-box or overflow-only payloads and coordinate parents;
non-box participants expose dedicated metadata geometry. Neither state supplies
a singular `NodeOutputOf` projection.

A warm host returns the original single-box unrounded entry through
`LayoutTree::unrounded_layout`; storing only its visual `NodeOutputOf` or returning
a final entry does not supply that custody. Layout checks the node, geometry
phase, existing compute/cache context and any required actual canonical atomic
frame before reuse. Unusable custody causes recomputation. Normal geometry and
readback phase remain private; hosts do not construct or recover them.

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
in the final Paragraph-ending group. It preserves the ordinary line
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
formatted occurrence ends Paragraph, then attempts body improvement.
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
for each name: `Unknown`, `Absent`, or `Available`. Generic
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
consulted. If the selected fact is absent, Layout synthesizes
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

#### Grid alignment baseline references

`NodeInputOf::grid_baseline_references` supplies resolved references for this
Grid container's local Column and Row alignment contexts. Both default to
`GridBaselineReference::Physical`. A `Named(BaselineName)` value is an explicit
normalized choice; writing mode does not choose a name. An inherited shared
subgrid context retains the actual owning axis's reference, including before
that context has any baseline target. A standalone context keeps its own input.

Grid chooses First or Last from its actual baseline-alignment preference. The
child's inline reference and `baseline_source` do not select Grid's set or name.
Available named facts retain their actual physical border-frame coordinate and
axis; a wrong axis is `LayoutInvalidInputOf::GridBaseline`. An unknown demanded
fact is `LayoutMissingContext::NamedGridBaseline`, not evidence of absence.
Absent facts allow Grid to synthesize alphabetic at the border line-under
edge and central at the border-box midpoint. Actual margins enter the owning
Grid contribution once, separately from this border point. Inline atomic
margin-edge synthesis remains a different operation.

The same selected source governs intrinsic contributions, performed census and
placement. First/Last selection, including Available Last0, remains distinct
from owner-relative role reversal. Actual subgrid maps transport the selected
point once; a reference change cannot reuse another context's target or cached
named demand. Existing scroll-container Last-set export and clamping remain
producer facts as specified above; they do not make Grid use inline set selection.

GridLanes applies the selected physical or named reference in its grid axis,
including inherited baseline groups. Stacking-axis baseline alignment values
use their fallback without forming sharing groups. An explicit named
stacking-axis operation returns
`LayoutUnsupportedCapability::NamedGridLanesBaseline` with the actual axis;
an unused selector does not reject layout. Container stacking baselines select
the first and last packed items of each occupied track, including dense backfill,
without reordering membership for negative margins or relative positioning.
Missing item-axis baselines are synthesized from the item's border edges in an
axis-compatible writing mode.
A Lane output without a named container fact remains Unknown.

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

Each binding explicitly chooses `inline_box_fragments` as
`InlineBoxFragmentationDemand::NotRequested` or `CompleteBodies`. The latter
requests one Text-owned inline-box family for the complete prepared output,
using the admitted captured boxes and complete extra native-pair inventory.
It requires Complete captured topology and actual complete body coverage;
Text's precise capability errors remain Text errors at `TextCapture`.
Association-only metadata refuses `InlineBoxFragmentsRequireFlow` in both
checked constructors. Computed-only semantic flow can carry the same
explicit demand. The early wrap-edge receipt is still validated before final
admission in `try_new_with_edges`.

The family is shared through fitting, issued flow/conditional observations,
retries and whole-output placement. `PlacedTextOutput::inline_box_fragments()`
returns `None` for NotRequested and a `PlacedInlineBoxFragments` view for a
demanded result, including a result with no placed pieces. Its `family()`
retains the exact checked preparation and its `pieces()` borrows Text's actual
placed pieces; Layout does not project another fragment geometry. This exposes
fragment identities and existing geometry, not complete decoration painting or
NCA-controlled formatting.

NotRequested allocates no family-slot owner and adds no family preflight.
Requested families share one optional binding-indexed owner; when captured
Scope limits apply, its actual header/backing and Text's real retained family
roots participate in admission and exact related-product census. These limits
already fund the successful preparation; family construction does not introduce
a separate funding mode. With no applicable ceiling, Native allocations remain
fallible without acquiring a new global quota.

The resource owner admits completed standalone-census work and new record charges
on success and failure. Borrowed Text roots and live Layout leases remain separate
from cumulative materialization. If completed costs exceed a cumulative ceiling,
the owner retains that exhaustion for later admissions; a failed census still
propagates its original cause. No output reservation or successful union authority
is created from a failed census.

Numeric formatters request a Layout-owned family through
`WrappingInput::native_inline_boxes`. Both checked constructors authenticate its
complete generated-child proof and actual native ordering roles.
`CompletedLayoutBatch::native_inline_box_families()` retains formatter-wide box,
body, fragment and ancestry identities across wrapping;
`native_inline_box_pieces()` exposes actual selected membership and executed edge
associations. A demanded empty result retains its family with no pieces;
unrequested formatters have no family. These numeric observations preserve native
custody without issuing Text's `InlineFragmentOutput`, decoration envelopes,
paint rectangles or NCA-controlled formatting. Actual scheduled Prepared Text
execution refuses native-family demand and uses the Prepared capability above.

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
declares normalized endpoints; `Captured` declares an authentic inline
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

Numeric inline forces use participant cuts, including leading/terminal
mandatory progress, and partition maximum-content groups. Propagation consumes
actual first/last in-flow child evidence; `InlineChildOrderInput` distinguishes
the paired box from its endpoints and declares prefix/suffix completeness.
Missing demanded incidence returns `MissingChildOrder`. Mixed inside/edge
avoidance uses the [independent-constraint policy](../../../references/text-wrap-edges--native-determination.md#independent-avoidance-constraints).
Avoidance adds no ordinary opportunities.

Text-backed soft edge avoidance selects one Text-issued original incidence of
the existing boundary, with authentic generated-box order and association.
Empty captured boxes can have distinct before/after choices; one choice
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

Complete generated child order can include inline scopes without physical
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
exits precede that paragraph's exit. After an actual Forced selection,
Text's issued ParagraphEnd selection can complete remaining boxes within the
same actual scheduled paragraph. Layout retains this proof separately
from the preceding original pin, which remains unchanged. The proof carries the
selected sequence, incoming box attribution and captured paragraph horizon
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
in-flow atomic, each individual nonzero supported inline margin, border or padding, and
an actual Forced ending remain formatted even with zero extent. A terminal
strut-only receiver does not inherit the preceding Forced status. These native
results use the numeric owner directly and do not manufacture Text receipts.

Captured paragraphs whose only prepared participants are computed
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

Native non-atomic endpoints carry finite signed margin and finite nonnegative
border/padding. `InlineBoundaryEdgeOf::try_new(margin, border, padding)` keeps the
components separate. Ordinary border/padding addition retains native scalar
rounding; `border_padding_advance()` and checked signed `flow_advance()` are
different observations.
`InlineBoundaryInputOf::contribution(kind, metrics, edge)` supplies the existing
endpoint operation without a full-box claim and refuses nonzero margin.
`box_edge(kind, metrics, edge,
parent_axes)` requests actual box-side geometry with a separately resolved
immediate-parent basis. The existing writing-mode/direction fields remain the
containing Block formatter and retain `InlineFlowMismatch` validation;
`parent_edge_axes()` is present only for BoxEdge. Neither a child's direction nor
its parent's endpoint formatter fields establish the parent's own used axes.

Layout authenticates the actual captured scope/native pair and parent relation,
then lowers checked Text contexts or uses the numeric occurrence proof.
Selected BoxEdges surround their actual indivisible packet or authentic empty gap on the
assigned sides. Flattened native containment, equal source cuts and bidi parity
do not establish this ownership. Original levels and non-edge payload order stay
unchanged; context-only controls add no flow. A failed BoxEdge request refuses
without becoming a Contribution. Captured Contribution controls remain
Contribution operations, including zero.

The numeric formatter excludes actual BoxEdge operations from content L2 before
ordering and retains their physical allocation. Ordinary Contributions remain
supplied operands, including zero. A demanded and proven normal family can
classify its zero-component undecorated endpoints as transparent logical
aliases while retaining their Contribution operation and supplied metrics.
`WrappingInput::gaps` carries resolved
neighbor/formatter/delimiter/empty-parent attachments when actual facts cannot
determine an ordering-empty box's gap uniquely. Attachments must remain valid for
the actual selected cut; a body-valid neighbor on another line supplies no gap.
See the [numeric ordering contract](../../../docs/text-layout-contract.md#numeric-physical-edge-ordering)
for anchor sides, physical tie order and precise refusals.

The strip footprint stays nonnegative even when margin makes flow negative or
zero. On a formatted line, its zero-block-extent node sits at the actual control
baseline; supplied metrics independently determine line height. Node
location/size/border/padding describe that strip; `NodeOutputOf::margin`
retains actual physical margins. Individual nonzero components establish
participation and shaping separation, so opposing margins or cancellation with
border/padding do not turn an empty pair into a phantom line. Actual
strut/baseline evidence remains independent.

Text's lane-specific component receipt preserves the used native arithmetic
before widening. Layout consumes its separate directed-flow and strip readback
and the [margin-edge fit observation](../../surgeist-text/docs/atomic-fit.md),
retaining independent Native opaque geometry. Raw overflow does not replace that
fit demand. Complete fragmentation/NCA formatting and decoration painting remain
with [fragmented inline decoration work](https://github.com/bj-data-eng/surgeist/issues/780).

Text-backed scopes use authenticated mixed-prefix and canonical source incidence,
including transformed, collapsed and removed content. A semantic captured scope
can span a whole typographic unit without physical endpoints. Paired endpoints
at admitted decoded atom seams retain their captured chronology,
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
canonical Text scope from its captured endpoint controls and the actual
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

A captured paragraph with no original, control, atomic, participant or
generated-contributor inventory can associate an empty domain through the existing
paragraph binding. Its supplied node and binding inventories must also be empty,
and `line_strut` must be absent. This association adds no Text run, line, strut,
fragment or placed Text output; ordinary Box geometry is unchanged. Removed-only
source and omitted real bindings do not qualify. Empty incidence must be authentic
and unambiguous, and an empty paragraph does not certify that its whole domain is
empty. Unknown inline topology does not erase canonical paragraph/domain
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
the first line of an actual block container and lines after forced
breaks; `hanging` inverts that predicate. Only accepted lines advance this state,
including empty lines. Candidate retries and clearance do not consume it.
Text-bearing and box/control-only inline runs use the same policy.

Prepared Text hanging punctuation uses explicit First/Last formatted-container
roles. Layout supplies actual First state and validates demanded Last assumptions
against accepted Text, numeric, empty and Forced lines, including later
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
Separate paragraph bindings retain their terminal receipts. A
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
(including empty indications) remain permitted and consume capacity. A
nonhyphenated ending resets it. Trials, floats and administrative run splits do
not themselves change it; nested block containers own separate state. Min-content
uses only the static zero/nonzero policy, independently of actual line history:
zero retains manual partitions without automatic refinement, positive counts and
no-limit retain refinement. Max-content stays unbroken. See the
[successive-line contract](../../../docs/text-layout-contract.md#successive-hyphenated-lines-and-intrinsic-measurement).


`NodeInputOf::hyphenate_limit_zone` supplies a signed length/percentage, default
zero. For each automatic candidate with an ordinary fallback immediately
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

Leading empty blocks can share their top and bottom margins with the parent's
start margin and later adjoining sibling margins. Layout retains the positive
and negative extrema until that group is consumed or exported. An exported
group stays outside the parent's content and adds no local offset. Borders,
padding, sizing, line participation and context boundaries determine adjacency.

An ordinary float between adjoining margins starts from the border position of
an [otherwise empty anonymous block parent](../../../references/css2--visuren.html--3f334c530cf4.md#float-rules).
Placement applies float constraints and retains the normal-flow cursor and
adjoining group.

An `item_is_table` node represents the normalized outer table wrapper. Its
contents establish an independent block flow with `BlockFlowContext::Normal`;
its outer border participates in the parent's float avoidance. Supplied
measurements or normalized child boxes determine the outer geometry. The marker
does not dispatch table-grid layout.

Float placement and independent block sizing and collision use float margin
boxes, including shape-backed floats. Shape intervals apply to inline wrapping;
see the [float input contract](../src/lib.rs).

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
preparation retains its named-reference refusal.
incoming native floats outside the paragraph binding remain Layout constraints.
Physical Text baseline
coordinates alone do not satisfy a named font-baseline request.

Separate actual normal and visual canonical frames, line allocations, ordered
spacing allocations and float bands govern same-invocation retries. Normal
frames retain normal border and content-box facts, including used padding and
gutters, before relative visual offsets. Fitting, normal line height and exported
operational baselines use that frame. Visual NodeOutput and placed Text geometry
use the actual visual frame. Ordinary rounding retains both ancestry channels;
a canonical Text atomic retains its measured and canonical normal geometry and
its visual canonical box without snapping either canonical box again. Named
and scroll readback finalize once before the complete final entry is installed.
Positioned Text occurrences retain both actual static ports from the same
accepted selection; normal placement facts use the normal port, while physical
publication uses its visual port.
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
own authority even beside a CSS margin box. The adapter supplies
`AtomicFlowAllocation::normal_border_inline_size` from the actual normal
canonical border extent in Text's local inline axis. CSS fitting uses that fact
with the normal advance and occupied bounds; the separate visual box may have
a different pixel extent. Relative offsets and relative ancestry affect actual
visual placement without changing normal flow, fitting, line height or exported
normal baselines through pixel phase. Changed normal canonical facts still
participate in the existing feedback loop.

The coupled entry is a by-value private state on the existing output owner.
Its largest geometry state determines every entry's storage cost, including
ordinary rows. Unrounded/final arrays, retained frames, cache restoration and
float packets carry the actual types and capacities; they do not maintain a
second atomic-origin authority. Existing Native geometry storage keeps its
Native resource responsibility. Feature-owned Text buffers and Layout scope
copies retain their actual
[scope resource accounting](../../surgeist-text/docs/scope-alignment.md#bound-each-transaction).

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
- Finite values and resolution: `LengthPercentageOf`, `PercentageBasisOf`, `LengthOf`, and `LengthAutoOf`.
- Sizing requests: `AvailableSpaceOf`, `SizingOperation`, `AxisSizingRequestOf`, `SizingRequestOf`, and `OrthogonalFallbackOf`.

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
prefix does not run otherwise unused stacking measurements. Minimum, min-content
and max-content contributions are measured separately against that point's track
basis. The minimum uses the item's actual minimum-size frame; min-content and
max-content retain their distinct intrinsic measurement purposes. A changed
individual min-content contribution permits the source-defined once-only axis
revisit; that revisit replaces obsolete basis-dependent contributions. The final
row stage does not reopen columns. Final placement uses the actual used box,
including saturation when a maximum size shrinks its gutters.
Subgrid gaps use actual own-content definiteness: cyclic percentages resolve
against zero for intrinsic contributions while retaining fixed calculation terms.
Normal content layout uses the accepted child box. Grid-lanes Auto preparation
keeps issued opposite availability when its common stage and source qualify it.
An intrinsic Auto observation runs actual child layout in a disposable computation session over
the same immutable inputs and invalidation basis; only its geometry observation
returns. Probe node, inline-fragment, warm-cache, and cache-publication staging
is discarded. Fully known size-only requests retain their measurement fast
path. Gutter rectangles describe layout geometry; painting and live scrolling
remain consumer responsibilities. The supplied finite scrollbar thickness is
an input, including zero; Layout does not select a host scrollbar policy.

### Finite grid utilities

Sources: [grid](../src/grid/mod.rs) and [track values](../src/value.rs).

A subgrid in a Grid Lanes parent adopts the parent's physical grid/stacking
axis pair when either child axis requests subgridding. Its grid axis shares
parent tracks when requested; its stacking axis has no inherited track vector.
Adoption overrides the child's selected pair and gives an ordinary Grid child
lanes geometry, while preserving the child's reverse progression and density.
Writing-mode mapping determines the corresponding physical axes. In an ordinary
Grid parent, a Grid Lanes child retains its selected pair and shares tracks only
on its grid axis. Its stacking-axis template is ignored.
Registered host-measured leaves retain their measurement boundary; their grid
templates do not establish a nested formatter or shared tracks.

Stacking content alignment translates the packed first/last outer-edge range
as one subject. Item alignment uses frozen adjacent cells: ordinary successors
provide no extra space, a following spanner retains its fixed gutter, and
terminal cells end at the endmost terminal outer edge. Auto margins and Stretch
use that cell while percentages retain the full containing-block basis.
Local alignment leaves other cells and the common content translation fixed.

Positioned children resolve stacking line `1`/`-2` at the translated packed
range start and line `2`/`-1` at its end. This positioned-only namespace does
not materialize a stacking track. Grid-axis references use ordinary grid-line
qualification and gutter edges. A reversed signed range retains the selected
start anchor and floors the containing extent at zero.

Auto lines use the canonical auto-edge containing area. When both insets on an
axis are auto, align the margin box within the container's content-box static
rectangle, with auto margins resolved to zero. Absolute children leave track demand, packed ranges
and other absolute children's frozen bounds unchanged; their overflow still
contributes to final scroll geometry.

- Sparse lane placement uses `place_lanes`, `LanePlacementInputOf`, and
  `LanePlacementReportOf`, with an explicit grid axis and reverse traversal.
  Container layout consumes `NodeInputOf::grid_lanes_flow` for the resolved axis
  pair, progression and dense packing.
- Intrinsic lane sizing: `lane_intrinsic_sizing`, `LaneIntrinsicSizingInputOf`, and `LaneIntrinsicSizingReportOf`.
- Grid computation reports: `GridComputationOf` and `GridComputationReport`.
- Finite track construction: `TrackSizingOf`, `TrackComponentOf`, `TrackComponentListOf`, and `track_sizing_components_of`.

### Normalized fragmentation scheduling

Source: [fragmentation](../src/fragmentation/mod.rs).

`resolve_fragmentation` computes a complete schedule from caller-owned source
occurrences, fragmentation contexts and a `FragmentationProvider`. The provider
supplies fragmentainer constraints and measures the remaining source under the
current frame. Opportunities must describe that frame; measurements made at a
different inline size cannot stand in for them.

`FragmentationKind::Page { progression }` carries `PageProgression` separately
from the context's geometric `flow_axes`. Its
[classifier](../src/fragmentation/page.rs) consumes the document principal used
axes supplied after upstream root/body propagation. Horizontal-tb follows the
used direction; vertical-rl and sideways-rl progress right-to-left, and
vertical-lr and sideways-lr progress left-to-right. A fragmented subtree's axes
do not substitute for this document fact.

Frame providers supply physical page side when known. Recto and verso use the
nearest Page context's progression, while physical Left/Right controls retain
their physical meaning. Child Column/Region axes and side annotations do not
override that Page; page-less contexts ignore page controls. The provider owns
the actual starting side and frame inventory. Unknown side is allowed until a
side force requires it, when `MissingPageSide` identifies the affected frame.

`FragmentationReplyOf` distinguishes raw controls at the source cursor from a
measured window. A window ends at the original flow end or a potential control
boundary and reports only that covered span, with an explicit measured head
endpoint. This permits disclosure of a forced boundary before measuring its
successor in the wrong frame. Required provider errors still propagate.
The measured head can include mode-certified trailing zero-source closures;
its body includes their retained closing decoration. These attachments remain
visible to legal cuts and forced-break controls.

The scheduler resolves control applicability before accepting a provisional
span. An ignored or replayed candidate advances only `scan_after`, requesting a
replacement measurement from the same original cursor and formatting inputs.
That replacement supplies current geometry, line counts and legal opportunities;
obsolete ordinary points are discarded. Established control candidates retain
their source identity and complete controls. A newly applicable force behind the
scan bound is invalid. Raw controls at the cursor do not require a synthetic
measured opportunity at the fragmentainer top.

Query and piece `start_provenance` distinguish flow entry, progress within a
frame and the actual preceding break reason. Discovery retries preserve it;
accepted progress updates it. All flows finish at their original
source end or return an error without partial output.

Each opportunity carries a `FragmentationCutOf`: `Source` identifies an exact
source boundary; `Graphical` identifies strict interior progress within an
anchored occurrence. Graphical Class C cuts use the same avoidance rules and
selection stages as source cuts. Their progress does not create source units.

The scheduler owns applicable forced breaks, exact latest-fitting selection,
widow/orphan then avoidance relaxation, cloned block-end then block-start
truncation stages, and final slicing or one-time unsliceable overflow. Parallel
flows advance independently. Source occurrence order is separate from geometric
position, so zero-sized content can advance source coverage without a positive
geometric extent.

The report retains accepted occurrence coverage, frame and context relations,
and selection stages. Source positions use exact integer units independently of
layout scalar precision. A graphical piece records its anchor and graphical
coverage. Other covered source is consumed once; only the anchored occurrence
repeats while graphical progress remains incomplete. A nonfinal cut
leaves the semantic cursor at its anchor. Caller keys identify input occurrences;
they do not authenticate a native tree, Text boundary or generated box. Failure
returns no partial report. The normalized report itself does not publish node
geometry or cache effects; the native Block adapter supplies that integration.

### Native Block fragmentation and publication

Sources: [Block adapter](../src/block_fragmentation/mod.rs),
[completed batches](../src/output.rs) and [sink contract](../src/tree.rs).

Construct `LayoutBoundRequestOf::block_fragmentation` with a source basis, a
standalone root request, fragmentation kind, flow axes and ordered physical
`BlockFragmentainerOf` rectangles. Capture it with
`LayoutFlowInput::try_new_bound` and compute through `compute_layout_with_flow`.
Each frame invokes the Block and leaf producers at
its own inline size. Rejected trials are discarded; accepted source intervals
are recomputed. Frame capacity and used fragment fill do not supply a definite
percentage-parent basis; required missing bases retain the producer's typed
error. Ordinary auto Block ancestors retain their actual continuation geometry.
Atomic boxes with no internal breakpoints overflow once when they cannot fit a
fresh frame.

The bound Page kind retains its progression through root and child scopes.
Completed frame metadata derives its physical/logical page-side pair through
the same classifier; nonpage frames have no logical page classification.
Missing required side authority returns
`BlockFragmentationFailure::MissingPageSide` with its frame index, without a
completed partial batch or cache/publication effects. Invalid geometry and
unavailable fragmentainers remain separate errors.

Break-margin trimming uses contributor groups recorded by Block's collapse
operations. A descendant margin separated from the break by padding, border or
an independent formatting context remains internal. Unforced breaks trim
adjoining margins on both sides; forced breaks preserve the post-break group.

The box-only profile supports ordinary auto Block containers and atomic empty,
measured, replaced or orthogonal boxes. Source participants require box inputs.
Outside an enclosing atomic unit, boxes use Block layout, `Position::Relative`
and no floats; non-atomic containers require an auto preferred block size.
Preferred inline `MinContent` and `MaxContent` retain content contributions from
all source children under canonical containing contexts. Each actual fragment
still resolves its own insets and clamps and performs Normal layout at that
width. Whole-source component preparation and actual measured-leaf intrinsic
queries use the first fragmentainer as ICB authority, separately from the
original percentage-parent basis. Leaf purpose is selected after authored
properties and known sizes are lowered; the actual query retains its nearest
scrollport and local containing limits. Normal leaf queries retain their
original fallback context. Intrinsic preparation does not perform the Normal
flow of a future source window. An intrinsic demand
crossing an unmeasured atomic container into an unsupported physical axis
returns the profile error; opaque measured leaves retain their physical
axis request. Size containment obtains intrinsic sizing from the existing empty-box
producer; descendants retain their actual layout. Eligible both-axis size-containment
boxes form one monolithic unit. Internal break controls do not split that unit;
outer controls and whole-box deferral remain effective. Inline-only containment
retains normal block-axis continuation and resolves its inline size in each
accepted fragmentainer.
Descendant box layout inside an atomic unit uses the ordinary producers without
splitting that unit. Nodes hidden by `Display::None` publish non-box metadata.
The standalone root requires matching flow axes, auto preferred block size,
zero margins and auto insets.

Supported absolute block minima retain one content target across fragments,
including `calc-size(any, …)` calculations without percentage dependence.
Each fragment's content extent, including fragmentation fill, counts toward that
target. A remaining minimum can produce further fragments after the last child;
those continuations retain the source node and continuation chain with empty
`source_children`.
Border-box conversion uses the original full padding and border. Each fragment
counts content after removing only its retained slice insets. Continuing
border-box minima require block insets independent of percentage bases.

Absolute block maxima bound principal content progress, including fragmentation
fill. A natural terminal fragment does not grow to consume an unused maximum.
The original minimum takes precedence over a smaller maximum. Border-box maxima
use the original full padding and border; continuing border-box maxima require
block insets independent of percentage bases.

After the principal box reaches its maximum, visible descendant content retains
a separate fragmentation schedule. Its occupied extent and later forced breaks
do not advance following siblings beyond the principal endpoint. Original Slice
opening and closing edges remain principal-box obligations, including pending
closing padding and border. A parent placement change discards the affected
child trial before recomputation.

A continuing non-atomic container with a context-dependent block bound,
an aspect ratio, non-visible overflow, opposite block progression on
the same physical axis or unsupported intrinsic inline sizing returns
`LayoutUnsupportedCapability::BlockFragmentationProfile`. A box with unsupported
constraints also refuses if its used extent exceeds the available extent, even
after its last child.
A whole container that fits an accepted frame retains ordinary sizing.
Fragmentainer exhaustion and invalid native geometry return typed errors without
a partial batch. This adapter does not provide column geometry or fragmented
Text composition.

`LayoutFragmentId` and fragmentainer handles belong to one completed result,
independently of the source/constraint target. `fragment` resolves singular and
multiple fragment occurrences and, like `fragmentainer`, rejects handles from
another result. `FragmentCoordinateSpace` names a fragment, an external frame or
root space. A continuous shared source node can lack a unique coordinate parent;
external frames are absent in continuous layout. `child_fragments` resolves
actual child occurrences from canonical parent links, independently of source-child
associations. The batch retains continuation links and paired unrounded/final
geometry. `FragmentPayloadOf::ActualBox` exposes principal geometry through
`principal_box`; `OverflowOnly` exposes only origin and content bounds through
`overflow_only`, with no principal edges, baselines or scroll box. An actual
zero-sized box remains an actual-box payload. Principal and overflow continuations
share the same result-scoped identity and node fragment collection. Native Block
retains overflow-only ancestor carriers when later descendant fragments require
coordinate ancestry after the principal box has ended. Their origins and content
bounds use the actual frame and containing context. Child source progress can
require another actual principal fragment without increasing the enclosing
principal endpoint.

Relative visual insets consume only their selected axis. An actual principal
supplies its measured containing size to children accepted in that frame,
including zero block size and content beyond its maximum. Without an actual
principal, accepted Block inline extents supply inline percentage bases. Implicit
ancestor carriers use the target frame's normal inline placement and containing
inline sizes, qualified by adoption of their descendant content. Relative
displacement reaches descendants once through coordinate ancestry and does not
change normal flow or cap consumption.

A selected block percentage without an accepted containing-size basis returns
`MissingContext(RequiredBasis)` at value resolution, identifying the affected
child, containing owner, frame and axis. Absolute offsets, zero coefficients,
ignored opposing percentages and independently measured inner containing boxes
retain their ordinary behavior. The unavailable overflow-only block-axis basis
remains tracked in [#1010](https://github.com/bj-data-eng/surgeist/issues/1010).

`LayoutSourceBasis` is the host's promise that captured inputs, topology and all
consulted measurement/binding environments remain unchanged. It does not inspect
or retain that source. A new bound request receives a separate constraint
identity; cloning it retains that identity. Before computation, install its
`target_basis()` in the sink's `LayoutPublicationTargets` for the root. This
updates the intended target independently of the geometry currently visible.

`apply_to` checks the installed target, prepares an owned complete replacement,
rechecks the target and commits under exclusive publication ownership. The
call-scoped `LayoutBatchPreparation` and `LayoutBatchCommit` carriers cannot be
retained as replay authority. Missing or stale targets, overlapping tracked root
ownership and preparation failures leave published geometry unchanged. The sink
must retain its registry with visible state and replace prior root members,
including detached nodes, in the same commit as geometry, Text and cache effects.
Do not reset the registry while its published roots survive or change publication
through another access path between the final check and commit.

Bound computation bypasses host cache reads and retained layout custody, including
probe and intrinsic sessions, and emits no Layout cache stores. Whole-root
replacement clears performed-layout cache entries while retaining independent
measurements for unchanged source. Explicit invalidation and
`cache_clear_entries()` still clear measurement entries as well, including in
bound batches. `LayoutBoundRequestOf::continuous` uses the same checked protocol
to replace fragmented output with continuous geometry. Unbound computation cannot
overwrite a tracked root.

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

Each `AxisSizingRequestOf` carries physical `AvailableSpaceOf` independently
from its `SizingOperation`: Normal, MinContent or MaxContent. Finite space does
not turn an intrinsic contribution into normal layout. Imposed sizes,
percentage-parent bases and the containing flow axes remain separate inputs.
`SizingRequestOf` also retains qualified initial-containing-block, nearest
scrollport and local containing-block limits for orthogonal fallback. Unknown
authority is distinct from an authenticated unbounded limit; entering a nearer
scrollport replaces the previous scrollport candidate.

Flex separates intrinsic contributions from Normal contents sizing. After
accepting an unrounded inline content extent, it resolves contents percentages
against that extent and finishes an auto block size from actual child
measurements. Definite or imposed block sizes retain their own authority.
Contents cannot enlarge the accepted
[cyclic inline extent](../../../references/css-sizing-3--WD-css-sizing-3-20260904--ca0f1db3d5ad.md#cyclic-percentage-contribution).
Normal size queries use the same sizing rule without staging geometry;
MinContent and MaxContent requests remain intrinsic operations.

Cyclic Flex gap percentages
[resolve against zero](../../../references/css-align-3--WD-css-align-3-20260130--7361249cbb73.md#gap-percent)
in every phase, preserving a calculation's fixed term. A later contents size
does not replace the gap's original qualified basis.

Standalone roots explicitly declare their containing axes and percentage-parent
bases. Flex-item roots additionally declare the actual host allocation. The
bounded BlockFlow and PositionedAutoInsets root contexts carry their own
admission conditions; source/capture adapters establish those facts before
constructing a request. Measured leaf providers receive `ContentSizingRequestOf`
after Layout's property and box-edge lowering, together with the node's flow
axes and actual containing context.

`compute_leaf` is the direct, fallible leaf-measurement boundary. Its provider
receives non-negative content-space constraints, and invalid provider output or a
provider error becomes a typed layout error. Resolved padding and border must be
non-negative and finite, including their combined axis sums. Overflow in
layout-owned box arithmetic returns `LayoutInvalidInputOf::InvalidNumeric`;
invalid provider dimensions retain the distinct `MeasurementOutput` diagnostic.
Valid fully known size-only requests can complete without calling the provider.
Performed leaf layout still measures the actual content and scroll facts.

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
does not mean absent natural dimensions; a zero measurement is zero-sized
content, not absence.

`ContainingLayoutContext` keeps the containing flow axes and
`ParentFormattingContext` role together as the complete containing context and
cache identity. Flex-item roots require explicit parent flow axes and keep the
host allocation in the root request separate from the percentage
parent in `FlexItemRootContext`.

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
