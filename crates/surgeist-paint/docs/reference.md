# Reference

The [manifest](../Cargo.toml) owns dependencies and Rust compatibility.
The [public front door](../src/lib.rs) exports these surfaces:

| Surface | Contract |
| --- | --- |
| `resolve_border_image`, `BorderImageRequest`, `BorderImageGeometry` | Slice, outset, and width geometry from a concrete source and a supplied border box. |
| `resolve_border_image_area`, `BorderImageArea`, `resolve_border_image_in_area` | Compute the outset area before source sizing and reuse it for partitioning. |
| `resolve_border_image_tiling`, `BorderImageTiling` | Scale and position tiles from the admitted nine-slice geometry. |
| `BoxId`, `BoxFacts` | Rendering-box slots and explicit resolved topology/context facts. |
| `ChildOrder`, `Participation`, `Position`, `ZIndex`, `ContextEffect` | Shared source/flex/grid traversal and actual-context classification facts. |
| `plan_stacking`, `PaintOutcome`, `StackingPlan`, `ContextView` | Context membership and ordered immediate child contexts, or typed failure. |
| `BoxPaint`, `BoxPaintKind`, `FloatParticipation` | Same-slot resolved box categories and float participation. |
| `TextSource`, `PaintLine`, `PaintFragment`, `BlockInInlinePlacement` | Logical sources and exact line/fragment occurrences with explicit promotion placement. |
| `TextId`, `LineId`, `FragmentId`, `PrimitiveId` | Distinct request-local table slots, without global provenance. |
| `PaintPrimitive`, `PrimitiveKind`, `CanvasPaintKind`, `FragmentPaintKind`, `DecorationKind` | Resolved appearance roles and local record order. |
| `PaintRequest`, `plan_paint_order`, `PaintOrderOutcome`, `PaintOrder` | Borrowed resolved input and complete supported order, or typed failure. |
| `PaintOrderEvent`, `BoxView`, `LineView`, `FragmentView`, `PrimitiveView` | Actual-context boundaries and exact borrowed source-record observations. |
| `PaintError`, `PaintInputError`, `BoxCategoryError` | Planning/accounting failures and typed input/category/unsupported-profile diagnostics. |
| `PaintLimits`, `PaintOptions`, `PaintReport`, `PaintPhase` | Limits, clock/profiling policy and named Ledger usage access. |
| `resolve_linear_gradient`, `LinearGradientDirection`, `LinearGradientLine`, `LinearGradientOutcome` | Concrete box-local physical line geometry, length and optional accepted-prefix report. |
| `GradientGeometryError` | Invalid dimensions/angle or unrepresentable positive line, wrapped by `PaintError::Gradient`. |
| `GradientPosition`, `GradientStopPositions`, `GradientStopItem<C>` | Raw affine positions and native source records with opaque stop payloads or distinct hints. |
| `resolve_gradient_stops`, `GradientStopsOutcome`, `GradientStops`, `UsedGradientItem` | Complete ordered used positions with exact source association, or typed failure and optional report. |
| `GradientStopsError`, `GradientPositionComponent` | List/length/component/numerical diagnostics with source indices and expanded-position ordinals, wrapped by `PaintError::GradientStops`. |
| `resolve_radial_gradient`, `RadialGradientSize`, `RadialGradientExtent` | Concrete box/used-center request with coupled circle/ellipse size and default ellipse/farthest-corner. |
| `RadialGradientGeometry`, `RadialGradientCase`, `RadialGradientOutcome` | Finite center and positive ellipse axes or ordered ZeroCircle/ZeroWidth/ZeroHeight, plus optional report. |
| `RadialGradientError` | Native dimension/center/radius and numerical diagnostics, using Layout `PhysicalAxis` where applicable and wrapped by `PaintError::RadialGradient`. |
| `resolve_zero_height_gradient_colors`, `GradientRepeat`, `ZeroHeightGradientColorsOutcome`, `ZeroHeightGradientColors`, `WeightedGradientColor` | Authenticated ordinary/average color measure for the native ZeroHeight case, borrowing exact source payloads. |
| `ZeroHeightGradientError` | Expected-case or normalized color-measure representation failure, wrapped by `PaintError::ZeroHeightGradient`. |

## Ordinary gradient stop views

`resolve_gradient_stops` validates its source, resolves and qualifies numerical
positions, then prepares Stop/Hint records with direct borrowed payloads and
captured source identities. The construction owner creates and advances its own
staging. `first_stop()` borrows the mandatory typed `UsedGradientStop`;
`tail()` borrows the remaining records, including a first Two's second occurrence.
`records()` borrows the whole list as `GradientRecord` views. Obtaining a view
does not inspect the prepared fields: a consumer admits its visit before
`item()` projects the record. `items()` remains a whole-list projection of this
same inline head and tail, without source indexing or interpreting source kinds
again. No prepared record is stored twice. `source()` retains the exact original slice;
the result and its views cannot outlive that source.

## Border-image geometry

`BorderImageRequest` takes the border-box origin, size, and computed border
widths in CSS pixels. The source carries its concrete coordinate extent and
independently optional natural CSS-pixel dimensions. A raster slice can therefore
refer to decoded image pixels while an automatic destination width uses the
corresponding natural CSS-pixel extent. When that natural axis is absent, the
automatic width uses the supplied border width.

The outset paint area supplies the percentage basis for destination widths.
For a source that first needs sizing, call `resolve_border_image_area` with a
`BorderImageAreaRequest` and supply the resulting `default_size()` to the
sizing owner. The extent is calculated from the box size and outsets before
placement can round the edges. Pass that same `BorderImageArea` and the
concrete `BorderImagePartitionRequest` to `resolve_border_image_in_area`.
The resolved area retains the border-width basis for multiples and automatic
fallbacks. For an already-sized source, `resolve_border_image` performs both
calculations under one budget.

One common factor fits opposing widths to the area. The nine output regions
are in physical top-to-bottom, left-to-right order and distinguish independent
source and destination emptiness. An overlapping source axis removes its middle
span while preserving the corner slices. Center source geometry is supplied
only when fill is requested. Local axis geometry determines whether the middle
is empty before the cuts are placed. A positive partition that collapses in
the output coordinates returns a typed numeric error. The resulting paint area
is separate from layout, scrolling, and pointer geometry.

Ledger reports admitted `BorderImageValidation`, `BorderImageCoordinate`, and
`OutputRecord` work. The staged area reports its coordinate work in
`BorderImageArea` and publishes one output record. The partition operation
charges each region and the complete result. Each operation accounts for its
own fixed retained result and releases provisional custody on refusal.

## Border-image tiling

`resolve_border_image_tiling` consumes the admitted partition and horizontal
and vertical repeat choices. Corners stretch to their destination rectangles.
Edges fit the cross-axis thickness and use the same proportional scale along
the edge before repetition. The filled center uses the top and left scaling
bases, falling back to bottom and right respectively. When neither edge
supplies a basis, one source coordinate maps to one CSS pixel on that axis.
Empty source or destination regions emit no plan.

Each drawable region carries its source rectangle, destination clip, and two
patterns local to the clip minimum. Stretch emits one tile. Repeat centers
a tile and extends in both directions to the clip. Its anchor represents
that centered pattern modulo one tile. Round chooses the nearest positive
whole count, with half ties toward the higher count, then divides the clip
extent evenly. Stretch and Round use the clip endpoints for the first and
final boundaries of the subdivision.

Space retains only whole tiles and records the exact uncovered extent.
The equal gap is `uncovered / (count + 1)` before, after, and between the
tiles. The start of tile `i` in real arithmetic is
`i * extent + (i + 1) * uncovered / (count + 1)`. This representation
retains equal gaps even when that quotient rounds in binary64. Space omits
a region when a tile cannot fit on an axis using that mode. Finite counts
retain distinguishable binary64 indices and an exactly representable extra
gap count. Unrepresentable arithmetic returns the typed border-image
geometry error.

Ledger admits coordinate actions and emitted region records in
`BorderImageTiling` and accounts for the fixed plan at publication. Source
sizing, resource eligibility, composited order, and concrete draw lowering
use their respective owners.

[CSS Backgrounds and Borders Level 3, drawing the border image](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-process)

## Zero-height gradient colors

`resolve_zero_height_gradient_colors` consumes authentic
`RadialGradientGeometry` in the `ZeroHeight` case, raw `GradientStopItem<C>`
records, a `GradientRepeat` selection and `PaintOptions`. It returns
`ZeroHeightGradientColorsOutcome`, with a complete borrowed color measure or
typed error and the existing optional report. `source()` retains the exact
source association. `colors()` yields `WeightedGradientColor` views with a
borrowed payload, source index, Two-position ordinal and finite nonnegative
weight. Payloads remain opaque to Paint.

`selected_last_color()` borrows the actual Last record as a
`WeightedGradientRecord`; `None` means Average, rather than an absent endpoint.
`records()` borrows the selected measure's ordered weighted records, including
zero weights. Admit each consumer visit before `color()` projects its fields.
`colors()` uses the same projection and backing. These borrowed operations add
no allocation, source read, stored flag, or producer work; Measure storage and
its weighting and custody remain unchanged.

Construction captures each color's payload and first/second-position identity
from its authored stop. The finished measure retains these associations;
`colors()` projects prepared records without reading or interpreting source
items again. Repeating weighting starts from a qualified adjacent color pair.

`GradientRepeat::None` selects the last actual color at weight one. A final Two
uses ordinal one. `GradientRepeat::Repeat` retains all expanded color occurrences
and their average weights; hints remain separate positional inputs. Repetition
is consumed into the private result state. Consumers use that authenticated
measure without choosing its meaning again. A singleton needs no average vector.

The selected [native basis policy](https://github.com/bj-data-eng/surgeist/issues/2100)
takes the substituted horizontal width toward positive infinity. This follows
the [current radial substitution](https://drafts.csswg.org/css-images-3/#degenerate-radials)
without retaining an unused radius or choosing a large finite substitute.
The text leaves mixed-unit fixup ordering implicit;
[historical editor intent](https://lists.w3.org/Archives/Public/www-style/2011Aug/0379.html)
instead retains the original width. The selected limit is explicit native policy,
rather than a claim of standards consensus or browser equivalence.

Treat each affine position as `(fraction, length)`, ordered lexicographically
as width grows. Apply endpoint defaults and positioned stop/hint fixup to those
pairs. If endpoint fractions differ, their fractions determine the relative
measure; otherwise lengths determine it. Normalize required coordinates before
distributing omitted colors, including positioned hint boundaries. This retains
length weights when all percentages agree and avoids overflowing raw spans or
unrepresentable tiny distributed positions. An irrelevant hint's hypothetical
normalization loss cannot refuse an otherwise representable color measure.

The [specified repeating average](https://drafts.csswg.org/css-images-3/#gradient-average-color)
uses adjacent-color trapezoids in premultiplied sRGBA. For red at zero, green at
10 absolute units and blue at 100%, the chosen limit has weights `(0, .5, .5)`.
Equal-percentage positions with length contributions `0, 10, 40` instead retain
weights `(.125, .5, .375)`. An identically zero whole span uses uniform replacement
spacing over actual expanded colors; hints add no color measure.

`PaintError::ZeroHeightGradient` carries `ZeroHeightGradientError`.
`ExpectedZeroHeight` precedes source reads for other geometry cases. Existing
`GradientStops` diagnostics retain malformed-list, component and source/ordinal
details. `UnrepresentableWeight` names the pair whose required normalized color
interval or weight loses its representation. Ordinary nonzero f64 rounding is
allowed; numerical loss never selects another semantic average. Both-zero
ellipses remain `ZeroWidth` and are outside this operation.

The result borrows its source; its generic weights can be consumed by a color
interpolation owner. Render provides the optional
[`prepare_zero_height_gradient_color`](../../surgeist-render/docs/reference.md#zero-height-paint-colors)
consumer for numeric-sRGBA `Color`. Its owned solid output outlives the measure,
source and preparation contexts.

## Accounting schema

The operations share 20 work dimensions and two byte dimensions. Actions count
algorithm work, not CPU instructions or elapsed time.

| Work family | `WorkKind` dimensions |
| --- | --- |
| Shared stacking and output | `BoxValidation`, `ParentEdge`, `TopologyVisit`, `SiblingComparison`, `MembershipVisit`, `ContextComparison`, `OrderMove`, `OutputRecord` |
| Paint input and sequencing | `PaintValidation`, `FragmentVisit`, `LineVisit`, `InlineMemberVisit`, `PrimitiveVisit`, `PaintScopeVisit`, `PaintOrderComparison`, `DecorationComparison` |
| Gradient geometry | `GradientValidation`, `GradientDirection`, `GradientCoordinate` |
| Gradient stop lists | `GradientStopVisit`, plus shared validation, coordinate and output dimensions |

`ResourceKind` is `ScratchBytes` or `RetainedResultBytes`.
Fixed child phases are `validate`, `topology`, `sibling_order`, `membership`,
`context_order`, `publication` and `cleanup`. `plan_stacking` has these seven
children under the `paint stacking` root: eight scopes in total.
`plan_paint_order` adds `paint_validate` and `paint_sequence` under the
`paint order` root: nine children and ten scopes in total. It reuses stacking
inside that single operation/report. Phase usage is inclusive; root totals and
child attribution must not be summed together.

`resolve_linear_gradient` uses gradient validation, geometry and publication
attribution. Validation counts each input scalar or direction discriminant;
direction work counts actual resolution actions. Coordinate work counts actual
scalar geometry calculations and inspections, including repeated work. A successful
operation admits one output record and the fixed `LinearGradientLine` result's
size as retained bytes. It allocates no geometry buffer; caller input and
Ledger's bounded observer remain outside these byte dimensions. See
[the implementation](../src/gradient.rs) for the precise charged action units.

`resolve_radial_gradient` uses the existing gradient validation/geometry,
publication and cleanup child phases under its own root, five scopes total.
It reuses `GradientValidation`, `GradientCoordinate` and `OutputRecord`; actual
bounded binary-representation proof actions are included in coordinate work.
It retains only the fixed result owner's actual size, with no geometry buffer
or scratch allocation. See [the implementation](../src/radial_gradient.rs) for
the precise charged scalar units.

`resolve_gradient_stops` uses the `paint gradient stops` root with six child
phases: validation, stop resolution, prefix fixup, omitted distribution,
publication and cleanup. `GradientStopVisit` counts each actual source or
expanded-record visit, including repeated passes and boundary inspection.
`GradientCoordinate` counts actual numeric calculations/conversions/inspections;
`GradientValidation` counts input domain/grammar actions. Each expanded numerical
record, prepared Stop/Hint record and final fixed result owner contributes an
`OutputRecord`. After numerical qualification, preparation reuses the first
source Stop's occurrences captured during charged validation. With S source
items and R expanded records, publication performs S-1+R visits: each later
source item and each numerical record once. It previously performed S+R visits;
numeric work and R prepared emissions are unchanged. Numerical staging uses
actual `ScratchBytes`; retained custody is the actual tail capacity multiplied
by `size_of::<UsedGradientItem<C>>()`, plus `size_of::<GradientStops<C>>()` for
the complete fixed owner containing its inline typed head. The tail reserves
R-1 items; a single-stop result needs no prepared heap storage. These actual
sizes replace the former full-vector capacity and smaller fixed owner, so byte
thresholds and refusal prefixes can change. Both numerical and prepared backings
coexist during preparation, and cleanup releases staging before return. Source
and payload storage remain caller-owned. See
[the implementation](../src/gradient_stops.rs) for the precise charged units.

`resolve_zero_height_gradient_colors` uses the `paint zero-height gradient
colors` root and the same six stop-resolution child phases. It shares the
validation, source/record-visit and scalar-action units above. Distribution
includes normalization, color qualification and weight construction. Scratch
charges actual affine, normalized and qualified record capacities. Retained
bytes cover the fixed result owner, including its inline color records, and
the actual capacity of any remaining color-weight records. Last selection
retains only that fixed owner.
Publication is atomic, and failure cleanup releases every admitted capacity.
Borrowed source and payload storage remain outside custody. See
[the projection](../src/zero_height_gradient.rs) for the charged actions.

## Source map

[Accounting](../src/accounting.rs) owns the named schema and capacity admissions.
[The context algorithm](../src/algorithm.rs) owns shared classification,
traversal and fallible sorting; [stacking models](../src/model.rs) own context
views. [Paint-order models](../src/paint_order/model.rs) own request records,
typed input errors and borrowed order views. [Validation](../src/paint_order/validation.rs)
owns cross-table incidence; [sequencing](../src/paint_order/sequencing.rs) owns
paint bands and recursive scope ordering. The [operation](../src/paint_order/mod.rs)
composes these under one report.
[Gradient geometry](../src/gradient.rs) owns physical direction resolution,
numerical validation and the line result; its
[public tests](../tests/linear_gradient.rs) use real Layout values and independent
coordinate, budget and profiling expectations.
[Shared stop resolution](../src/gradient_stops.rs) owns expansion, affine
positions, ordered fixup, omitted runs and borrowed result views;
[public stop tests](../tests/gradient_stops.rs) exercise its values, associations
and accounting through the front door.
[Radial geometry](../src/radial_gradient.rs) owns size resolution, finite
representation and ordered case selection;
[public radial tests](../tests/radial_gradient.rs) cover geometry, cases,
composition and accounting.
[Zero-height colors](../src/zero_height_gradient.rs) compose the shared stop
engine into a borrowed color measure; [public color tests](../tests/zero_height_gradient.rs)
cover mixed units, normalization, hints, source identity and accounting.

[Stacking tests](../src/algorithm/tests.rs), [paint-order tests](../src/paint_order/tests.rs)
and [accounting tests](../src/paint_order/accounting_tests.rs) supply literal order,
input-refusal and work/storage-boundary expectations. Public producer composition
is covered by [Layout](../tests/paint_order_layout.rs) and
[native inline](../tests/paint_order_native_inline.rs) tests.
