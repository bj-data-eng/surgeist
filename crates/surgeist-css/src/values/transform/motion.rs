//! Checked authored Motion Path values. Geometry and resource resolution remain symbolic.

use crate::numeric::{CssAngle, CssLengthPercentage, CssNumeric};
use crate::values::common::{CssBoxEdgeKeyword, CssUrl};
use crate::values::position::CssPosition;
use crate::values::shape::CssBasicShape;
use std::{error::Error, fmt};

/// A rejected intrinsic Motion construction, before any contextual evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssMotionConstructionError {
    kind: MotionConstructionKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MotionConstructionKind {
    /// The imported Box 3 coord-box grammar excludes this geometry box.
    InvalidCoordBox,
    /// A rotation requires a modifier, an angle, or both.
    EmptyRotate,
    /// The shorthand requires a position or a path before its optional anchor.
    EmptyOffset,
    /// Distance and rotation require an explicitly authored path property.
    MissingPath,
}
impl From<MotionConstructionKind> for CssMotionConstructionError {
    fn from(kind: MotionConstructionKind) -> Self {
        Self { kind }
    }
}
impl CssMotionConstructionError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether the imported Box 3 coord-box grammar excludes this geometry box.
        is_invalid_coord_box => MotionConstructionKind::InvalidCoordBox,
        /// Returns whether a rotation requires a modifier, an angle, or both.
        is_empty_rotate => MotionConstructionKind::EmptyRotate,
        /// Returns whether the shorthand requires a position or a path before its optional anchor.
        is_empty_offset => MotionConstructionKind::EmptyOffset,
        /// Returns whether distance and rotation require an explicitly authored path property.
        is_missing_path => MotionConstructionKind::MissingPath,
    }
}
impl fmt::Display for CssMotionConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match &self.kind {
            MotionConstructionKind::InvalidCoordBox => {
                "box is outside the Motion coord-box grammar"
            }
            MotionConstructionKind::EmptyRotate => "offset rotation requires a modifier or angle",
            MotionConstructionKind::EmptyOffset => "offset requires a position or path",
            MotionConstructionKind::MissingPath => "offset distance and rotation require a path",
        })
    }
}
impl Error for CssMotionConstructionError {}

/// Box 4 `<coord-box>`: the six geometry boxes that exclude `margin-box`.
///
/// Motion's `offset-path` and `offset` and Masking's `mask-origin`, `mask-clip` and `mask`
/// all take this one production.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
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
        CssBoxEdgeKeyword::from_keyword(keyword).and_then(|edge| Self::try_from(edge).ok())
    }
    /// Returns the canonical keyword.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.edge().as_str()
    }
    /// The geometry-box keyword this `<coord-box>` names.
    const fn edge(self) -> CssBoxEdgeKeyword {
        match self {
            Self::ContentBox => CssBoxEdgeKeyword::ContentBox,
            Self::PaddingBox => CssBoxEdgeKeyword::PaddingBox,
            Self::BorderBox => CssBoxEdgeKeyword::BorderBox,
            Self::FillBox => CssBoxEdgeKeyword::FillBox,
            Self::StrokeBox => CssBoxEdgeKeyword::StrokeBox,
            Self::ViewBox => CssBoxEdgeKeyword::ViewBox,
        }
    }
}
impl TryFrom<CssBoxEdgeKeyword> for CssCoordBox {
    type Error = CssMotionConstructionError;
    fn try_from(value: CssBoxEdgeKeyword) -> Result<Self, Self::Error> {
        Ok(match value {
            CssBoxEdgeKeyword::ContentBox => Self::ContentBox,
            CssBoxEdgeKeyword::PaddingBox => Self::PaddingBox,
            CssBoxEdgeKeyword::BorderBox => Self::BorderBox,
            CssBoxEdgeKeyword::FillBox => Self::FillBox,
            CssBoxEdgeKeyword::StrokeBox => Self::StrokeBox,
            CssBoxEdgeKeyword::ViewBox => Self::ViewBox,
            CssBoxEdgeKeyword::MarginBox => {
                return Err(CssMotionConstructionError::from(
                    MotionConstructionKind::InvalidCoordBox,
                ));
            }
        })
    }
}

/// An explicitly authored ray size; omission separately denotes closest-side.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
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
    pub const fn as_str(self) -> &'static str {
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
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRay {
    angle: CssNumeric<CssAngle>,
    size: Option<CssRaySize>,
    contain: bool,
    position: Option<CssPosition>,
}
impl CssRay {
    /// Composes checked children without inserting missing size or position defaults.
    #[must_use]
    pub const fn new(
        angle: CssNumeric<CssAngle>,
        size: Option<CssRaySize>,
        contain: bool,
        position: Option<CssPosition>,
    ) -> Self {
        Self {
            angle,
            size,
            contain,
            position,
        }
    }
    #[must_use]
    pub const fn angle(&self) -> &CssNumeric<CssAngle> {
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
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }
}

/// Exactly one authored path production, distinct from the whole property grammar.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetPathKind {
    Ray(CssRay),
    Url(CssUrl),
    BasicShape(CssBasicShape),
}

/// A checked path with an optional explicitly authored coordinate box.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
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
pub(crate) enum OffsetPath {
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
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
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
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetPosition {
    Normal,
    Auto,
    Position(CssPosition),
}
/// The authored offset-anchor, before transform-origin or reference-box resolution.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetAnchor {
    Auto,
    Position(CssPosition),
}
/// An explicit authored tangent-rotation modifier; absence stays distinct from auto.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOffsetRotateModifier {
    Auto,
    Reverse,
}
/// A nonempty authored modifier/angle combination, without tangent evaluation.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Debug, PartialEq)]
pub struct CssOffsetRotate {
    modifier: Option<CssOffsetRotateModifier>,
    angle: Option<CssNumeric<CssAngle>>,
}
impl CssOffsetRotate {
    /// Rejects the absent-both state and retains every provided constituent.
    pub fn try_new(
        modifier: Option<CssOffsetRotateModifier>,
        angle: Option<CssNumeric<CssAngle>>,
    ) -> Result<Self, CssMotionConstructionError> {
        if modifier.is_none() && angle.is_none() {
            return Err(CssMotionConstructionError::from(
                MotionConstructionKind::EmptyRotate,
            ));
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
    pub const fn angle(&self) -> Option<&CssNumeric<CssAngle>> {
        self.angle.as_ref()
    }
}

/// Authored shorthand constituents, before its five intrinsic reset contributions.
///
/// # Specified serialization
///
/// Serializes authored constituents without resolving paths, bearings or resources.
#[derive(Clone, Debug, PartialEq)]
pub struct CssOffset {
    position: Option<CssOffsetPosition>,
    path: Option<CssOffsetPath>,
    distance: Option<CssNumeric<CssLengthPercentage>>,
    rotate: Option<CssOffsetRotate>,
    anchor: Option<CssOffsetAnchor>,
}
impl CssOffset {
    /// Requires position or path, and requires path for distance or rotation.
    pub fn try_new(
        position: Option<CssOffsetPosition>,
        path: Option<CssOffsetPath>,
        distance: Option<CssNumeric<CssLengthPercentage>>,
        rotate: Option<CssOffsetRotate>,
        anchor: Option<CssOffsetAnchor>,
    ) -> Result<Self, CssMotionConstructionError> {
        if position.is_none() && path.is_none() {
            return Err(CssMotionConstructionError::from(
                MotionConstructionKind::EmptyOffset,
            ));
        }
        if path.is_none() && (distance.is_some() || rotate.is_some()) {
            return Err(CssMotionConstructionError::from(
                MotionConstructionKind::MissingPath,
            ));
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
    pub const fn distance(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
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
