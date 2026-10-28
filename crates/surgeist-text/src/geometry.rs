/// Outline axes of the glyphs in an admitted uniform run.
///
/// Glyph offsets use font x to the right and converted font y downward. The
/// logical glyph pen follows the run's inline axis independently of these axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphBasis {
    /// Horizontal and whole-upright glyph outlines keep their original axes.
    Unrotated,
    /// Whole-sideways glyph outlines rotate clockwise by ninety degrees.
    Clockwise,
}

impl GlyphBasis {
    pub(crate) const fn for_orientation(orientation: crate::LineOrientation) -> Self {
        match orientation {
            crate::LineOrientation::Horizontal | crate::LineOrientation::Vertical => {
                Self::Unrotated
            }
            crate::LineOrientation::Sideways => Self::Clockwise,
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
