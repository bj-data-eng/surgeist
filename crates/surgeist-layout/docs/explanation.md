# Layout boundaries and geometry

`surgeist-layout` is an independent library boundary. It accepts normalized
layout facts from its host and produces geometry and computation results. The
[crate-level documentation](../src/lib.rs) describes the public contracts;
[host traits](../src/tree.rs) describe how a tree supplies inputs and receives
completed results.

## Layout and integration ownership

Layout owns normalized layout values, algorithm inputs, traversal contracts,
caches, reports, and box output. Retained tree identity and sibling coordination
belong to the root integration layer that provides the tree implementation.

Root `surgeist` owns cross-crate adapters, including canonicalizing authored CSS
sizing values, lowering computed-style values and authored CSS order into these
layout contracts, and rejecting property-invalid authored states before layout.
This crate does not parse authored CSS. Root also owns box-generation
replacedness, invalidation, consumer migration and renames, facade composition,
integration, and generated API artifacts; this crate carries no root adapters or
API artifact copies.

Text arrives as validated shaped segments. Layout constructs lines and fragment
geometry; the integration layer supplies source association and shaping facts.
The browser fixture adapter serves this same finite input boundary for tests.
The shared generator owns browser and artifact lifecycle, while layout owns
measurement decoding and XML semantics; see the [runtime reference](reference.md#browser-parity-runtime).

## Normalized grid boundary

Sources: [grid algorithms](../src/grid/mod.rs), [subgrid](../src/grid/subgrid.rs),
and [grid-lanes](../src/grid/lanes.rs).

The grid front door consumes normalized layout facts. Layout owns finite track
components and repetitions, explicit and implicit topology, integer placement,
named lines and validated template-area cells, auto flow, gaps, alignment,
layout-ready order and replacedness, flow axes, and canonical physical box and
scroll output. Lengths, percentages, intrinsic keywords, fit-content limits,
and flex factors enter through their existing validated property domains; this
boundary is not an authored-CSS grammar.

The implemented CSS Grid Level 2 subset is deliberately narrower than broad
conformance: ordinary grids and inherited-axis subgrids compose through shared
topology, placement-before-sizing, auto-fit occupancy, fit-content/flex sizing,
auto-maximum stretch, named occurrences, template areas, standalone-axis
measurement boundaries, inherited baseline data, and canonical overflow
contribution. Baseline grouping remains the existing inherited-axis model.
Grid-aligned positioned layout, additional baseline distribution, and
fragmentation are outside this implemented grid subset.

Inherited subgrid axes keep their inherited track extent. Definite placements
are clamped to that extent before automatic placement; settled automatic
placements are clamped afterward and may overlap when capacity is exhausted.
Only standalone axes materialize implicit tracks. Sparse placement retains
progress per locked major track, while dense placement searches earlier holes.

The experimental CSS Grid Level 3 subset is limited to `grid-lanes` single-axis
packing: hybrid item containing blocks, intrinsic candidate projection,
lanes-specific auto-fit collapse, and nested indefinite subgrid contribution
projection. It does not claim stacking-axis grid-lanes baseline alignment or
complete Level 3 behavior.

Root `surgeist` owns authored CSS parsing, cascade and computed-value
normalization, lowering into these finite inputs, cross-crate integration,
retained identity, and generated API artifacts. The browser fixture adapter is
only a constrained layout-ready test bridge and is not a second CSS front door.

## Geometry, flow, and scroll

Sources: [geometry and flow axes](../src/geometry.rs),
[scroll geometry](../src/scroll/mod.rs), and [node output](../src/output.rs).

The public physical geometry contract uses x/y points, width/height sizes, and
top/right/bottom/left edges. Public layout outputs, cached geometry, and scroll
geometry remain physical. Layout algorithms may use
crate-private logical algorithm geometry while working in inline/block
coordinates. Those carriers stay private until the owning `FlowAxes` projects
them to physical geometry at a contextual boundary.

`FlowAxes` is the sole production owner of writing-mode mapping for
`HorizontalTb`, `VerticalRl`, `VerticalLr`, `SidewaysRl`, and `SidewaysLr`. Its
`Direction` is the already-resolved used inline direction, not authored or
otherwise unresolved CSS. Root `surgeist` owns computed-style lowering and
supplies that used value through its cross-crate adapters.

Scroll inputs are normalized computed or otherwise layout-ready values, not
authored CSS syntax. `ComputedOverflow` atomically validates the two computed
axes; layout privately derives used overflow from that pair and replacedness.
`OverflowClipMarginOf`, `ScrollPaddingOf`, `ScrollMarginOf`, and the snap types
carry finite closed inputs. `ScrollbarWidthOf` is the explicit physical
thickness selected by the caller's overlay/classic scrollbar environment, so
layout neither probes host metrics nor guesses a missing policy.

Successful layout may publish immutable `ScrollGeometryOf`. Its read-only
helpers expose canonical border, padding, content and scrollport boxes;
independent x/y clips; physical-edge gutter rectangles; the optimal viewing
region; used overflow; and one signed physical range containing the zero
initial anchor. The canonical scroll size on each axis is `maximum - minimum`,
including zero. `NodeOutputOf::content_box_size()` and `scrollbar_size()` derive
from that same geometry instead of maintaining mutable duplicate state.

Every present geometry also contains `ScrollTargetGeometryOf`: the target's
local physical border box and scroll margin plus its flow axes, block/inline snap
alignment, and snap-stop metadata. Root consumes these values after retained
association and coordinate transformation. Root also owns authored CSS parsing,
computed-style normalization and lowering, explicit host scrollbar policy,
current offsets, focus/target scrolling, snap-container association and
selection, CSSOM, scrollbar UI, host events, and invalidation. This crate does
not claim a live scrolling runtime.

The general signed physical and flow-relative range types keep finite ordered
minimum and maximum bounds. When an axis runs in reverse, `FlowAxes` swaps and
negates endpoints so negative minima and maxima retain their meaning.

These boundaries describe the represented behavior; they do not establish broad
CSS conformance. The [browser parity guide](../tests/layout/browser_parity/README.md)
describes the checked-in evidence and its classified unsupported cases.
