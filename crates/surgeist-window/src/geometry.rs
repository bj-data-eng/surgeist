use super::{Error, ErrorCode, Result};

/// A point in logical window coordinates.
///
/// Logical coordinates use `f64` values and are scaled for the active display;
/// they are distinct from native [`PhysicalPoint`] integer pixel coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Horizontal logical coordinate.
    pub x: f64,
    /// Vertical logical coordinate.
    pub y: f64,
}

/// A logical width and height in window coordinates.
///
/// These values are display-scale independent, unlike [`PhysicalSize`] pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    /// Logical width.
    pub width: f64,
    /// Logical height.
    pub height: f64,
}

/// A point in native integer pixel coordinates.
///
/// Use [`Metrics`](crate::Metrics) to convert between this representation and
/// logical coordinates at an observed display scale.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalPoint {
    /// Horizontal native-pixel coordinate.
    pub x: i32,
    /// Vertical native-pixel coordinate.
    pub y: i32,
}

/// A size in native integer pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalSize {
    /// Native-pixel width.
    pub width: u32,
    /// Native-pixel height.
    pub height: u32,
}

/// Logical insets around a window region.
///
/// Each edge is expressed in the same logical units as [`Point`] and [`Size`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// Logical inset from the top edge.
    pub top: f64,
    /// Logical inset from the right edge.
    pub right: f64,
    /// Logical inset from the bottom edge.
    pub bottom: f64,
    /// Logical inset from the left edge.
    pub left: f64,
}

/// A rectangle in logical window coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Logical origin of the rectangle.
    pub origin: Point,
    /// Logical extent of the rectangle.
    pub size: Size,
}

pub(crate) fn normalize_outer_position(position: Point) -> Result<Point> {
    if !position.x.is_finite() || !position.y.is_finite() {
        return Err(Error::new(
            ErrorCode::InvalidRequest,
            "outer position must use finite coordinates",
        ));
    }

    Ok(Point {
        x: canonical_zero(position.x),
        y: canonical_zero(position.y),
    })
}

pub(crate) fn normalize_nonnegative_size(size: Size, name: &str) -> Result<Size> {
    if !size.width.is_finite() || !size.height.is_finite() || size.width < 0.0 || size.height < 0.0
    {
        return Err(Error::new(
            ErrorCode::InvalidRequest,
            format!("{name} must use finite nonnegative dimensions"),
        ));
    }

    Ok(Size {
        width: canonical_zero(size.width),
        height: canonical_zero(size.height),
    })
}

pub(crate) fn normalize_local_rect(rect: Rect) -> Result<Rect> {
    if !rect.origin.x.is_finite()
        || !rect.origin.y.is_finite()
        || rect.origin.x < 0.0
        || rect.origin.y < 0.0
    {
        return Err(Error::new(
            ErrorCode::InvalidRequest,
            "local rectangle origin must use finite nonnegative coordinates",
        ));
    }

    Ok(Rect {
        origin: Point {
            x: canonical_zero(rect.origin.x),
            y: canonical_zero(rect.origin.y),
        },
        size: normalize_nonnegative_size(rect.size, "local rectangle size")?,
    })
}

const fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// Opaque native window identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u64);

impl Id {
    /// Creates an opaque identifier from its raw numeric representation.
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw numeric representation of this identifier.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}
