# Reference

## Package and public surface

[Cargo.toml](../Cargo.toml) declares package `surgeist-shape` version `0.1.0`,
library import name `surgeist_shape`, Rust edition `2024`, and dependency
`kurbo = "=0.13.1"`. It declares no crate features or `rust-version`.

All public types are reexported from [src/lib.rs](../src/lib.rs); implementation
modules are private.

| Area | Public types | Source |
| --- | --- | --- |
| Numeric validation | `Finite`, `NonNegative`, `NumericKind` | [value.rs](../src/value.rs) |
| Logical geometry | `Point`, `Size`, `Rect`, `Insets`, `Radii`, `Transform` | [geometry.rs](../src/geometry.rs) |
| Primitive shapes | `Shape` | [primitive.rs](../src/primitive.rs) |
| Paths and fill | `Command`, `FillRule`, `Path`, `PathBuilder` | [path.rs](../src/path.rs) |
| Classified bounds | `Bounds`, `BoundsKind` | [bounds.rs](../src/bounds.rs) |
| Geometry identity | `Key` | [key.rs](../src/key.rs) |
| Stroke description | `Stroke`, `StrokeAlign`, `LineCap`, `LineJoin` | [stroke.rs](../src/stroke.rs) |
| Dash description and output | `Dash`, `DashAnchor`, `DashConstraint`, `DashGeometry`, `DashSegment`, `Side`, `SideSet`, `Corner` | [dash.rs](../src/dash.rs) |
| Errors | `Error`, `ErrorCode`, `Result` | [error.rs](../src/error.rs) |

## Shapes, paths, and bounds

`Shape` constructors accept rectangles, rounded rectangles, circles, ellipses,
and validated paths. Ellipse `Size` values are radii along the two axes.
`PathBuilder` supports move, line, quadratic, cubic, and close commands.
`FillRule` is `NonZero` or `EvenOdd`; `NonZero` is its default.

| Operation | Current behavior |
| --- | --- |
| `Shape::bounds()` | Axis-aligned source bounds; paths use Kurbo's bounding box |
| `Shape::visual_bounds(stroke)` | Source bounds with alignment-dependent uniform outset |
| `Shape::support_bounds(insets)` | Source bounds with caller-supplied finite outsets |
| `Shape::transformed_bounds(transform)` | Bounds of the four transformed source-bounds corners |
| `Shape::contains(point)` | Primitive containment or path winding under its fill rule |
| `Shape::to_path()` / `to_kurbo_path()` | Path conversion with primitive curves approximated as needed |
| `Shape::key()` / `Path::key(fill_rule)` | Geometry hash; path fill rule contributes to the key |

`BoundsKind` labels are `Source`, `Fill`, `Stroke`, `Visual`, `Support`, and
`Transformed`. A `Bounds` label does not calculate that kind of geometry;
`Bounds::union` and `intersection` retain the left operand's label.

## Stroke and dash facts

`Stroke::default()` and `Stroke::hairline()` use width `1.0`, centered alignment,
miter join, butt caps, miter limit `4.0`, and no dash. Width and miter limit must
be finite and nonnegative. `StrokeAlign` is `Center`, `Inside`, or `Outside`.

`Dash::dashed()` uses density `1.0`, phase `0.0`, all sides, no stored anchors,
and unrounded marks. Density must be finite and greater than `f64::EPSILON`;
phase must be finite. Side selection must be nonempty. Up to four corner or
nonnegative contour-offset anchors can be stored. `Dash::dotted()` enables the
`Circular` constraint and rounded marks.

Current dash generation uses side selection for rectangles and rounded
rectangles; circles and ellipses use a full closed contour. Stored user anchors
are not read by either generator. Rectangle generation derives its own corner
and side-run anchors. See [dash.rs](../src/dash.rs).

## Limits and errors

| Request | Result |
| --- | --- |
| Inflate or deflate an arbitrary path shape | `UnsupportedStrokeBounds` |
| Inside/outside visual stroke bounds for a path shape | `UnsupportedStrokeBounds` |
| Dash an arbitrary path shape | `InvalidDash` |
| Resolve dashed geometry without a dash specification | `InvalidDash` |

`Error` contains public `code: ErrorCode` and `message: String` fields and
implements `std::error::Error`. Other error codes are `NonFinite`, `NegativeSize`,
`NegativeRadius`, `InvalidPath`, `InvalidStroke`, and `EmptyPath`.
The [explanation](explanation.md) describes arithmetic and approximation limits.

The [repository guide](../AGENTS.md) owns configured verification commands and
source discovery. [src/tests.rs](../src/tests.rs) contains the focused unit tests.
