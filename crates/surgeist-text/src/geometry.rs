/// Outline axes of the glyphs in an admitted uniform run.
///
/// Glyph offsets use font x to the right and converted font y downward. The
/// logical glyph pen follows the run's inline axis independently of these axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphBasis {
    /// Glyph outlines keep their original axes.
    Unrotated,
    /// Glyph outlines rotate clockwise by ninety degrees.
    Clockwise,
    /// Glyph outlines rotate counterclockwise by ninety degrees.
    Counterclockwise,
}

/// Executed physical frame of a glyph run, including its independent pen axis.
///
/// Raw glyph x/y offsets use [`Self::outline_x`] and [`Self::outline_y`]. The
/// run-local pen first follows [`Self::pen_direction`] in those outline
/// coordinates. A combined cell retains horizontal pens and horizontal-only
/// compression even when its containing line advances vertically.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphFrame {
    origin: Point,
    kind: GlyphFrameKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GlyphFrameKind {
    Line(crate::LineOrientation, GlyphBasis),
    Horizontal(f64),
}

impl GlyphFrame {
    pub(crate) const fn line(
        origin: Point,
        orientation: crate::LineOrientation,
        basis: GlyphBasis,
    ) -> Self {
        Self {
            origin,
            kind: GlyphFrameKind::Line(orientation, basis),
        }
    }

    pub(crate) const fn horizontal(origin: Point, scale: f64) -> Self {
        Self {
            origin,
            kind: GlyphFrameKind::Horizontal(scale),
        }
    }

    /// Actual physical font-design origin after cell and enclosing placement.
    #[must_use]
    pub const fn origin(self) -> Point {
        self.origin
    }

    /// Physical vector for one unit of raw font x, including compression.
    #[must_use]
    pub const fn outline_x(self) -> Point {
        match self.kind {
            GlyphFrameKind::Horizontal(scale) => Point::new(scale, 0.0),
            GlyphFrameKind::Line(_, GlyphBasis::Unrotated) => Point::new(1.0, 0.0),
            GlyphFrameKind::Line(_, GlyphBasis::Clockwise) => Point::new(0.0, 1.0),
            GlyphFrameKind::Line(_, GlyphBasis::Counterclockwise) => Point::new(0.0, -1.0),
        }
    }

    /// Physical vector for one unit of converted, downward-positive font y.
    #[must_use]
    pub const fn outline_y(self) -> Point {
        match self.kind {
            GlyphFrameKind::Horizontal(_) | GlyphFrameKind::Line(_, GlyphBasis::Unrotated) => {
                Point::new(0.0, 1.0)
            }
            GlyphFrameKind::Line(_, GlyphBasis::Clockwise) => Point::new(-1.0, 0.0),
            GlyphFrameKind::Line(_, GlyphBasis::Counterclockwise) => Point::new(1.0, 0.0),
        }
    }

    /// Pen direction in raw outline coordinates, before the outline transform.
    #[must_use]
    pub const fn pen_direction(self) -> Point {
        use crate::LineOrientation::{
            Horizontal, SidewaysClockwise, SidewaysCounterclockwise, Vertical,
        };
        use GlyphBasis::{Clockwise, Counterclockwise, Unrotated};
        match self.kind {
            GlyphFrameKind::Horizontal(_)
            | GlyphFrameKind::Line(Horizontal, Unrotated)
            | GlyphFrameKind::Line(SidewaysClockwise | Vertical, Clockwise)
            | GlyphFrameKind::Line(SidewaysCounterclockwise, Counterclockwise) => {
                Point::new(1.0, 0.0)
            }
            GlyphFrameKind::Line(SidewaysClockwise | Vertical, Unrotated)
            | GlyphFrameKind::Line(Horizontal, Counterclockwise) => Point::new(0.0, 1.0),
            GlyphFrameKind::Line(Horizontal, Clockwise)
            | GlyphFrameKind::Line(SidewaysCounterclockwise, Unrotated) => Point::new(0.0, -1.0),
            GlyphFrameKind::Line(SidewaysClockwise | Vertical, Counterclockwise)
            | GlyphFrameKind::Line(SidewaysCounterclockwise, Clockwise) => Point::new(-1.0, 0.0),
        }
    }

    pub(crate) fn point(self, pen: f64, dx: f64, dy: f64) -> Point {
        match self.kind {
            GlyphFrameKind::Line(orientation, basis) => {
                crate::layout::glyph_origin(orientation, basis, self.origin, pen, dx, dy)
            }
            GlyphFrameKind::Horizontal(scale) => {
                Point::new(self.origin.x + scale * (pen + dx), self.origin.y + dy)
            }
        }
    }
}

/// Point in the coordinate frame documented by its owning operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// Size in the coordinate frame documented by its owning operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    #[must_use]
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
}

/// Rectangle in the coordinate frame documented by its owning operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[must_use]
    pub const fn origin(self) -> Point {
        self.origin
    }

    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.origin.x + self.size.width
            && point.y <= self.origin.y + self.size.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LineOrientation::{
        Horizontal, SidewaysClockwise, SidewaysCounterclockwise, Vertical,
    };
    use GlyphBasis::{Clockwise, Counterclockwise, Unrotated};

    #[test]
    fn line_frame_keeps_pen_direction_independent_of_outline_rotation() {
        // Physical origin (10,20), pen3 and raw offset(2,4). These expected
        // points follow the independently specified line and outline axes.
        let cases = [
            (Horizontal, Unrotated, Point::new(15.0, 24.0)),
            (Horizontal, Clockwise, Point::new(9.0, 22.0)),
            (Horizontal, Counterclockwise, Point::new(17.0, 18.0)),
            (Vertical, Unrotated, Point::new(12.0, 27.0)),
            (Vertical, Clockwise, Point::new(6.0, 25.0)),
            (Vertical, Counterclockwise, Point::new(14.0, 21.0)),
            (SidewaysClockwise, Unrotated, Point::new(12.0, 27.0)),
            (SidewaysClockwise, Clockwise, Point::new(6.0, 25.0)),
            (SidewaysClockwise, Counterclockwise, Point::new(14.0, 21.0)),
            (SidewaysCounterclockwise, Unrotated, Point::new(12.0, 21.0)),
            (SidewaysCounterclockwise, Clockwise, Point::new(6.0, 19.0)),
            (
                SidewaysCounterclockwise,
                Counterclockwise,
                Point::new(14.0, 15.0),
            ),
        ];
        for (orientation, basis, expected) in cases {
            let frame = GlyphFrame::line(Point::new(10.0, 20.0), orientation, basis);
            assert_eq!(frame.point(3.0, 2.0, 4.0), expected);
            let pen = frame.pen_direction();
            let x = 2.0 + 3.0 * pen.x;
            let y = 4.0 + 3.0 * pen.y;
            assert_eq!(
                Point::new(
                    frame.origin().x + frame.outline_x().x * x + frame.outline_y().x * y,
                    frame.origin().y + frame.outline_x().y * x + frame.outline_y().y * y,
                ),
                expected,
            );
        }
    }

    #[test]
    fn horizontal_frame_compresses_x_and_pen_without_scaling_y() {
        let frame = GlyphFrame::horizontal(Point::new(10.0, 20.0), 0.25);
        assert_eq!(frame.outline_x(), Point::new(0.25, 0.0));
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        assert_eq!(frame.point(3.0, 2.0, 4.0), Point::new(11.25, 24.0));
        assert_eq!(frame.point(-3.0, 2.0, -4.0), Point::new(9.75, 16.0));
    }
}
