# How to use the geometry API

These examples assume the [first-success check](getting-started.md) passes and
your Rust caller can import `surgeist_shape`. Each block is a complete function
using the public API; call it to execute its assertions.

## Construct a rounded rectangle and resolve dashes

Use finite coordinates, nonnegative dimensions and radii, and a stroke with a
dash specification. This example verifies both the source bounds and nonempty
dash output:

```rust
use surgeist_shape::{Dash, Radii, Rect, Result, Shape, Stroke};

fn rounded_rectangle() -> Result<()> {
    let rect = Rect::try_new(0.0, 0.0, 80.0, 40.0)?;
    let shape = Shape::try_rounded_rect(rect, Radii::try_all(8.0)?)?;
    let stroke = Stroke::try_centered(2.0)?.with_dash(Dash::dashed());
    let geometry = shape.dashed_stroke(stroke)?;

    assert_eq!(shape.bounds(), rect);
    assert!(!geometry.is_empty());
    assert!(geometry.segments().iter().all(|segment| segment.width() == 2.0));
    Ok(())
}
```

`Dash::dotted()` requests circular, rounded marks. For rectangles and rounded
rectangles, `Dash::dashed().with_sides(SideSet::top())?` selects the top side;
import `SideSet` to use it. See the [reference](reference.md) for shape-specific
dash limits.

## Build a path with an explicit fill rule

Start with a move command, add the segments, and consume the builder with
`build()`. An empty builder is valid empty geometry; a line before any move is
rejected when built. This closed triangle has nonempty bounds, and its fill rule
participates in its geometry key:

```rust
use surgeist_shape::{FillRule, PathBuilder, Point, Result, Shape};

fn triangle() -> Result<()> {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point::zero())
        .line_to(Point::try_new(10.0, 0.0)?)
        .line_to(Point::try_new(10.0, 10.0)?)
        .close();
    let path = builder.build()?;

    assert_ne!(path.key(FillRule::NonZero), path.key(FillRule::EvenOdd));
    let shape = Shape::try_path(path, FillRule::NonZero)?;
    assert!(!shape.bounds().is_empty());
    Ok(())
}
```

## Inspect stroke bounds and handle invalid input

For a rectangle, centered visual bounds expand each side by half the stroke
width. `Error.code` supports typed handling of a rejected value; the message
provides diagnostic context.

```rust
use surgeist_shape::{ErrorCode, Rect, Result, Shape, Stroke};

fn bounds_and_errors() -> Result<()> {
    let shape = Shape::try_rect(Rect::try_new(10.0, 20.0, 30.0, 40.0)?)?;
    let bounds = shape.visual_bounds(Some(Stroke::try_centered(8.0)?))?;
    assert_eq!(bounds, Rect::try_new(6.0, 16.0, 38.0, 48.0)?);

    let error = Stroke::try_new(-1.0).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidStroke);
    Ok(())
}
```

These bounds are axis-aligned outsets, not a computed stroke outline. Read the
[bounds explanation](explanation.md) before using them as a rendering estimate.
The examples follow the constructors and assertions in
[the focused tests](../src/tests.rs); complete signatures are mapped in the
[reference](reference.md).
