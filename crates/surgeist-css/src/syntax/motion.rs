//! Checked authored Motion Path values. Geometry and resource resolution remain symbolic.

use super::{CssBasicShape, CssBoxEdgeKeyword, CssPhysicalPosition, CssUrl, optional_numeric_eq};
use crate::{CssAngleValue, CssSpecifiedLengthPercentage};
use std::{error::Error, fmt};

/// A rejected intrinsic Motion construction, before any contextual evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMotionConstructionError {
    /// The imported Box 3 coord-box grammar excludes this geometry box.
    InvalidCoordBox,
    /// A rotation requires a modifier, an angle, or both.
    EmptyRotate,
    /// The shorthand requires a position or a path before its optional anchor.
    EmptyOffset,
    /// Distance and rotation require an explicitly authored path property.
    MissingPath,
}
impl fmt::Display for CssMotionConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidCoordBox => "box is outside the Motion coord-box grammar",
            Self::EmptyRotate => "offset rotation requires a modifier or angle",
            Self::EmptyOffset => "offset requires a position or path",
            Self::MissingPath => "offset distance and rotation require a path",
        })
    }
}
impl Error for CssMotionConstructionError {}

/// The six Box 3 coordinate boxes imported by Motion; margin-box is excluded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssCoordBox {
    ContentBox,
    PaddingBox,
    BorderBox,
    FillBox,
    StrokeBox,
    ViewBox,
}
impl CssCoordBox {
    /// Admits a decoded coordinate-box keyword without selecting used geometry.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "content-box" => Self::ContentBox,
            "padding-box" => Self::PaddingBox,
            "border-box" => Self::BorderBox,
            "fill-box" => Self::FillBox,
            "stroke-box" => Self::StrokeBox,
            "view-box" => Self::ViewBox,
            _ => return None,
        })
    }
    /// Returns the canonical keyword.
    #[must_use]
    pub const fn as_css_str(self) -> &'static str {
        match self {
            Self::ContentBox => "content-box",
            Self::PaddingBox => "padding-box",
            Self::BorderBox => "border-box",
            Self::FillBox => "fill-box",
            Self::StrokeBox => "stroke-box",
            Self::ViewBox => "view-box",
        }
    }
}
impl TryFrom<CssBoxEdgeKeyword> for CssCoordBox {
    type Error = CssMotionConstructionError;
    fn try_from(value: CssBoxEdgeKeyword) -> Result<Self, Self::Error> {
        match value {
            CssBoxEdgeKeyword::ContentBox => Ok(Self::ContentBox),
            CssBoxEdgeKeyword::PaddingBox => Ok(Self::PaddingBox),
            CssBoxEdgeKeyword::BorderBox => Ok(Self::BorderBox),
            CssBoxEdgeKeyword::FillBox => Ok(Self::FillBox),
            CssBoxEdgeKeyword::StrokeBox => Ok(Self::StrokeBox),
            CssBoxEdgeKeyword::ViewBox => Ok(Self::ViewBox),
            CssBoxEdgeKeyword::MarginBox => Err(CssMotionConstructionError::InvalidCoordBox),
        }
    }
}

/// An explicitly authored ray size; omission separately denotes closest-side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRaySize {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
    Sides,
}
impl CssRaySize {
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "closest-side" => Self::ClosestSide,
            "closest-corner" => Self::ClosestCorner,
            "farthest-side" => Self::FarthestSide,
            "farthest-corner" => Self::FarthestCorner,
            "sides" => Self::Sides,
            _ => return None,
        })
    }
    #[must_use]
    pub const fn as_css_str(self) -> &'static str {
        match self {
            Self::ClosestSide => "closest-side",
            Self::ClosestCorner => "closest-corner",
            Self::FarthestSide => "farthest-side",
            Self::FarthestCorner => "farthest-corner",
            Self::Sides => "sides",
        }
    }
}

/// An authored bearing, where zero points up and positive angles run clockwise.
/// The checked angle is retained without modulo, tangent or geometric evaluation.
#[derive(Clone, Debug)]
pub struct CssRay {
    angle: CssAngleValue,
    size: Option<CssRaySize>,
    contain: bool,
    position: Option<CssPhysicalPosition>,
}
numeric_fields_eq!(CssRay, [angle], [], [size, contain, position]);
impl CssRay {
    /// Composes checked children without inserting missing size or position defaults.
    #[must_use]
    pub const fn new(
        angle: CssAngleValue,
        size: Option<CssRaySize>,
        contain: bool,
        position: Option<CssPhysicalPosition>,
    ) -> Self {
        Self {
            angle,
            size,
            contain,
            position,
        }
    }
    #[must_use]
    pub const fn angle(&self) -> &CssAngleValue {
        &self.angle
    }
    #[must_use]
    pub const fn size(&self) -> Option<CssRaySize> {
        self.size
    }
    #[must_use]
    pub const fn contain(&self) -> bool {
        self.contain
    }
    #[must_use]
    pub const fn position(&self) -> Option<&CssPhysicalPosition> {
        self.position.as_ref()
    }
}

/// Exactly one authored path production, distinct from the whole property grammar.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetPathKind {
    Ray(CssRay),
    Url(CssUrl),
    BasicShape(CssBasicShape),
}

/// A checked path with an optional explicitly authored coordinate box.
#[derive(Clone, Debug, PartialEq)]
pub struct CssOffsetPathValue {
    path: CssOffsetPathKind,
    coord_box: Option<CssCoordBox>,
}
impl CssOffsetPathValue {
    #[must_use]
    pub const fn new(path: CssOffsetPathKind, coord_box: Option<CssCoordBox>) -> Self {
        Self { path, coord_box }
    }
    #[must_use]
    pub const fn path(&self) -> &CssOffsetPathKind {
        &self.path
    }
    #[must_use]
    pub const fn coord_box(&self) -> Option<CssCoordBox> {
        self.coord_box
    }
}

#[derive(Clone, Debug, PartialEq)]
enum OffsetPath {
    None,
    CoordBox(CssCoordBox),
    Path(CssOffsetPathValue),
}
/// A borrowed whole offset-path branch, preserving authored box omission.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetPathRef<'a> {
    None,
    CoordBox(CssCoordBox),
    Path(&'a CssOffsetPathValue),
}
/// The whole authored offset-path property, including none and box-only branches.
#[derive(Clone, Debug, PartialEq)]
pub struct CssOffsetPath {
    value: OffsetPath,
}
impl CssOffsetPath {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            value: OffsetPath::None,
        }
    }
    #[must_use]
    pub const fn from_coord_box(value: CssCoordBox) -> Self {
        Self {
            value: OffsetPath::CoordBox(value),
        }
    }
    #[must_use]
    pub const fn from_path(value: CssOffsetPathValue) -> Self {
        Self {
            value: OffsetPath::Path(value),
        }
    }
    #[must_use]
    pub const fn view(&self) -> CssOffsetPathRef<'_> {
        match &self.value {
            OffsetPath::None => CssOffsetPathRef::None,
            OffsetPath::CoordBox(value) => CssOffsetPathRef::CoordBox(*value),
            OffsetPath::Path(value) => CssOffsetPathRef::Path(value),
        }
    }
}

/// The authored offset-position, before containing-block or own-box resolution.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetPosition {
    Normal,
    Auto,
    Position(CssPhysicalPosition),
}
/// The authored offset-anchor, before transform-origin or reference-box resolution.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetAnchor {
    Auto,
    Position(CssPhysicalPosition),
}
/// An explicit authored tangent-rotation modifier; absence stays distinct from auto.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetRotateModifier {
    Auto,
    Reverse,
}
/// A nonempty authored modifier/angle combination, without tangent evaluation.
#[derive(Clone, Debug)]
pub struct CssOffsetRotate {
    modifier: Option<CssOffsetRotateModifier>,
    angle: Option<CssAngleValue>,
}
numeric_fields_eq!(CssOffsetRotate, [], [angle], [modifier]);
impl CssOffsetRotate {
    /// Rejects the absent-both state and retains every provided constituent.
    pub fn try_new(
        modifier: Option<CssOffsetRotateModifier>,
        angle: Option<CssAngleValue>,
    ) -> Result<Self, CssMotionConstructionError> {
        if modifier.is_none() && angle.is_none() {
            return Err(CssMotionConstructionError::EmptyRotate);
        }
        Ok(Self { modifier, angle })
    }
    #[must_use]
    pub const fn auto() -> Self {
        Self {
            modifier: Some(CssOffsetRotateModifier::Auto),
            angle: None,
        }
    }
    #[must_use]
    pub const fn modifier(&self) -> Option<CssOffsetRotateModifier> {
        self.modifier
    }
    #[must_use]
    pub const fn angle(&self) -> Option<&CssAngleValue> {
        self.angle.as_ref()
    }
}

/// Authored shorthand constituents, before its five intrinsic reset contributions.
#[derive(Clone, Debug)]
pub struct CssOffset {
    position: Option<CssOffsetPosition>,
    path: Option<CssOffsetPath>,
    distance: Option<CssSpecifiedLengthPercentage>,
    rotate: Option<CssOffsetRotate>,
    anchor: Option<CssOffsetAnchor>,
}
numeric_fields_eq!(CssOffset, [], [distance], [position, path, rotate, anchor]);
impl CssOffset {
    /// Requires position or path, and requires path for distance or rotation.
    pub fn try_new(
        position: Option<CssOffsetPosition>,
        path: Option<CssOffsetPath>,
        distance: Option<CssSpecifiedLengthPercentage>,
        rotate: Option<CssOffsetRotate>,
        anchor: Option<CssOffsetAnchor>,
    ) -> Result<Self, CssMotionConstructionError> {
        if position.is_none() && path.is_none() {
            return Err(CssMotionConstructionError::EmptyOffset);
        }
        if path.is_none() && (distance.is_some() || rotate.is_some()) {
            return Err(CssMotionConstructionError::MissingPath);
        }
        Ok(Self {
            position,
            path,
            distance,
            rotate,
            anchor,
        })
    }
    #[must_use]
    pub const fn position(&self) -> Option<&CssOffsetPosition> {
        self.position.as_ref()
    }
    #[must_use]
    pub const fn path(&self) -> Option<&CssOffsetPath> {
        self.path.as_ref()
    }
    #[must_use]
    pub const fn distance(&self) -> Option<&CssSpecifiedLengthPercentage> {
        self.distance.as_ref()
    }
    #[must_use]
    pub const fn rotate(&self) -> Option<&CssOffsetRotate> {
        self.rotate.as_ref()
    }
    #[must_use]
    pub const fn anchor(&self) -> Option<&CssOffsetAnchor> {
        self.anchor.as_ref()
    }
}
