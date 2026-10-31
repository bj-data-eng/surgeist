# Align captured text scopes

Captured-scope alignment places an actual inline subtree in its containing
line. Text owns the captured identities, font receipts, selected occurrences and
executed geometry. Layout owns normalized alignment requests, line-edge fitting
and the direct adapter. A font measurement is context, not a box or line strut.

## Capture formatting before shaping

Supply `ScopeAlignmentBoundary` and `ScopeFontReference` on the actual
`SourceProcessingScope` entries before calling `process_contextual_source`.
The scope's captured `Style` supplies its metric font context. Authored body
spans retain their independently supplied styles; an inline scope context does
not restyle those spans. The reference supplies the actual script, metric axis
and baseline facts; it does not supply invented font coordinates or authorize
borrowing a child's fallback font.

`Some(Initial)` and `Some(NonInitial)` are formatting facts. An explicit zero
shift is NonInitial. `None` means absent evidence and cannot become Initial
because the eventual geometric displacement is zero. An active child's inline
ancestor also needs authentic classification for its frame; missing evidence
refuses with `MissingBoundary`. A domain's font context alone is not an alignment
target.

Qualifying NonInitial boundaries establish shaping breaks before provider work,
following pinned [Text3 boundary shaping](../../../references/css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#boundary-shaping).
Joining and lookaround retain their separate semantics; these are not inserted
Unicode characters or arbitrary line-break barriers. A demanded boundary that
divides an indivisible text unit refuses precisely rather than splitting raw
glyph output afterward.

Captured handles retain their actual capture, parent and domain. The
`capture_ordinal()` is diagnostic within that capture; the ordinal alone cannot
authorize an association. Layout uses the genuine `ScopeAssociations` proof,
independently of wrapping preferences.

## Prepare authentic metric context

Preparation intersects potential targets with real original and offered
alternative candidate incidence. It retains the genuine parent context needed
by active children, including glyph-free parents, and stops at child domains.
Dormant scopes and source-only removal without a frame-bearing occurrence do
not create font demands.

`PreparedText::scope_metrics` borrows the actual retained scope/font instance,
used size, own used line height and named-baseline outcomes. An unavailable
baseline differs from one that was not demanded. Querying a receipt does not
resolve a font or shape an empty sample.

The supplied scope reference and the actual preparation reference are distinct
facts. `common_baseline_reference()` and `orientation()` identify the used
composition frame. Raw horizontal font-design zero is distinct from requested
Alphabetic; upright Central also retains its genuine metric reference and
provenance. Named matching coordinates are relative to that actual native
reference, positive toward line-over. Incompatible references require a genuine
bridge or a typed refusal; equal names alone do not establish a zero offset.

## Allocate exact selected occurrences

`SelectedLine::scopes()` issues handles for the actual selected presentation.
Each handle exposes its scope and parent receipts before finalization. Direct
metric contributions and descendant-inclusive contributions are separate views;
the common-line inventory includes actual paragraph, line and box struts.
Scope-font receipts do not manufacture another strut. These pre-finalization
inventories belong to the active feature; an ordinary front without it retains
its existing finalization path.

`LineBoxStrut::scope` preserves the actual captured owner of a box contribution;
`None` means genuine common-line participation. A wrapping adapter cannot discard
that association and recover it later by matching metric values. When a whole
scope request owns the property, its nominated own boundary-box contribution is
intrinsic Baseline participation, with unchanged metrics. The adapter lowers it
once using the real scope/node/pair proof; genuine child edge operations remain
independent. Contradictory duplicate property requests refuse rather than silently
being dropped. A nominated own boundary is not a second descendant edge root.
The source owner can be an unclassified represented child whose frame is supplied
by an actual allocated ancestor; that numeric leaf does not acquire a child font
receipt or an invented Initial classification. Preserving source custody alone
on an ordinary wrapping front does not activate the alignment feature.
`CapturedControl::scope()` observes the exact original control event's enclosing
source scope, separately from an admitted own-box endpoint nomination. A
native-only box requires its real endpoint proof; it cannot acquire a fabricated
captured box scope. A genuinely empty late-only contribution needs an authentic
early producer or a precise representation refusal.
Wrapping's raw box-metric declaration is a separate producer phase: its actual
Captured/native-pair binding issues the Text request's source owner. An embedded
input `None` makes no separate owner assertion; an input handle must agree with
that proof. It cannot override the producer's actual identity.

An allocated metric row is itself a selected contribution. Its source owner
need not also own a glyph or control in that line. Without a classified target
ancestor, it participates in the common frame while retaining its source
identity. A target ancestor can supply its frame only from an authentic font
receipt already prepared for that target; a late metric row cannot resolve a
font or invent a child's classification. Retained limits or domain font context
alone do not create alignment demand.

Finalize with `LineAllocations { spacing, scopes }`. `ScopeAllocations::Complete`
contains one `ScopeAllocation` for every demanded selected scope, including
explicit zero. `None` is valid only when no scope allocation is demanded.
Stale, foreign, duplicate and incomplete handles refuse before publication.
Shared native output requires compatible frames; ancestor sharing alone is not
a conflict.

Each own allocation is a used-unit positive-over displacement. Text executes
the actual participating parent chain once and retains own and cumulative
contributions in `FinalizedLine::scope_allocations()`. Retry comparison uses
captured identity and actual request/allocation facts, rather than the fresh
allocation identity of a newly selected handle.

Top/Bottom alignment cuts an aligned subtree at that child: its root aligns to
the containing line rather than inheriting an ancestor's vertical-align shift.
Lexical scope enclosure is unchanged. Baseline/Shift descendants remain in the
same aligned subtree; nested edge roots form independent groups. Existing
atomic/control Top/Bottom slots keep their canonical line-edge placement.
For an edge-root scope, the own allocation subtracts the parent's cumulative
contribution so that composition reaches the actual line-relative position once.
Use the same accepted line's edges, because refitting can change them. This follows
the pinned [CSS2 aligned-subtree definition](../../../references/css2--visudet.html--12e8bc0e6b7c.md)
and [Inline3 line-relative alignment](../../../references/css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md).
Minimal-height baseline ties retain the native line-metric policy.

`with_spacing` changes spacing from the immutable selected basis while retaining
scope placement. Finalization and replay do not resolve fonts or shape again.
Signed final bounds feed Layout's fit, float and height retries. Glyph ink,
decorations, caret, selection, hit and accessibility geometry consume the same
executed frame while raw source/font facts and local metrics remain unchanged.
In-flow slots already contain their scope contribution. An out-of-flow static
anchor consumes `AtomicSlotFeedback::scope_displacement` once at its real static
edge; it cannot infer that contribution from the slot's position or shift the
positioned box a second time.
`FinalizedControlView::scope_displacement()` supplies the corresponding actual
control-frame contribution for canonicalization and publication.
`line_box_strut_scope_displacement(index)` reads the actual canonical request row's
applied contribution. Valid common or escaped edge rows return explicit zero;
only an out-of-range index returns `None`. Source-owner equality alone cannot
substitute for that row identity when different leaves have different alignment.

## Bound each transaction

`ContextualSourceInput::scope_alignment_limits` supplies immutable feature limits
before metadata or boundary work. Missing limits with supplied facts refuse.
Zero limits are meaningful. The feature does not install a mutable global
budget or claim a process-wide memory limit.

| Resource | Meaning |
| --- | --- |
| Records | Retained input rows plus rows materialized by this transaction, including scratch later dropped |
| Font queries | Actual scope-font receipt resolver invocations by this transaction |
| Structural work | Documented actual scope, event, edge, operand, metric and storage work |
| Payload bytes | Live feature custody, including capacities and unique pinned resources; peak includes actual coexistence |

`ScopeAlignmentUsage` records executed transaction accounting.
Work admission covers feature-owned ordering, lookup candidates, descriptor
visits and storage movements, including rejected candidates. A documented row
unit can include its bounded reference arithmetic. Pure comparison callbacks
may use a checked conservative bound from the actual retained rows, admitted
before comparison; that bound is not an execution trace. Borrowed readback
outside an allocating transaction does not acquire a transaction ledger.
`ScopeAlignmentRetention` separately reports the actual retained rows and live
bytes at a completed producer boundary. A successor seeds its budget from that
retention, not a prior monotonic record count that includes discarded scratch.
Layout includes its own feature storage alongside the retained Text input.
Its canonical per-line scope descriptor buffer shares its storage and lease
across paragraph and packet clones until the final owner retires. Global
convergence and cycle comparisons admit their normalized scope and frame-row
payload work before entering the native equality operation.
Its early admission remains live through feature-triggered association and
proof copies, admitting their actual capacity and scratch coexistence before
growth. Existing wrapping graphs and ordinary proof machinery retain their
native resource responsibility; newly retained scope-owned copies remain part
of feature accounting.
It supplies a reduced feature allowance before a child Text operation allocates
while Layout buffers remain live. The allowance cannot exceed the immutable
captured limit; a post-operation sum alone does not establish the combined peak.
`ScopeAlignmentAllowance` supplies total Text-operation record and byte caps,
including retained input. The explicit `select_with_scope_allowance`,
`finalize_with_scope_allowance` and `with_spacing_with_scope_allowance` operations
apply those reductions without changing geometry identity. Ordinary methods use
the captured caps. An excessive allowance refuses rather than being clamped.
Active finalization counts its actual new geometry, spacing, prefix and cloned
shaping scratch, including old/new replay coexistence. Preflight bounds are
admission evidence, not observed peak bytes. Related retained fitter products
need actual shared-owner deduplication; summing scalar receipts is insufficient.
Incoming `Complete` allocation vectors join the retained input before scratch
admission: their lengths contribute records and their actual buffer capacities
contribute heap bytes. The by-value enum and vector headers remain on the stack
and add no separate heap allocation.

`scope_alignment_census` counts an actual input first and then unique related
product allocations. Supply `ScopeAlignmentCensusInput::Product` for a completed
product, `Selection` for a valid preparation and selection request, or `Flow`
for a valid preparation, segment and observation request. The operation views
include the actual prior line results retained by their typed request proofs.
A preparation alone does not own those request resources. The census observes
resource ownership; it does not validate a request or replace native association
and admission checks.

A selected result retains the unique transitive prior line results used by its
request proofs. Their shared allocations count once, including when those same
results also appear among related products. Traversal uses the retained flat
closure rather than recursively walking an arbitrarily deep line history.

`ScopeAlignmentProducts::Mixed` borrows heterogeneous strong products;
`FinalizedSlots` borrows stable optional strong line slots. Every slot inspection,
including `None`, consumes work. An allocation-free uniqueness pass establishes
the complete live union before fallible seen-storage growth. Related products
must remain stable across those repeated inspections.

The native Layout adapter admits and initializes its registry and strong-guard
slots before producing the outputs they will track. Initialized slots are real
Layout metadata rows; replacing `None` with a retained handle consumes work,
without creating a new slot. It pins actual canonical products before census and
through the child Text operation. A weak observation alone cannot freeze this
set: expiration between rescans can invalidate ordinal-based uniqueness.
`ScopeAlignmentWeakProduct` observes the canonical line allocation, including its
real clones and fragments, without inventing source or placement authority.
Expired observations are dropped before census; the current adapter's private,
synchronous session owns the related-product membership.

Shared allocations count once by actual resource identity. Allocator overhead,
process RSS, opaque provider internals and arbitrary caller-retained results
are outside this precise payload contract. Ordinary body matching and shaping
retain their existing native/provider responsibility; this feature does not
retrofit a universal cap over those owners. Newly owned scope-cut, occurrence
and frame work remains charged. Refusal preserves previously accepted results.

See [composed lines](composed-lines.md) for selection and placement, and the
[Text/Layout contract](../../../docs/text-layout-contract.md) for subsystem
ownership. Combined-cell construction and its payload integration remain a
separate producer responsibility; moving an endpoint alone does not certify it.
