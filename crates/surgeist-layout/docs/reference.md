# Reference

This page describes the current crate interfaces and local tooling. The
[manifest](../Cargo.toml), [public crate root](../src/lib.rs), and linked source
files own their current definitions.

## Package and features

| Item | Current value |
| --- | --- |
| Package / library | `surgeist-layout` / `surgeist_layout` |
| Version | `0.1.0` |
| Rust edition / minimum version | `2024` / `1.97` |
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

- Tree layout: `compute_layout` and `compute_layout_invalidated`.
- Direct leaf measurement: `compute_leaf`.
- Host contracts: `Traverse`, `LayoutTree`, and `LayoutBatchSink`.
- Cache and failure contracts: `CacheOf`, `LayoutResultOf`, and `LayoutErrorOf`.

### Node input

Source: [node input](../src/node_input/mod.rs) and its domain modules.

- Aggregate input: `NodeInputOf` and its `NodeInput` default-scalar alias.
- Tree participation: `LayoutInputOf`, `Display`, `Position`, and `ItemOrder`.
- Flow and alignment: `WritingMode`, `Direction`, `AlignItems`, and `AlignContent`.
- Inline and flex input: `InlineTextInputOf`, `InlineBoundaryInputOf`, `FlexDirection`, `FlexWrap`, and `FlexItemCollapse`.
- Grid input: `GridPlacement`, `GridAutoFlow`, and `GridFlowToleranceOf`.
- Scroll input: `ComputedOverflow`, `OverflowClipMarginOf`, `ScrollPaddingOf`, `ScrollMarginOf`, and `ScrollSnapType`.

### Sizing

Sources: [sizing](../src/sizing.rs) and [values](../src/value.rs).

- Property domains: `PreferredSizeOf`, `MinSizeOf`, `MaxSizeOf`, and `FlexBasisOf`.
- Validated calculations: `SizingCalculationOf` and `CalcSizeCalculationOf`.
- Finite values and resolution: `LengthPercentageOf`, `PercentageBasisOf`, `LengthOf`, `LengthAutoOf`, and `AvailableOf`.

### Geometry and scroll

Sources: [geometry](../src/geometry.rs) and [scroll](../src/scroll/mod.rs).

- Physical geometry and flow mapping: `Point`, `Size`, `Edges`, and `FlowAxes`.
- Canonical scroll geometry: `ScrollGeometryOf`, `ScrollRectOf`, `PhysicalScrollRangeOf`, and `FlowRelativeScrollRangeOf`.
- Read-only scroll details: `OverflowClipOf`, `ScrollbarGutterRectsOf`, and `ScrollTargetGeometryOf`.

### Output

Source: [output](../src/output.rs).

- Root request and transaction: `LayoutRootRequestOf` and `CompletedLayoutBatchOf`.
- Node and sizing results: `NodeOutputOf`, `ComputeInputOf`, and `ComputeOutputOf`.
- Inline output: `InlineFragmentOutputOf` and `InlineFragmentOutputEntryOf`.
- Root context: `LayoutRootContextOf`, `FlexItemRootContextOf`, and `ContainingLayoutContext`.

### Finite grid utilities

Sources: [grid](../src/grid/mod.rs) and [track values](../src/value.rs).

- Lane placement: `place_lanes`, `LanePlacementInputOf`, and `LanePlacementReportOf`.
- Intrinsic lane sizing: `lane_intrinsic_sizing`, `LaneIntrinsicSizingInputOf`, and `LaneIntrinsicSizingReportOf`.
- Grid computation reports: `GridComputationOf` and `GridComputationReport`.
- Finite track construction: `TrackSizingOf`, `TrackComponentOf`, `TrackComponentListOf`, and `track_sizing_components_of`.

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

`LayoutRootRequestOf<S>` validates root input for the public
`compute_layout` front door. A successful call returns a
`CompletedLayoutBatchOf<Node, S>` containing the staged layout and cache updates;
a `LayoutErrorOf<Node, S, M>` returns no partial public result. Recursive
algorithm modes remain internal.

`compute_leaf` is the direct, fallible leaf-measurement boundary. Its provider
receives non-negative content-space constraints, and invalid provider output or a
provider error becomes a typed layout error. Resolved padding and border must be
non-negative and finite, including their combined axis sums. Overflow in
layout-owned box arithmetic returns `LayoutInvalidInputOf::InvalidNumeric`;
invalid provider dimensions retain the distinct `MeasurementOutput` diagnostic.
Valid fully known sizing requests can complete without calling the provider.

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

`ContainingLayoutContext` keeps the containing flow axes and
`ParentFormattingContext` role together as the complete containing context and
cache identity. Flex-item roots require explicit parent flow axes and keep the
host allocation in the root request separate from the viewport percentage
context in `FlexItemRootContext`.

## Inline metrics

Source: [inline input](../src/node_input/inline.rs).

`InlineMetricsOf<S>` is layout-ready line box data. Layout consumes it for inline
line construction and does not derive it from authored CSS or fonts. Integration
layers should provide metrics from computed style and text/font measurement
before constructing `LineBreakInputOf<S>`.

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
