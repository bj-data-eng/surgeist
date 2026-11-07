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
[reference inventory](../../../references/SOURCE-EDITIONS.md). Ordering contexts
alone does not place non-context inline/float/positioned-auto primitives among
them. Full primitive emission and computed Style integration remain future work.

## Work and storage

Ledger owns all cumulative action counts, live/peak bytes, limits, refusals and
phase timing. Topology visits include buffer initialization, parent walks,
adjacency construction, DFS stack actions and group boundary inspection.
Comparator and index-move actions are admitted before fallible sorting proceeds.
Output records include initial membership slots, final membership/context writes,
child references and context range updates; they are not draw-command counts.

Requested storage is previewed without committing usage. After fallible
allocation succeeds, its actual capacity is acquired before use/publication;
refused backing is dropped. Failed reservation records no accepted live bytes
or artificial peak. Arrays are preallocated with proven cardinality bounds, so
there is no unaccounted reallocation growth/overlap. Scratch is released when
dropped. The successful report retains result Vec capacities and the fixed plan
owner; dropping the plan later is outside this synchronous operation. Failed
operations drop/release all admitted backing and retain accepted work.

Caller input and Ledger's separately bounded observer/report storage are outside
Paint byte caps. Integer counters that encode graph positions/ranges, identities
and capacity custody are algorithm state, not a second numeric budget authority.
Resolved levels outside i64 require an upstream diagnostic; Paint does not
truncate authored CSS values or resolve calculations.
