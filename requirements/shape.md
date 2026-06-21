# surgeist::shape Requirements

`surgeist::shape` is the pure resolved-geometry boundary for Surgeist. It owns reusable shape definitions, normalized geometry, bounds, containment, path conversion, and stable geometry keys for downstream render, hit-test, canvas, and effect caches.

The public API should be designed for use as `surgeist::shape::*`. The module path supplies the layer name, so public type names should stay short and local: `Point`, `Size`, `Rect`, `Radii`, `Insets`, `Transform`, `Shape`, `Path`, `Command`, `FillRule`, `Stroke`, `LineJoin`, `LineCap`, `StrokeAlign`, `Dash`, `DashAnchor`, `DashConstraint`, `DashGeometry`, `DashSegment`, `Side`, `SideSet`, `Corner`, `Bounds`, `BoundsKind`, `Key`, `Error`, and `Result`.

## Scope

This module owns:

- Shared resolved logical geometry primitives.
- Rectangles, rounded rectangles, circles, ellipses, and paths.
- Radius normalization and clamping.
- Bounds and visual overflow calculations.
- Fill containment and coarse hit testing.
- Shape inflation and deflation for built-in shapes.
- UI-level dash geometry for rectangles, rounded rectangles, circles, and ellipses.
- Conversion into backend-neutral and `kurbo` path geometry.
- Stable geometry keys and fingerprints for downstream caches.
- Headless tests for geometry correctness.

This module describes resolved geometry. It does not own GPU resources, Vello/wgpu integration, raster masks, blur algorithms, rendered shadow caches, tessellation caches, UI layout, CSS parsing, CSS units, unresolved percentages, document identity, widgets, accessibility semantics, animation timelines, or application commands.

## Dependencies

Expected direct dependencies:

```text
surgeist::shape
  -> kurbo
```

`surgeist::shape` may use `kurbo` internally and may expose explicit conversion APIs for advanced callers. Front-door APIs should use Surgeist types so higher layers do not become coupled to a rendering backend.

`surgeist::shape` must remain independent from render, Vello, `wgpu`, Parley, text, window, retained, document, widget, DSL, and app behavior. Since Surgeist foundation layers now live as modules in one crate, this boundary is enforced through module privacy, review, tests, and dependency discipline rather than separate Cargo packages.

The dependency direction should be:

```text
surgeist::ui / widgets / canvas -> surgeist::shape
surgeist::render                -> surgeist::shape
surgeist::text                  -> independent
```

Render may consume shape geometry. Shape must not call render.

## Naming

Public names are authored for the `surgeist::shape` namespace:

- `Point` is a logical 2D point.
- `Size` is a logical 2D size.
- `Rect` is a logical rectangle.
- `Insets` describes edge offsets.
- `Radii` describes resolved per-corner rectangle radii.
- `Transform` describes affine geometry transforms.
- `Shape` is the main geometry enum or opaque shape value.
- `Path` is an authored vector path.
- `Stroke` describes geometry-relevant stroke facts when calculating stroked bounds or stroke outlines.
- `Dash` describes UI-level dash distribution over built-in shapes.
- `DashAnchor` describes a required dash placement on a contour, such as a corner dash.
- `DashConstraint` describes optional dash solve constraints, such as circular dotted dashes.
- `DashGeometry` is the resolved geometry produced by UI-level dash distribution.
- `SideSet` describes included rectangle or rounded-rectangle sides for side-scoped stroke geometry.
- `Bounds` reports source, fill, stroke, visual, support, or transformed bounds.
- `Key` is a stable geometry fingerprint component for downstream caches.
- `Error` and `ErrorCode` are stable diagnostics.
- `Result<T>` is this module's result alias.

Avoid repeating `Shape` in type names when the module path already supplies it. Prefer `surgeist::shape::Path` over `surgeist::shape::ShapePath`.

## Coordinate Model

Shape uses resolved logical coordinates. Scale conversion belongs to render surfaces, windows, or caller-owned projection layers.

Unresolved style values do not belong in this crate. Values such as CSS percentages, viewport units, `auto`, layout-relative lengths, inherited keywords, and authored strings should be resolved by style or layout before constructing `surgeist::shape` values.

```rust
pub struct Point { pub x: f64, pub y: f64 }
pub struct Size { pub width: f64, pub height: f64 }
pub struct Rect { pub origin: Point, pub size: Size }
pub struct Insets { pub top: f64, pub right: f64, pub bottom: f64, pub left: f64 }
pub struct Radii { pub top_left: f64, pub top_right: f64, pub bottom_right: f64, pub bottom_left: f64 }
pub struct Transform(/* affine representation */);
```

Rules:

- Geometry values must be finite.
- Sizes must be non-negative unless a method explicitly accepts signed deltas.
- Radii must be finite and non-negative.
- Normalized radii must fit the rectangle using browser-compatible proportional clamping.
- Methods must document whether they return source bounds, fill bounds, stroke bounds, visual bounds, support bounds, or transformed bounds.
- Shape construction should reject malformed geometry rather than silently producing invalid paths.
- `Rect` should not use an ambiguous `Default` as a meaningful geometry value unless a later implementation can prove that default is unambiguous. Prefer explicit constructors and constants.
- Accumulating bounds should have an explicit empty sentinel, such as `Bounds::empty()` or `Rect::empty()`, so callers can union geometry without inventing invalid rectangles.

## Bounds Model

Bounds names must be precise because higher layers will use them for invalidation, hit testing, clipping, support-space calculations, and diagnostics.

```rust
pub struct Bounds {
    pub rect: Rect,
    pub kind: BoundsKind,
}

pub enum BoundsKind {
    Source,
    Fill,
    Stroke,
    Visual,
    Support,
    Transformed,
}
```

Meanings:

- `Source` is the authored geometry before stroke, transform, or caller-supplied support outsets.
- `Fill` is the filled outline for a shape.
- `Stroke` includes the geometry occupied by a stroke.
- `Visual` includes visible geometry produced by fill and stroke.
- `Support` includes caller-requested geometric support space around a shape, expressed as explicit outsets.
- `Transformed` is a bound after applying a transform.

Shape does not compute renderer-owned effect bounds by itself. It may provide generic geometric support helpers that accept explicit outsets. Effect, render, and style layers decide which outsets are required for shadows, blurs, filters, or other visual operations.

## Core API

```rust
pub enum Shape {
    Rect(Rect),
    RoundedRect { rect: Rect, radii: Radii },
    Circle { center: Point, radius: f64 },
    Ellipse { center: Point, radii: Size },
    Path { path: Path, fill_rule: FillRule },
}

impl Shape {
    pub fn rect(rect: Rect) -> Self;
    pub fn rounded_rect(rect: Rect, radii: Radii) -> Self;
    pub fn circle(center: Point, radius: f64) -> Self;
    pub fn ellipse(center: Point, radii: Size) -> Self;
    pub fn path(path: Path, fill_rule: FillRule) -> Self;

    pub fn validate(&self) -> Result<()>;
    pub fn bounds(&self) -> Rect;
    pub fn visual_bounds(&self, stroke: Option<Stroke>) -> Result<Rect>;
    pub fn support_bounds(&self, outset: Insets) -> Result<Rect>;
    pub fn transformed_bounds(&self, transform: Transform) -> Rect;
    pub fn contains(&self, point: Point) -> bool;
    pub fn inflate(&self, amount: f64) -> Result<Self>;
    pub fn deflate(&self, amount: f64) -> Result<Self>;
    pub fn to_path(&self) -> Result<Path>;
    pub fn key(&self) -> Key;
}
```

`Shape` is pure data plus deterministic geometry operations. It should be cheap to clone and easy to compare in tests.

`Shape::bounds` returns source bounds. Methods that include stroke, support space, or transform must say so in their names.

## Rectangles And Radii

Rounded rectangles are first-class because most UI surfaces are boxes.

```rust
impl Radii {
    pub const fn all(radius: f64) -> Self;
    pub const fn new(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Self;
    pub fn is_uniform(&self) -> bool;
    pub fn normalized_for(self, rect: Rect) -> Result<Self>;
    pub fn inset(self, amount: f64) -> Self;
    pub fn outset(self, amount: f64) -> Self;
}
```

Rules:

- `Radii::normalized_for` should proportionally reduce radii when adjacent corners exceed the available width or height.
- `Radii` values are resolved logical lengths, not CSS percentages or viewport-relative values.
- Radius normalization must be deterministic and tested for asymmetric cases such as `0, 0, 5, 10`.
- Insets and outsets must never create negative radii.
- Rectangle inflation/deflation should update radii in a predictable way when used for stroke alignment or shadow spread.
- Render should not duplicate radius-clamping logic once this crate exists.

## Paths

`Path` is an authored vector outline. It should be expressive enough for imported SVG-like paths and app-authored custom shapes.

```rust
pub struct Path {
    // private command storage
}

pub enum Command {
    MoveTo(Point),
    LineTo(Point),
    QuadTo { control: Point, end: Point },
    CubicTo { control_a: Point, control_b: Point, end: Point },
    Close,
}

impl Path {
    pub fn new() -> Self;
    pub fn move_to(&mut self, point: Point) -> &mut Self;
    pub fn line_to(&mut self, point: Point) -> &mut Self;
    pub fn quad_to(&mut self, control: Point, end: Point) -> &mut Self;
    pub fn cubic_to(&mut self, control_a: Point, control_b: Point, end: Point) -> &mut Self;
    pub fn close(&mut self) -> &mut Self;
    pub fn commands(&self) -> &[Command];
    pub fn validate(&self) -> Result<()>;
    pub fn bounds(&self) -> Rect;
    pub fn contains(&self, point: Point, fill_rule: FillRule) -> bool;
    pub fn key(&self, fill_rule: FillRule) -> Key;
}

pub enum FillRule {
    NonZero,
    EvenOdd,
}
```

Rules:

- Path commands must be finite.
- A path must begin with `move_to` before drawing segments.
- Empty paths are allowed as data but should report empty bounds and should not be considered drawable.
- Bounds should use curve-aware bounds, not only control-point bounds, when practical.
- Path command storage should remain inspectable for tests and advanced callers without requiring mutation.
- Exact path boolean operations, offsetting arbitrary paths, and general path simplification are not first-pass requirements.

## Stroke Geometry

The shape module may define geometry-facing stroke facts so bounds and hit-test logic can be shared. Rendering-specific stroke lowering still belongs in `surgeist::render`.

```rust
pub struct Stroke {
    pub width: f64,
    pub join: LineJoin,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub miter_limit: f64,
    pub dash: Option<Dash>,
    pub align: StrokeAlign,
}

pub enum StrokeAlign {
    Center,
    Inside,
    Outside,
}
```

Rules:

- `Stroke` in this crate is for geometric facts such as visual bounds and containment.
- Render may define its own stroke type, convert from this one, or re-export this one if the API remains aligned.
- Built-in shapes should support deterministic stroke bounds for inside, center, and outside alignment.
- Arbitrary path offsetting is not a first-pass requirement. Unsupported stroked-path bounds must be explicit.
- Stroke paint, gradients, opacity, and render quality are not geometry facts and must not be added to this type.

## UI Dashes

Generic dash arrays are not enough for polished UI borders. Rectangle, rounded-rectangle, circle, and ellipse dashed strokes need contour-aware distribution so dashes begin from visually important anchors, bend through corners when appropriate, and avoid clipped partial dashes.

The shape module owns this reusable geometry. Style owns authored intent such as dashed or dotted stroke style. Render owns painting the resolved dash paths.

```rust
pub struct Dash {
    // storage is private; callers use builder methods and accessors
}

impl Dash {
    pub fn dashed() -> Self;
    pub fn dotted() -> Self;
    pub fn with_density(self, density: f64) -> Self;
    pub fn with_phase(self, phase: f64) -> Self;
    pub fn with_sides(self, sides: SideSet) -> Self;
    pub fn rounded(self) -> Self;
    pub fn circular(self) -> Self;
    pub fn with_corner_anchors(self) -> Self;
    pub fn with_anchor(self, anchor: DashAnchor) -> Self;
    pub fn density(self) -> f64;
    pub fn phase(self) -> f64;
    pub fn sides(self) -> SideSet;
    pub fn is_rounded(self) -> bool;
    pub fn anchors(&self) -> &[DashAnchor];
}

pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct SideSet {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

pub enum DashAnchor {
    Corner(Corner),
    ContourOffset(f64),
}

pub enum DashConstraint {
    Circular,
}

pub struct DashSegment {
    pub path: Path,
    pub width: f64,
    pub rounded: bool,
}

pub struct DashGeometry {
    pub segments: Vec<DashSegment>,
}

impl Shape {
    pub fn dashed_stroke(&self, stroke: Stroke) -> Result<DashGeometry>;
}
```

Rules:

- Dash geometry is resolved geometry, not style. It must not include color, opacity, gradients, or paint order.
- Style, CSS, or DSL layers own per-side style accumulation. Shape only receives an already-resolved `SideSet` and dash geometry facts.
- `Dash::density` is a visual density control, not a literal dash length or gap length.
- The resolved dash length, whitespace, and dash count are derived from the shape contour, corner geometry, stroke width, anchors, constraints, and density.
- Dash distribution should operate over the shape's stroked contour, not as four unrelated line segments.
- `Dash::sides` constrains which rectangle or rounded-rectangle side spans may emit dash geometry. Omitted sides produce no dash output.
- Rectangles, rounded rectangles, circles, and ellipses should use the same dash-and-gap distribution model wherever practical.
- Rectangular and rounded-rectangular strokes should evaluate each corner from its adjacent included sides.
- If both adjacent sides are included, the corner gets a full dash centered on the corner apex.
- If only one adjacent side is included, the corner gets a half dash constrained to that side's contour span. Square-corner half dashes end at the apex; rounded-corner half dashes follow the arc and end at the apex.
- If neither adjacent side is included, the corner emits no dash geometry.
- After required corner dashes are placed, the remaining spans between corner dashes should be solved with equal dash length and proportional whitespace.
- All dashes in one stroke should use the same resolved dash length unless the contour is too small to satisfy the required anchors. The resolved dash length must come from the geometry algorithm, not from a hard-coded author length.
- Rounded dashes should be supported as first-class dash geometry so the same solver can produce soft-ended dashes and dotted strokes.
- `DashConstraint::Circular` requires every resolved dash to be a consistent circle. This is the dotted-stroke helper constraint: the algorithm should solve dash length from stroke width and contour geometry so each dash renders as a circle rather than treating dots as a separate primitive.
- `DashConstraint::Circular` should not emit half circles for one-sided corners in the first implementation. If both adjacent sides are included, emit a full circular corner dash. If only one adjacent side is included, omit the corner dot and solve the side span from the side endpoint. A later revision may add semicircle rules if they prove visually useful.
- Distribution must avoid clipped terminal dashes. If the available contour length cannot fit the density intent and constraints, the algorithm must deterministically reduce count, adjust gaps, resolve a smaller shared dash length, or report an explicit error.
- Circles and ellipses must distribute dashes around the contour with stable phase and no accidental seam artifact.
- A circle can be treated as a rounded rectangle whose corner radii consume the sides. Its anchors may come from cardinal extrema or from caller-provided contour offsets.
- Rounded-rectangle and ellipse distribution may require arc-length measurement, numerical integration, or curve subdivision. Approximation tolerances must be deterministic and documented.
- The algorithm may need to solve for an integer dash count and a shared dash length from available contour length, anchor reservations, constraints, and desired density. This is preferable to exposing fragile literal dash lengths to UI authors.
- Side information may be reported for diagnostics or hit testing, but included side boundaries should only constrain output where `Dash::sides` excludes adjacent geometry.
- Dash geometry must be deterministic under scale-independent logical coordinates.
- Dash output must be inspectable in tests without a renderer.
- Existing `des-document` dashed and dotted border work is useful precedent: preserve important anchors, distribute gaps to fill the available length, and test cutoff avoidance. It was incomplete because it was egui paint code, treated sides and corners too separately, used simplified corner handling, and did not comprehensively model rounded corners, circles, ellipses, or contour correctness.
- Raw renderer dash arrays remain a render concern. Shape-level `Dash` is for polished border/outline geometry where visual distribution matters.

## Keys And Caching

`surgeist::shape` does not own caches. It only provides stable geometry keys for downstream caches.

```rust
pub struct Key(/* stable hash or structured fingerprint */);

impl Shape {
    pub fn key(&self) -> Key;
}

impl Path {
    pub fn key(&self, fill_rule: FillRule) -> Key;
}
```

Rules:

- Shape keys must include normalized geometry, path commands, path fill rule, and versioning needed for stable invalidation.
- Shape keys must not include render-only facts such as paint, blur radius, scale factor, backend options, or GPU resource identity.
- Render builds artifact keys by combining shape keys with render-specific facts.
- App and effect crates may use shape keys as one component in their own cache keys.
- Shape keys must be deterministic across process runs for the same geometry.

Example downstream render cache:

```text
shadow artifact key =
  shape key
  + shadow offset
  + shadow blur
  + shadow spread
  + shadow paint
  + surface scale
  + renderer quality options
```

## Conversion

The crate should provide explicit conversions into `kurbo` for render, effects, and test code. `kurbo` is the lower-level vector geometry escape hatch, not the default app-facing vocabulary.

```rust
impl Shape {
    pub fn to_kurbo_path(&self) -> Result<kurbo::BezPath>;
    pub fn to_kurbo_rect(&self) -> Option<kurbo::Rect>;
    pub fn to_kurbo_rounded_rect(&self) -> Option<kurbo::RoundedRect>;
}
```

Rules:

- Conversion methods should be explicit so callers know when they are losing shape semantics.
- Built-in shapes should retain their semantic form for as long as possible.
- `surgeist::render` may use semantic forms for fast paths before converting to generic paths.
- Front-door APIs should not force ordinary UI, widget, or DSL code to name `kurbo` types.

## Future Extensions

Complex UI shapes such as callouts, popovers with arrows, notched panels, and graph-node silhouettes are valid future extensions, but they should not drive the first crate implementation.

Future shape families should follow the same rules:

- Pure geometry only.
- Deterministic normalization.
- Bounds and containment.
- Path conversion.
- Stable geometry keys.
- No renderer caches or GPU state.

First implementation should prioritize simple, well-tested primitives over broad shape coverage.

## Test Contract

Required tests:

- Rejects non-finite points, sizes, radii, and path commands.
- Normalizes asymmetric radii deterministically.
- Calculates bounds for rectangles, rounded rectangles, circles, ellipses, and paths.
- Calculates transformed bounds for simple transforms.
- Performs containment for built-in filled shapes.
- Converts built-in shapes to paths.
- Produces stable keys for equal geometry.
- Produces different keys for different geometry.
- Keeps shape keys independent from render-only facts.
- Documents unsupported arbitrary path offsetting clearly.
- Resolves dashed rectangle strokes from corner anchors without clipped terminal dashes.
- Resolves circular dotted dash constraints from corner anchors without clipped terminal dashes.
- Resolves side-scoped rectangle and rounded-rectangle dashes, including full corners for adjacent included sides and half dashes for one-sided non-circular corners.
- Omits one-sided circular corner dots until a later semicircle rule is deliberately added.
- Derives resolved dash length and whitespace from contour length, stroke width, anchors, constraints, and density rather than hard-coded dash lengths.
- Resolves circle and ellipse dashes with stable seam placement.
- Keeps dash geometry independent from paint, renderer scale, and GPU state.
