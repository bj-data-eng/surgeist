# Explanation

## Linear-gradient geometry

The image's consuming context supplies a concrete gradient-box size. Gradients
have no natural dimensions, and a parsed gradient does not supply that size.
`resolve_linear_gradient` consumes Layout's physical `Size<f64>` and returns
start/end Layout points and a line length, all local to the box's top-left.
The consumer applies placement later. The operation does not infer a particular
Layout box, evaluate CSS numeric expressions or choose stop colors.

In physical y-down coordinates, zero degrees points up and increasing angles
rotate clockwise. Omitted direction is represented by the default bottom
choice. Side keywords retain exact cardinal directions. The centered line has
length `abs(W * sin(A)) + abs(H * cos(A))`; its endpoints may lie outside the
box. For a 200 by 100 box, 45 degrees gives endpoints (25,125) and (175,-25).
A corner keyword instead makes the line perpendicular to the diagonal joining
its neighboring corners. Top-right gives (60,130) and (140,-30), so those
neighboring corners both project to its midpoint. Physical directions are
independent of writing mode. These rules follow the pinned
[CSS Images3 linear geometry](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#linear-gradient-syntax).

Finite nonnegative dimensions include zero. Only a zero projected length
produces coincident center endpoints: a zero-width box with a vertical direction
can still have a nonzero line. Both-zero corner input has a zero line without
normalization by zero. Finite angles are reduced before trigonometry. Invalid
width, height or angle has a typed diagnostic. Overflow or numerical collapse
of a positive line returns `UnrepresentableLine`; the operation uses no
size epsilon or device-resolution fallback. This finite f64 representation
policy does not claim exact real arithmetic for every input.

The returned length supplies the shared stop-position resolver. Authored
numeric/color adaptation and Render field composition retain their separate
responsibilities. Geometry alone does not turn a parsed gradient into a complete
rendered image.

## Radial-gradient geometry and cases

`resolve_radial_gradient` uses a concrete physical Layout size, an optional
already-used center and a coupled circle/ellipse size request. The default is
box center and ellipse/farthest-corner. Explicit centers may lie outside the
box. The native point does not evaluate CSS Position, logical flow or edge
offsets; the consuming authored adapter still supplies those used facts.
Explicit circle radius is an absolute length. Explicit ellipse radii reuse
`GradientPosition` contributions: horizontal fractions resolve against the
full box width and vertical fractions against its full height, with fused
multiply-add. These are radii, not diameters. Native negative used radii are
typed input failures; that diagnosis does not decide authored CSS range clamping.

Side extents measure distance to infinite supporting lines. Selecting a corner
is separable in x and y, even when the center is outside the box. A circle uses
the selected pair's hypotenuse. For positive ellipse side axes a and b, preserving
their aspect ratio and passing through that corner gives axes sqrt2 times a and
b. This avoids an overflowing aspect quotient. Default 200 by 100 geometry
therefore has axes (100sqrt2,50sqrt2). The rules follow the pinned
[radial geometry](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#radial-gradient-syntax).

The private result exposes finite center and a resolved case view. Positive
circle fields use equal positive ellipse axes; the original shape flag no
longer affects their geometry. The ordered cases are zero circle, otherwise
zero horizontal ellipse axis regardless of height, otherwise zero vertical
axis. A both-zero ellipse is ZeroWidth. Zero-aspect corner requests select
their case directly: cancelling a zero axis in the ordinary corner derivation
would divide zero by zero. The result stores no guessed replacement radii or
original sizing flags. Only an Ellipse case's horizontal radius supplies the
normal rightward ray length. Degenerate cases require their later semantic
adapter, rather than an unconditional finite basis getter.

Native representation checks retain useful finite geometry without calculating
an unselected overflowing far-side distance or an unnecessary center-plus-radius
endpoint. Required overflow receives a typed refusal. A positive default-center
half that rounds to zero is refused; an explicit already-used zero coordinate
remains valid. If an affine radius rounds to zero, bounded binary64 dyadic
arithmetic distinguishes true cancellation/zero from a nonzero value lost to
underflow. Only true zero selects a degenerate case. The operation uses no
pixel epsilon, saturation or fabricated tiny/huge ellipse. Ordinary nonzero f64
rounding remains part of this native policy; exact real arithmetic for every
input is not promised.

Normal geometry can feed the shared stop resolver. Remaining degenerate stop/field
realization remains under [the radial contribution](https://github.com/bj-data-eng/surgeist/issues/332).
In particular, zero-height mixed absolute/percentage stops can have different
average weights when resolved against an original horizontal radius versus a
substituted unbounded-width limit. The dedicated
[zero-height color measure](reference.md#zero-height-gradient-colors) explicitly
selects the latter as native policy. This geometry result supplies ZeroHeight
without embedding that later color decision or retaining an unused radius.
Zero-width percentage positions resolve to zero through the shared stop resolver
with basis zero. Render's [horizontal-distance consumer](../../surgeist-render/docs/reference.md#horizontal-distance-fields)
uses those physical positions. Zero-circle realization remains separate;
these boundaries preserve the ordered rendering requirements.

## Gradient stop positions

`resolve_gradient_stops` shares used-position resolution between linear and
radial consumers. Supply a finite nonnegative line or ray length and a native
list of stops and hints. `GradientPosition` carries raw already-computed
absolute `length` and dimensionless `fraction` contributions; fraction 1 means
100%. Paint validates both components and computes their affine combination.
This input does not evaluate an authored calculation tree or identify a color
space. The consumer remains responsible for producing numeric and color facts.

A color stop carries an opaque payload and omitted, one or two positions.
Paint expands two positions into consecutive occurrences with the same borrowed
payload. A hint always has a position and remains distinct from a colored stop.
The native list must be nonempty, begin and end with stops, and contain no
adjacent hints. The result borrows that exact list, owns used-position records,
and exposes each occurrence's source index; stop views also expose position
ordinal 0 or 1. Omitted/single stops use ordinal 0. The payload needs no cloning
or color trait, and source identity uses indices even for zero-sized payloads.

Resolve positions before ordering. Default an omitted first stop to zero, then
an omitted last stop still unset to the line length. One omitted stop therefore
stays at zero. Raise each specified position below the greatest earlier
specified position, then distribute each remaining omitted run between its
positioned neighbors. Positioned hints bound those runs under the adopted
[CSSWG resolution](https://github.com/w3c/csswg-drafts/issues/3931#issuecomment-494876538),
while the pinned fixup text awaits that editorial clarification. The
[pinned Images4 syntax](https://www.w3.org/TR/2025/WD-css-images-4-20250930/#color-stop-syntax)
defines double-position expansion and permits a single stop. The result retains
source order, abrupt coincident positions and positions outside the line interval.

Zero length is valid and never a divisor: percentage contributions become zero,
while absolute contributions survive. Native arithmetic uses fused multiply-add
for affine positions and sign-aware bounded interpolation for omitted runs.
Finite positions at -MAX and +MAX can enclose a midpoint zero even though their
total difference overflows. Rounded coincidences are valid; no epsilon or
artificial separation is applied. A nonfinite final calculation or descending
numerical output receives a typed refusal. Render's finite repeat-span and
device-field restrictions remain at the consuming boundary.

The operation validates, expands, fixes up, distributes and publishes under one
Ledger report. Every actual traversal and repeated inspection is admitted;
retained custody includes allocated record capacity and the fixed result owner.
Caller payloads remain borrowed. A refusal drops unpublished backing and releases
admitted custody while preserving accepted work. Optional phase timing uses the
same Ledger authority. Caller iteration after return is outside this completed
synchronous operation, as with the other Paint result views.

This result supplies positions rather than rendered colors. Authored adapters,
hint interpolation, ordinary/repeating field realization and radial semantic
case selection retain their separate responsibilities.

## Rendering boxes and actual contexts

The caller supplies the rendering tree, including generated/anonymous boxes as
needed. Each slot is a distinct box; an external element may correspond to more
than one slot. Parent references can point forward. Validation requires exactly
one root, bounded parents and distinct source indices within each sibling set.
Gaps are valid. Every parent component is checked iteratively; with one root,
a disconnected component necessarily contains a cycle and receives that error.

Paint constructs source or order-modified child traversal itself. Flex/grid
items sort by ItemOrder then SourceIndex; absolute/fixed children use order zero.
Preorder over that reordered tree moves descendants with their item ancestor.
Geometry, bidi visual position and Layout batch slice order do not define it.

Root always creates a context. Applicable integer z-index, fixed/sticky auto,
and resolved effect triggers create other contexts; non-context ancestors do
not trap descendant contexts. Static ordinary z-index is inapplicable, including
when an independent effect creates a zero-level context. Child contexts sort
by effective signed level and reordered preorder. Borrowed views prevent passing
a context handle into another plan.

CSS Position3, Flexbox, Grid and Display3 refine CSS2 Appendix E. See the pinned
[reference inventory](../../../references/SOURCE-EDITIONS.md). `plan_stacking`
stops at contexts. `plan_paint_order` reuses that algorithm and
adds the supported non-table primitive sequence described below. Computed Style
integration and complete Render scene lowering remain separate work.

## Occurrences and primitive order

`PaintRequest` borrows six tables: semantic `BoxFacts`, same-slot `BoxPaint`
categories/float facts, logical `TextSource` records, `PaintLine` associations,
`PaintFragment` occurrences and `PaintPrimitive` appearance records. Text and
immediate child boxes share one source-index domain. One semantic owner may
have several line/bidi/fragment occurrences, including occurrences without
geometry. A line slot identifies an exact container occurrence and logical line
order; a producer's run-local line index is not a global identity.

Constructors preserve raw facts. The operation validates cross-table bounds,
category compatibility, mixed source positions, occurrence/line incidence,
logical order, primitive targets and decoration applicators. The root must be a
nonfloating block. Tables and inline tables return `UnsupportedBoxKind` pending
[#954](https://github.com/bj-data-eng/surgeist/issues/954); they are not flattened
into ordinary blocks. Empty records need no invented draw. Root background
appearance uses separate canvas records, each emitted once; root-fragment
background records are refused.

Box categories are resolved after CSS blockification. Floating, out-of-flow and
flex/grid item boxes therefore use `Block` or `BlockReplaced`, rather than
`Inline`, `InlineBlock` or `InlineReplaced`. Incompatible inline categories return
`InvalidBoxCategory` before occurrence sequencing. Relative/sticky in-flow inline
categories retain their line associations.

Within an actual context, Paint sequences root/canvas or context decoration,
negative child contexts, ordinary block decoration, non-positioned floats,
inline/replaced content, positioned auto/zero, positive contexts and late
outlines. Background color precedes the image appearance group and border.
Applicable text contributions run underline, overline, text, then line-through;
decoration applicators follow owner tree order within each category. Local
appearance records retain their resolved record order.

Float, atomic-inline and positioned-auto pseudo-contexts recursively collect
ordinary content. Positioned descendants and actual child contexts escape to
the containing actual context. Flex/grid items use atomic-inline behavior and
order-modified tree order. Shared classification still distinguishes integer
zero actual contexts from relative/absolute auto pseudo-contexts, and recognizes
fixed/sticky auto and applicable static item integer contexts.

Outlines flush at the tail of each recursively painted scope, including each
pseudo-context invocation. Within that collection, Paint's deterministic local
policy orders owners by order-modified preorder, then their logical fragment
order, then record order. It does not collect every outline at the outer root.

`PaintFragment::block_in_inline` explicitly places a static nonfloating block
occurrence in a real container, skipping a nonempty semantic chain of `Inline`
ancestors. Its `content_order` shares the container-local domain of
`PaintLine::logical_order`, interleaving line-before, promoted block foreground
and line-after. The owner's separate logical occurrence order remains intact.
Semantic ancestry still determines context membership and decoration applicability.
Skipping an unmarked `Block` wrapper returns `UnsupportedFragmentTopology`:
the request cannot certify broader anonymous formatting-wrapper promotion.
That producer composition remains in [#951](https://github.com/bj-data-eng/surgeist/issues/951),
and the diagnostic does not classify the CSS topology as invalid.

Every accepted primitive slot appears once in `PaintOrder::events`, and each
actual context has one balanced `BeginContext`/`EndContext` pair, even when empty.
Pseudo-contexts add no actual-context boundary. Draw views borrow exact request
records. Slot IDs are local integer references: an in-range ID copied from
another request has no intrinsic foreign provenance to validate. Callers retain
immutable maps from these slots to authentic source, geometry, formatter/run/line
and appearance associations. Ordering placement never changes coordinate or
clip ancestry. Actual-context intervals alone do not describe a complete
transform/clip tree or a backend command stream.

## Work and storage

Ledger owns all cumulative action counts, live/peak bytes, limits, refusals and
phase timing. Topology visits include buffer initialization, parent walks,
adjacency construction, DFS stack actions and group boundary inspection.
Comparator and index-move actions are admitted before fallible sorting proceeds.
Output records include membership/context writes, child references, context
range updates and paint-order events; they are not backend draw-command counts.
The complete order operation uses one Ledger report for context planning, input
validation and sequencing, with no nested discarded report.

Requested storage is previewed without committing usage. After fallible
allocation succeeds, its actual capacity is acquired before use/publication;
refused backing is dropped. Failed reservation records no accepted live bytes
or artificial peak. Stacking arrays use proven cardinality bounds. Growing
sequencing buffers admit
new backing while old backing is still held, so growth overlap contributes to
peak custody before the old allocation is released. Scratch is released when
dropped. The successful report retains result Vec capacities and the fixed plan
owner; dropping the plan later is outside this synchronous operation. Failed
operations drop/release all admitted backing and retain accepted work.

Caller input and separately bounded observer/report storage are outside Paint
byte caps. This includes Ledger's observer and Paint's resolved report adapter.
The adapter reserves its backing fallibly during operation setup using the same
scope and detailed-event bounds. Off reserves no report backing; Summary reserves
no event backing. Conversion after Ledger finish uses only the reserved capacity,
so changing profiling does not spend forward algorithm quotas or change live/peak
result custody. Allocator spare capacity does not enlarge the logical bounds.

Paint registers each created scope handle with its explicit `PaintPhase`.
Report construction resolves summaries, refusals and events through those
registrations once, checks every referenced scope against the finished report,
and stores the resolved observations. Display names remain filter labels;
they do not select phases. Readers perform no scope lookup or defaulting.
Unknown handles fail construction with Paint's typed accounting invariant;
upstream Ledger defects retain their named `Defect` payload.

Integer counters that encode graph positions/ranges, identities
and capacity custody are algorithm state, not a second numeric budget authority.
Resolved levels outside i64 require an upstream diagnostic; Paint does not
truncate authored CSS values or resolve calculations.
