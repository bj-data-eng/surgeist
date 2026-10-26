# Layout boundaries and geometry

`surgeist-layout` is an independent library boundary. It accepts normalized
layout facts from its host and produces geometry and computation results. The
[crate-level documentation](../src/lib.rs) describes the public contracts;
[host traits](../src/tree.rs) describe how a tree supplies inputs and receives
completed results.

## Layout and integration ownership

Layout owns normalized layout values, algorithm inputs, traversal contracts,
caches, box output and its direct adapters from computed style and Text. DOM owns
the canonical retained tree; hosts implement the tree interface. Text owns
shaping and source mappings, Render owns paint conversion, and Runtime owns
orchestration and coherent publication. Root `surgeist` provides optional facade
composition, bootstrap and shared API auditing. See the
[shared text contract](../../../docs/text-layout-contract.md) for the selected
preparation and line-finalization boundary.

The native Text entry retains actual immutable preparation and finalized-line
identities through a checked borrowed-tree capture. Layout fits Text-issued
candidates against box and float constraints, finalizes their actual allocations,
and places the exact lines only after their canonical frame is stable. Completed
batches retain the placed Text layout and authentic source fragment handles for
downstream consumers. Caller-shaped widths and local segment IDs cannot replace
this preparation.
Layout does not parse authored CSS or resolve computed style. Its consuming
adapters own lowering producer facts into these normalized inputs.
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

Layout owns lowering consumed producer facts into these finite inputs; CSS
parsing and computed-value resolution remain with their respective owners. The
browser fixture adapter is a constrained layout-ready test bridge, not another
CSS front door.

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
otherwise unresolved CSS. Layout owns its consuming computed-style adapter;
callers supply that resolved value at the current normalized input boundary.

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
alignment, and snap-stop metadata. Consuming subsystems associate and transform
these values into their own coordinate and lifetime domains. Live offsets,
focus/target scrolling, snap selection, CSSOM, scrollbar UI and host events remain
with those consumers; Runtime coordinates coherent publication. Layout does not
claim a live scrolling runtime.

The general signed physical and flow-relative range types keep finite ordered
minimum and maximum bounds. When an axis runs in reverse, `FlowAxes` swaps and
negates endpoints so negative minima and maxima retain their meaning.

These boundaries describe the represented behavior; they do not establish broad
CSS conformance. The [browser parity guide](../tests/layout/browser_parity/README.md)
describes the checked-in evidence and its classified unsupported cases.
