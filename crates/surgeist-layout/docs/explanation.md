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
Positioned descendants use grid-line references with ordinary absolute sizing
and static-position rules. Additional baseline distribution and fragmentation
are outside this implemented grid subset.

Inherited subgrid axes keep their inherited track extent. Definite placements
are clamped to that extent before automatic placement; settled automatic
placements are clamped afterward and may overlap when capacity is exhausted.
Only standalone axes materialize implicit tracks. Sparse placement retains
progress per locked major track, while dense placement searches earlier holes.

The experimental CSS Grid Level 3 subset is limited to `grid-lanes` single-axis
packing: hybrid item containing blocks, intrinsic candidate projection,
lanes-specific auto-fit collapse, nested indefinite subgrid contribution
projection, packed alignment and positioned line references. Stacking-axis
baseline sharing and fragmentation are unsupported.

Layout owns lowering consumed producer facts into these finite inputs; CSS
parsing and computed-value resolution remain with their respective owners. The
browser fixture adapter is a constrained layout-ready test bridge, not another
CSS front door.

## Fragmentation scheduling

The [normalized scheduler](reference.md#normalized-fragmentation-scheduling)
owns break selection and progress over mode-supplied source and layout facts.
Its provider measures the remaining content for each fragmentainer; Layout
retains independent parallel flows and nested break requirements. The resulting
schedule describes mode-supplied occurrences. The
[native Block adapter](reference.md#native-block-fragmentation-and-publication)
uses actual Block and leaf production at each frame width and publishes a
source-bound fragment graph through the completed batch. Principal boxes and
overflow-only ancestor carriers retain separate payloads. Its box-only profile
refuses unsupported formatting modes; column geometry and fragmented Text
composition are outside that profile. Publication checks the installed source
and constraint target and replaces the root atomically, including detached
members and performed-layout cache state.

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

#### Scroll offset coordinates

`PhysicalScrollOffsetOf` measures x rightward and y downward.
`FlowRelativeScrollOffsetOf` measures inline and block components toward the
respective flow ends. `FlowAxes` exchanges the components when the inline
axis is vertical and negates a component when its physical progression is
decreasing. Both forms accept finite signed values and canonicalize zero.
Conversion preserves the reference position.

The corresponding range types store finite ordered endpoints and clamp each
component to its closed interval. On a decreasing axis, conversion swaps and
negates the endpoints. This preserves the result of clamping in either
coordinate space. The range in `ScrollGeometryOf` uses zero for Layout's
initial anchor and exposes an interval of `[0, 0]` on an axis without a scroll
range. Standalone range values accept any ordered finite endpoints.

[CSS Overflow 3's scroll offset definition](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/#scroll-offset)
leaves the module-wide coordinate direction open.
[CSSOM View's scrolling area](https://www.w3.org/TR/2025/WD-cssom-view-1-20250916/#terminology)
uses physical coordinates increasing rightward and downward, with an origin
at the initial containing block origin for the viewport or at the top-left
padding edge of an element in its default scroll position. A CSSOM View
consumer uses that operation's own scrolling-area origin and overflow
direction when mapping Layout geometry to its exposed coordinates. The
CSSOM View origin and the initial scroll position can differ.

#### Scroll snap geometry

Composition supplies transformed target border bounds in the scroll
container's physical coordinate frame. `ScrollTargetGeometryOf` applies the
target's signed scroll margins to construct its snap area. Equal edges
retain a zero extent. Inverted intervals yield an empty area. Consumers
can use this area for scroll-into-view with snapping disabled.

A `ScrollSnapContextOf` combines the snapport at zero offset, the signed
reachable range, and the scroller's flow axes and active snap axis.
Alignment roles follow the scroller's block and inline axes. When the
area exceeds the snapport along the target's own inline axis, start and
end use the target's edge directions. The aligned offsets are clamped to
the reachable range.

For an oversized axis, the covering interval intersects the offsets that
keep the snapport within the area, the reachable range, and the gap between
eligible adjacent offsets. A gap between two neighbors must exceed the
snapport extent. An absent neighbor leaves that end of the gap open.
The scrolling consumer checks visibility at the final two-axis offset
before selecting a destination. Shared boundary coordinates count as
visible. [CSS Scroll Snap 1](https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/#scroll-snap-align)

These boundaries describe the represented behavior; they do not establish broad
CSS conformance. The [browser parity guide](../tests/layout/browser_parity/README.md)
describes the checked-in evidence and its classified unsupported cases.
