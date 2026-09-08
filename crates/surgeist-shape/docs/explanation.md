# Geometry boundary and design

## Resolved geometry

`surgeist-shape` owns logical geometry, primitive shapes, path data, bounds,
containment, geometry keys, and initial UI dash geometry. Callers supply resolved
numbers and receive geometry without introducing layout decisions, style
resolution, GPU resources, rendering, widgets, or application behavior into this
crate. This boundary is stated in [the public entry point](../src/lib.rs).

This workspace crate owns its domain source and focused tests. The root `surgeist`
repository owns public composition, cross-crate lowering and adapters, integration tests,
workspace wiring, and generated API audit artifacts. See the
[repository ownership guide](../AGENTS.md). The crate's Kurbo dependency supplies
geometry operations and conversion types; sibling Surgeist internals are not
part of this public surface.

## Construction and validation

The modeling contract places validation at public construction boundaries:
finite coordinates, nonnegative dimensions and radii, valid path ordering,
stroke geometry, and dash geometry. [Numeric wrappers](../src/value.rs) and
[geometry constructors](../src/geometry.rs) reject nonfinite values and apply
the appropriate sign constraints. Insets may be negative; size and radius
constructors reject negative values.

A [path builder](../src/path.rs) accumulates commands before `build()` validates
them. A segment or close command requires a preceding move, and control points
must be finite. An empty path is explicitly valid empty geometry. This ordering
check does not require a closed contour or prohibit self-intersection.

[Stroke](../src/stroke.rs) construction checks width and miter limit;
[dash](../src/dash.rs) construction checks density, phase, side selection, and
anchor capacity. Shape operations also perform their operation-specific checks
and return typed errors for the unsupported cases listed in the
[reference](reference.md).

Input validation is not a guarantee against every later floating-point overflow.
Some fallible operations revalidate calculated values, while accessors and
internal arithmetic can return derived values without that check. For example,
rectangle maximum coordinates add origin and size, and circle bounds double
the radius. These finite inputs can still exceed the `f64` range.

## Normalization and identity

[Radius normalization](../src/geometry.rs) applies one factor, at most one, to
all four radii so adjacent corners fit their rectangle. A rounded rectangle
retains the supplied radii; path conversion, containment, and key generation
normalize them when needed. Consequently, different supplied radii can yield
the same shape key when they normalize to the same geometry.

[Keys](../src/key.rs) hash the geometry data. Path keys include the fill rule,
so identical commands with `NonZero` and `EvenOdd` produce different keys.
The [focused tests](../src/tests.rs) exercise these distinctions. A key is a
geometry identifier; it is not a serialization format or a cryptographic digest.

## Bounds and curve approximations

[Visual bounds](../src/primitive.rs) expand source bounds by half the centered
stroke width, none for inside alignment, or the full outside width. They do not
compute an outline from line joins, caps, or individual dashes. Transformed
bounds enclose the transformed source rectangle, which can be larger than the
tight bounds of the transformed shape. Support bounds apply explicit insets
supplied by the caller.

Primitive path conversion uses Kurbo with the internal path tolerance `0.1` in
[src/lib.rs](../src/lib.rs). [Dash generation](../src/dash.rs) separately uses
polylines: rounded-corner arc sampling uses an internal step count of `8`, and
ellipse sampling uses between `32` and `192` points based on radius. Render
points and inset measurement points let dash placement account for stroke
alignment while distributing marks along those sampled contours.

These are current approximation choices, not exact curve-length guarantees.
`DashSegment::contour_length()` measures line segments and endpoint chords for
quadratic or cubic commands; `DashGeometry::contour_length()` sums its emitted
segments rather than the entire underlying contour. Arbitrary-path inflation,
dashing, and inside/outside stroke bounds remain unsupported.
