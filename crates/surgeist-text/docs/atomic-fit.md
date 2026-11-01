# Margin-edge fitting

An in-flow atomic allocation carries an explicit `AtomicFitAllocation`. Use
`NativeOccupied` for the existing Native contract. CSS-used geometry can supply
`CssMarginBox(CssMarginBoxFit::try_new(start_margin, border_inline_size, end_margin)?)`.
The receipt retains finite signed margins, a nonnegative border inline size and
its checked used advance. Selection validates it against the same issued atomic's
actual flow, border size and occupied envelope. It does not infer CSS authority
from a difference between advance and occupied size. Relative positioning and
cross-axis metrics remain separate geometry facts.

For a non-atomic inline BoxEdge, use the checked native-lane
`InlineBoxEdgeComponents` with its authenticated owner, parent and assigned side.
Its signed flow and nonnegative border/padding strip are separate observations;
see [endpoint construction](reference.md#native-inline-endpoint-geometry).
Contribution remains a finite nonnegative scalar without this CSS authority.

`FinalizedLine::measurements()` keeps its existing raw `InlineMeasure` views.
`margin_edge_fit()`, `margin_edge_min_content_contribution()` and
`margin_edge_max_content_contribution()` return the operation-specific
`MarginEdgeFit`. Its authority, signed advance, signed constraint endpoints and
required inline size are indent-free observations of that same finalized selection.
`native_occupied_bounds()` retains the actual unanchored opaque extrema, or
`None` when CSS composition has no opaque allocation or authority is Native-only. They
do not relocate or clip a box, change glyph/source geometry, or cause shaping.
Spacing replay retains the allocation authority and rebuilds the observation
from the same chosen provider basis.

## Native authority

A `NativeOnly` observation preserves the existing requirement that both signed
advance and actual occupied span fit. A Native advance6 with occupied100..110
therefore needs10; it is not implicitly anchored to0. Its input rectangles and
signed flow remain exactly what the Native caller allocated.

## CSS-composed authority

A genuine selected CSS margin-box allocation or checked component BoxEdge
activates `CssComposed`, including explicit all-zero components. Context-only
BoxEdges grant no selected fit authority. This is an explicit Surgeist native mixing
policy, not a CSSWG definition for arbitrary Native allocation mixtures.

Execute the actual selected visual traversal in its prealignment line frame.
Keep final signed flow F. For each `NativeOccupied` atomic, translate its admitted
local occupied interval by the actual incoming pen exactly once. Retain the
joint minimum and maximum of those positioned Native intervals. CSS-authorized
occupied/border/ink rectangles and non-atomic decoration strips do not enter that
independent requirement.

With opaque endpoints l/r, the signed constraint endpoints are
`L = min(0, all l)` and `R = max(F, all r)`. Without an opaque allocation use
`L = 0, R = F`. The required size is `max(0, R - L)`, with only that final
nonnegative floor. In particular R has no implicit zero endpoint.

| Selected flow and actual opaque interval | F | L | R | Required size |
| --- | ---: | ---: | ---: | ---: |
| CSS100, then Native6/[0,10] at100..110 | 106 | 0 | 110 | 110 |
| CSS−100, then Native6/[0,10] at−100..−90 | −94 | −100 | −90 | 10 |
| The preceding selection, then CSS100 | 6 | −100 | 6 | 106 |
| Ahem10, then non-atomic end margin−5 | 5 | 0 | 5 | 5 |

The last row leaves actual text at0..10; a narrower margin-edge requirement does
not clip or relocate its geometry.

Later selected CSS contributions may retract the pen. Do not retain a CSS
prefix maximum, clamp at each box, borrow an unselected future closing margin,
sum independent Native widths or re-zero each Native interval. Another box's
margin cannot erase retained opaque endpoints. The required size is admission
evidence, not a containment rectangle or a new origin.

Layout supplies actual used CSS atomic and non-atomic components, consumes this observation
through its ordinary finite/intrinsic/conditional fit loop, and retains genuine
block-height/float retry and independent BFC rules. An authorized negative
margin can overlap a float without falsifying the actual border or ink.
The numeric box/control path already consumes actual CSS signed margin-box
flow rather than unknown Native allocations.

## Indentation and intrinsic demand

The consumer combines its actual signed indent I with the retained operands;
it must not translate the already origin-anchored constraint interval. For
`CssComposed`, compute `F_I = I + F`. When opaque extrema are present use
`L_I = min(0, I + native_min)` and `R_I = max(F_I, I + native_max)`;
otherwise use `L_I = 0, R_I = F_I`. The requirement is
`W_I = max(0, R_I - L_I)`, compared against the full preindent band B.
All additions/subtraction must be checked. Do not floor flow before adding
indent, subtract indent a second time, or move any retained geometry.

Thus all-CSS F−20/I10 needs0 and fits B5, while F0/I10 needs10 and does not.
F20/I−10 needs10. Opaque100..110/F106/I−20 needs90: the old artificial zero
anchor must not become−20. Opaque−100..−90/F−94/I10 still needs10.
`NativeOnly` keeps the existing comparison against B−I without acquiring
this CSS composition rule.

Layout uses the same checked composition for fit, min-content and max-content
observations. Its block intrinsic inline demand consumes the accepted line
report, independently of physical child borders and visible overflow. A border
of10 with end margin−4 contributes6 to that demand while its border, content
and scroll geometry retain their actual extents. Float intrinsic contributions
and true block-height retries remain independent. The numeric all-CSS path
retains signed flow comparison F≤B−I for nonnegative B; no premature width
floor is inserted into that equivalent expression.

## Ownership and limits

The authority is stored in the canonical request and actual finalized atomic or
endpoint record, not a side cache. It participates in frame/conditional/replay
equality, including a BoxEdge's component lane and values. The fit observation adds
fixed data and no separate allocation arena. Applicable
existing Scope accounting uses the actual enlarged finalized-header/box-buffer
types before growth and in retention census. Existing Native request/proof
buffer ownership is unchanged; this is not a universal memory cap or a guessed
sum of resource receipts. Diagnostic storage keeps its existing exclusion.

Complete fragmented inline decoration and painting remain separate work.
Existing formatter-frame validation, Source bidi facts and unsupported Cell
combinations are unchanged.
