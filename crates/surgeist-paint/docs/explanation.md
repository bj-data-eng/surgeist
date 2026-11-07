# Explanation

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

Caller input and Ledger's separately bounded observer/report storage are outside
Paint byte caps. Integer counters that encode graph positions/ranges, identities
and capacity custody are algorithm state, not a second numeric budget authority.
Resolved levels outside i64 require an upstream diagnostic; Paint does not
truncate authored CSS values or resolve calculations.
