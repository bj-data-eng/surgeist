# Native box-edge wrapping

This determination selects the box-edge semantics consumed by #65/#72.
Crate documentation and source establish current implementation support.

## Applicability and propagation

Pinned [Text 4 §5.3](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#wrap-before)
defines noninherited `wrap-before`/`wrap-after` values on inline-level boxes and
flex items. `avoid` applies to both kinds of wrapping; `avoid-line` and
`avoid-flex` apply to their respective kind. `line` forces the applicable inline
line break. `flex` forces a flex-line break only for an actual flex item in a
multiline flex container. Applicability follows the box's actual role.

The imported [Break 3 propagation rule](css-break-3-layout-decisions--editor-capture-20261010--debb8c3e11e2.md#break-propagation)
uses the first/last **in-flow child box** and preserves computed declarations.
Layout owns effective propagation through the actual inline chain and its
formatting-context boundary. Each consumed step requires genuine child/parent
box identity, role, in-flow status, relevant child order and prefix/suffix
completeness. Before and after consume their respective side's evidence.

A zero-decoration, removed-only or explicitly inventoried empty inline box
remains a box. Out-of-flow children are excluded. An atomic child's own exterior
edge can participate; its internal formatting context is opaque to propagation.
Explicit native pairs identify their complete inline box, separately from its
endpoint nodes. Source-scope ancestry supports propagation when its validated
association also proves these generated-box facts. Missing box incidence or
ordering produces a precise refusal for the requested step.

## Independent avoidance constraints

[Text 4 §5.2](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#wrap-inside)
orders nested inside-avoidance, while §5.3 discourages the specified edge.
Neither clause supplies a universal edge-versus-inside priority.

The native completion retains independent constraints identified by actual
box/edge occurrences. Prefer a fitting candidate whose violated constraints are
a strict subset of another's. Preserve nested-inside ancestry preference where
edge constraints introduce no opposing preference. For incomparable mixed
constraints, retain the fitter's ordinary candidate preference. Select from the
nondominated candidates; declaration count and arbitrary weights supply no
priority. This mixed-conflict rule is explicit Surgeist policy.

Compare within the same admitted search stage and actual finalized fit context.
Avoidance preserves ordinary opportunities and their intrinsic effects. It cannot
create a cut, defeat mandatory completion or establish insufficiency merely by
hiding a fitting avoided point. Existing conditional and emergency stages retain
their own prerequisites.

## Early mandatory-boundary semantics

Layout owns box applicability, effective declarations and propagation. Text
owns authenticated mandatory-boundary causes, processing barriers, Unicode and
transformation analysis, shaping and issued forced-break capabilities. Numeric
and flex selection consume the same Layout-owned effective plan directly.

A computed inline force enters the existing `ContextualSourceInput` operation
through a distinct semantic boundary request. Its genuine input-local scope,
endpoint occurrence or supported atomic edge is validated during contextual
admission before processing executes once. Text retains computed-cause custody
separately from authored controls and original characters. Late promotion of a
prepared soft opportunity cannot supply these earlier semantics.

The selected native phase completion establishes the hard boundary before
transforms, typographic-unit formation and shaping. A base and combining mark
on opposite sides form separate hard-bounded segments; a contraction cannot
execute across that boundary. Preserve the original source contributions and
ordinary standalone-segment handling. This cross-unit treatment is bounded
Surgeist policy completing the immediate-force requirement.

Reject a late attempt to split an already prepared unit or executed contraction.
Reject an input edge inside an indivisible decoded scalar or document delimiter.
The effective force neither snaps to another cut nor divides a provider glyph.
Computed causes insert no source character; plaintext remains governed by its
existing source and whitespace contract.

## Phase and identity custody

The early producer supplies real input-local source associations and a borrowed
preparation-independent view of the host box inventory. Text admission remains
the sole owner of canonical source capture. Validate the resulting source,
effective targets and computed causes before issuing a resolved Layout plan.
Normal preparation then supplies actual issued participation.

Final Layout admission revalidates every influencing box role, declaration,
association, parent/order/context and consumed propagation proof against the
current normalized tree. It then binds the actual returned capture, causes and
prepared paragraph identities. A receipt alone cannot authenticate semantic
changes across preparation. Internal admission factoring preserves the existing
processing lifecycle; an independently reusable public capture stage requires
a concrete consumer need.

## Original box-cut incidence

For an existing prepared soft opportunity, Text authenticates where its executed
prefix can meet the original ordered box inventory. Layout supplies genuine box
and order evidence and selects among those authenticated placements using the
avoidance policy above. A placement binds the preparation, issued boundary,
capture and domain; it grants no additional break opportunity or capture stage.

All participating scopes describe one consistent original cut. Do not combine
mutually exclusive before/after placements into simultaneous edge violations.
An exact edge classification requires correspondence proof: a hidden edge inside
an indivisible typographic unit is a nonmatch; missing or ambiguous correspondence
is not a nonmatch. Nonempty scopes alone do not resolve intervening empty boxes
or retained events. Text derives this proof from capture and executed transforms,
not reconstructed UTF-16 offsets, glyph positions or witness extrema.

For example, an empty inline box between `A SPACE` and `B` can admit a placement
before or after that box at the same issued soft opportunity. If the box avoids
only its before edge, the otherwise equivalent after placement is preferred;
reverse the declarations to exercise the before placement. After the preceding
box and before the following box can also describe one genuine shared cut.
Several empty boxes still require one globally consistent ordered placement.

Retained removal ownership, phase-I deletions, empty-origin events and computed
or physical endpoint pins constrain admissible placements. Removed-only boxes
are not freely movable empties. The selected placement must preserve the consumed
event prefix and indivisible units. Carry it through outgoing/incoming box
assignment, replay and cache identity. Empty boxes can affect struts and line
height; equal glyph prefixes alone do not permit reuse of complete line results.

The basis is [Text 4 §5.3](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#wrap-before)
and [§6.5](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#line-break-details):
avoidance concerns actual edges and first/last-character breaks occur at the
box margin edge. The representation and choice of ambiguous metadata placements
are explicit Surgeist policy. Until correspondence is represented, precise
incidence refusal is truthful; full box-edge support includes representable empty
boxes. Acceptance must cover asymmetric empty-box declarations, one ordered cut
across several boxes, pins and removal custody, hidden-unit nonmatches, foreign
capture/domain errors, and selected-placement replay with observable box metrics.

## Inventory-realizable placements

Text's correspondence proof authenticates an original cut; Layout additionally
proves that the actual generated inventory can realize it. Before preference
ranking or metric trials, exclude a soft placement that internally cuts a wholly
movable empty in-flow subtree merely to distribute its struts. This includes a
glyph-free parent whose complete in-flow subtree has zero advance and no actual
content, forced boundary or other retained occurrence requiring separate line
fragments. An empty child alone does not supply that reason. Assign each such
box and its empty subtree wholly to one line, with each contribution once.

For nested S(T) under these conditions, Text may authenticate BeforeS,
InsideS/BeforeT, InsideS/AfterT and AfterS. Layout admits only BeforeS and AfterS.
S.After Avoid then prefers BeforeS; reversing the exterior constraint prefers
AfterS. WrapInside Auto does not admit an unrealizable placement. All-Auto may
retain either complete whole-subtree outcome when otherwise equally preferred.

Use genuine complete generated-box and participation evidence recursively. Skip
a proven unrealizable class; missing required realization evidence is a precise
refusal. Preserve Text's issued opportunity, executed relation prefix and public
incidence identity. Zero width alone is insufficient: real text, atomics,
retained removal/control pins, forced breaks, content on both sides, bidi and
decoration can require actual fragments. Nonempty parents and genuine forces
retain their existing fragmentation behavior.

The [inline fragmentation model](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#line-boxes)
describes splitting overflowing inline contents or a forced break; empty-box
metrics do not independently authorize an extra continuation. Text4's exterior
margin-edge rule supplies related edge precedent. The recursive completion for
these otherwise-empty native subtrees is explicit Surgeist policy, not a claim
that CSS makes every glyph-free parent monolithic.

## Actual box metrics

`WrappingScopeInput::metric_contribution` supplies the actual associated box's
line-box contribution. `Some(Strut(...))` is used geometry, including its
baseline/top/bottom alignment, and applies on the assigned line even when edge
declarations are Auto. `Some(NoContribution)` affirmatively supplies absence;
`None` means evidence was not supplied. Genuine canonical/native aliases share
one witness: a supplied value can fill an unsupplied alias, while contradictory
supplied values refuse. Do not add the same box contribution twice.

When original-placement selection reaches an otherwise unrepresented box,
missing evidence cannot become zero or a guessed font strut. A captured Inline
scope remains an actual box even without a declaration row or emitted glyphs.
Empty-origin events, removals and pins can fix its placement without supplying
its metrics. Neutral or irrelevant callers do not acquire unrelated witness
requirements.

An existing allocation discharges the demand only when the consumed contract
proves that it owns this same box's contribution. The current public Text
protocol permits this proof for an actually selected control strut with a
validated canonical/native endpoint association. A control merely inside an
ancestor box is not that proof. Control struts are Layout-allocated geometry,
not evidence that a font was resolved for another box.

The [inline layout-bounds rule](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#inline-height)
uses directly contained glyphs and, for glyph-free or fallback-only boxes, the
box's own first-available-font strut. Current glyph-font/range readback does not
certify that per-box ownership. Descendant font contributions, a PreparedScope,
an EmptyOrigin or the paragraph-wide strut therefore cannot discharge a missing
box witness. Supply real native used geometry or refuse precisely; do not move
font-family resolution or fallback inference into Layout.

Prepared Text flows combine the admitted allocations through Text's existing
metric groups. Numeric-only native flows use their existing numeric metric
owner and actual control geometry. Assignment and canonical allocation feedback
retain box identity, alignment and incoming/outgoing custody through candidate
evaluation, rejection, continuation, replay and cache reuse.

### Paragraph completion and retained pins

A boundary incidence authenticates an original prefix placement, not additional
selection or placement authority. A computed force and paragraph completion can
share a prepared boundary whose sole original placement remains inside a box.
Layout must not treat the last available incidence as proof that every box has
completed. For a paragraph-ending assignment, genuine inline-scope exits before
that captured paragraph's exit require an outgoing `AtAfter` or `After` relation.
Use actual same-capture/domain scope exits, not clipped intersection endpoints;
scopes continuing beyond the paragraph retain their existing frontier custody.

An existing issued ParagraphEnd selection can complete those local boxes after
an actual Forced ending, including a final selection with equal start and end
boundaries. Its completion frontier is distinct from the prefix incidence: it
retains the actual preceding Forced selection, attributed incoming box frontier,
same preparation and captured paragraph horizon. Text's ordinary Paragraph
selection owns the remaining executed correspondence. The original forced pin
and its incidence queries remain unchanged. Counterfactual measurement must
retain the corresponding authentic mandatory sequence and observed basis;
replay and reuse preserve the same distinction.

A genuine completion does not itself create a formatted CSS line. Apply the
[pinned phantom-line rule](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#phantom-line-box)
to each selected line: actual retained text, preserved whitespace, an in-flow atomic
(including zero-sized), any individual nonzero inline-axis margin, border or
padding, or an actual Forced ending prevents phantom disposition. Evaluate
individual sides; cancelling sums, zero advance or empty ink cannot establish
absence. Vertical struts and block-axis-only decoration do not prevent a line
from being phantom. A Paragraph receiver does not inherit the preceding line's
Forced status, regardless of whether earlier source nodes exist.

Retain phantom completion as a nonformatted result in the same selection and
placement path: genuine ParagraphEnd, unchanged pin, complete original-box exits,
actual metric receipts and whole-paragraph coverage remain authoritative. Real
numerical receiver metrics still require their genuine witnesses; suppression
is not permission to substitute zero or omit an allocation. Phantom completion
adds no published ordinary line rectangle or baseline, formatted ordinal,
first/last formatted-line role, block/intrinsic/float/scroll/paint extent or
nominal caret rectangle. Descendant positioning uses the rule's zero-height
line box through its actual owner. Retain appropriate source and nonvisual
provenance without dropping the final receipt or switching to partial coverage.
Raw standalone Text's native numerical metric policy is separate.

Within an already scheduled nonempty Text run, the existing genuine terminal
selection can perform this completion. Wholly node-free paragraph admission is a
separate scheduling scope and must apply the same CSS disposition once admitted.
When neither an authenticated prefix placement nor a genuine completion
transition closes local boxes, Layout returns `UnsupportedIncidence` before
publishing geometry; it must not forge an exterior incidence or source participant.
