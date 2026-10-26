//! Checked authored shape commands; geometric interpretation remains downstream.
use super::*;
use std::fmt;

/// Failure to construct an ordered authored shape command list or checked SVG path data.
///
/// A path-data failure also retains its input [`Self::origin`] and, when the decoded
/// string was scanned, its [`Self::decoded_byte_offset`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssShapeConstructionError {
    kind: ShapeConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShapeConstructionKind {
    EmptyCommands,
    PathData(Box<PathDataRejection>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathDataRejection {
    failure: path_data::PathDataFailure,
    origin: crate::CssValueOrigin,
    decoded_byte_offset: Option<usize>,
}
impl From<ShapeConstructionKind> for CssShapeConstructionError {
    fn from(kind: ShapeConstructionKind) -> Self {
        Self { kind }
    }
}
impl CssShapeConstructionError {
    pub(crate) fn path_data(
        failure: path_data::PathDataFailure,
        origin: crate::CssValueOrigin,
        decoded_byte_offset: Option<usize>,
    ) -> Self {
        Self {
            kind: ShapeConstructionKind::PathData(Box::new(PathDataRejection {
                failure,
                origin,
                decoded_byte_offset,
            })),
        }
    }
    const fn path(&self) -> Option<&PathDataRejection> {
        match &self.kind {
            ShapeConstructionKind::PathData(rejection) => Some(rejection),
            ShapeConstructionKind::EmptyCommands => None,
        }
    }
    fn path_failure(&self) -> Option<path_data::PathDataFailure> {
        self.path().map(|rejection| rejection.failure)
    }
    crate::error::kind_predicates! { kind;
        /// Returns whether the command list is empty.
        is_empty_commands => ShapeConstructionKind::EmptyCommands,
    }
    /// Returns whether the path-data component is not a quoted string.
    #[must_use]
    pub fn is_expected_string(&self) -> bool {
        self.path_failure() == Some(path_data::PathDataFailure::ExpectedString)
    }
    /// Returns whether the path-data string has an implicitly recovered termination.
    #[must_use]
    pub fn is_recovered_input(&self) -> bool {
        self.path_failure() == Some(path_data::PathDataFailure::RecoveredInput)
    }
    /// Returns whether the decoded path data is empty or contains only SVG whitespace.
    #[must_use]
    pub fn is_empty_path(&self) -> bool {
        self.path_failure() == Some(path_data::PathDataFailure::EmptyPath)
    }
    /// Returns whether the decoded string does not satisfy the complete SVG path grammar.
    #[must_use]
    pub fn is_invalid_path_data(&self) -> bool {
        self.path_failure() == Some(path_data::PathDataFailure::InvalidPathData)
    }
    /// Returns the original component or programmatic origin of rejected path data.
    #[must_use]
    pub const fn origin(&self) -> Option<&crate::CssValueOrigin> {
        match self.path() {
            Some(rejection) => Some(&rejection.origin),
            None => None,
        }
    }
    /// Returns the first invalid decoded byte, or decoded length for incomplete path data.
    /// This offset is independent of CSS escaping and is not a CSS source coordinate.
    #[must_use]
    pub const fn decoded_byte_offset(&self) -> Option<usize> {
        match self.path() {
            Some(rejection) => rejection.decoded_byte_offset,
            None => None,
        }
    }
}
impl fmt::Display for CssShapeConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(rejection) = self.path() else {
            return f.write_str("shape command list is empty");
        };
        f.write_str(match rejection.failure {
            path_data::PathDataFailure::ExpectedString => "SVG path data requires a quoted string",
            path_data::PathDataFailure::RecoveredInput => {
                "SVG path data string is implicitly closed"
            }
            path_data::PathDataFailure::EmptyPath => "SVG path data is empty",
            path_data::PathDataFailure::InvalidPathData => "invalid SVG path data",
        })?;
        if let Some(offset) = rejection.decoded_byte_offset {
            write!(f, " at decoded byte {offset}")?;
        }
        Ok(())
    }
}
impl std::error::Error for CssShapeConstructionError {}

/// A nonempty ordered command sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeCommandList {
    commands: Vec<CssShapeCommand>,
}
impl CssShapeCommandList {
    pub fn try_new(commands: Vec<CssShapeCommand>) -> Result<Self, CssShapeConstructionError> {
        if commands.is_empty() {
            Err(CssShapeConstructionError::from(
                ShapeConstructionKind::EmptyCommands,
            ))
        } else {
            Ok(Self { commands })
        }
    }
    pub fn commands(&self) -> &[CssShapeCommand] {
        &self.commands
    }
}
/// Two independently checked signed authored offsets.
#[derive(Clone, Debug)]
pub struct CssShapeCoordinatePair {
    x: CssNumeric<CssLengthPercentage>,
    y: CssNumeric<CssLengthPercentage>,
}
impl CssShapeCoordinatePair {
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: CssNumeric<CssLengthPercentage>,
    ) -> Self {
        Self { x, y }
    }
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }
    pub const fn y(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.y
    }
}
impl PartialEq for CssShapeCoordinatePair {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
/// Absolute or relative authored endpoint.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssShapeEndpoint {
    To(CssPosition),
    By(CssShapeCoordinatePair),
}
/// One authored command, without implicit moves or geometric defaults.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssShapeCommand {
    Move(CssShapeEndpoint),
    Line(CssShapeEndpoint),
    HorizontalLine(CssShapeHorizontalLine),
    VerticalLine(CssShapeVerticalLine),
    Curve(CssShapeCurve),
    Smooth(CssShapeSmooth),
    Arc(CssShapeArc),
    Close,
}
/// A checked horizontal line affinity and operand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssShapeHorizontalLine {
    ToOffset(CssNumeric<CssLengthPercentage>),
    ToKeyword(CssHorizontalPositionKeyword),
    By(CssNumeric<CssLengthPercentage>),
}
impl PartialEq for CssShapeHorizontalLine {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ToOffset(a), Self::ToOffset(b)) | (Self::By(a), Self::By(b)) => a == b,
            (Self::ToKeyword(a), Self::ToKeyword(b)) => a == b,
            _ => false,
        }
    }
}
/// A checked vertical line affinity and operand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssShapeVerticalLine {
    ToOffset(CssNumeric<CssLengthPercentage>),
    ToKeyword(CssVerticalPositionKeyword),
    By(CssNumeric<CssLengthPercentage>),
}
impl PartialEq for CssShapeVerticalLine {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ToOffset(a), Self::ToOffset(b)) | (Self::By(a), Self::By(b)) => a == b,
            (Self::ToKeyword(a), Self::ToKeyword(b)) => a == b,
            _ => false,
        }
    }
}
/// An explicitly authored shape control anchor keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssShapeControlAnchor {
    Start,
    End,
    Origin,
}
/// An explicitly authored shape arc sweep keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssShapeArcSweep {
    Cw,
    Ccw,
}
/// An explicitly authored shape arc size keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssShapeArcSize {
    Large,
    Small,
}
/// An absolute position or an explicitly anchored numeric control pair.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeAbsoluteControlPoint {
    value: AbsoluteControl,
}
#[derive(Clone, Debug, PartialEq)]
enum AbsoluteControl {
    Position(CssPosition),
    Coordinates {
        offset: CssShapeCoordinatePair,
        anchor: CssShapeControlAnchor,
    },
}
/// Borrowed absolute control state.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssShapeAbsoluteControlPointRef<'a> {
    Position(&'a CssPosition),
    Coordinates {
        offset: &'a CssShapeCoordinatePair,
        anchor: CssShapeControlAnchor,
    },
}
impl CssShapeAbsoluteControlPoint {
    pub const fn from_position(position: CssPosition) -> Self {
        Self {
            value: AbsoluteControl::Position(position),
        }
    }
    pub fn from_coordinates(
        offset: CssShapeCoordinatePair,
        anchor: Option<CssShapeControlAnchor>,
    ) -> Self {
        let value = if let Some(anchor) = anchor {
            AbsoluteControl::Coordinates { offset, anchor }
        } else {
            AbsoluteControl::Position(CssPosition::from_cartesian(
                CssCartesianPosition::try_new(
                    CssHorizontalPosition::Offset(offset.x),
                    CssVerticalPosition::Offset(offset.y),
                )
                .expect("two free offsets are paired"),
            ))
        };
        Self { value }
    }
    pub const fn view(&self) -> CssShapeAbsoluteControlPointRef<'_> {
        match &self.value {
            AbsoluteControl::Position(p) => CssShapeAbsoluteControlPointRef::Position(p),
            AbsoluteControl::Coordinates { offset, anchor } => {
                CssShapeAbsoluteControlPointRef::Coordinates {
                    offset,
                    anchor: *anchor,
                }
            }
        }
    }
}
/// A relative pair retaining anchor omission.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeRelativeControlPoint {
    offset: CssShapeCoordinatePair,
    anchor: Option<CssShapeControlAnchor>,
}
impl CssShapeRelativeControlPoint {
    pub const fn new(
        offset: CssShapeCoordinatePair,
        anchor: Option<CssShapeControlAnchor>,
    ) -> Self {
        Self { offset, anchor }
    }
    pub const fn offset(&self) -> &CssShapeCoordinatePair {
        &self.offset
    }
    pub const fn anchor(&self) -> Option<CssShapeControlAnchor> {
        self.anchor
    }
}
/// A curve whose controls are coupled to endpoint affinity.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeCurve {
    value: Curve,
}
#[derive(Clone, Debug, PartialEq)]
enum Curve {
    To {
        end: CssPosition,
        first: CssShapeAbsoluteControlPoint,
        second: Option<CssShapeAbsoluteControlPoint>,
    },
    By {
        end: CssShapeCoordinatePair,
        first: CssShapeRelativeControlPoint,
        second: Option<CssShapeRelativeControlPoint>,
    },
}
/// Borrowed affinity-coupled curve state.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssShapeCurveRef<'a> {
    To {
        end: &'a CssPosition,
        first: &'a CssShapeAbsoluteControlPoint,
        second: Option<&'a CssShapeAbsoluteControlPoint>,
    },
    By {
        end: &'a CssShapeCoordinatePair,
        first: &'a CssShapeRelativeControlPoint,
        second: Option<&'a CssShapeRelativeControlPoint>,
    },
}
impl CssShapeCurve {
    pub const fn to(
        end: CssPosition,
        first: CssShapeAbsoluteControlPoint,
        second: Option<CssShapeAbsoluteControlPoint>,
    ) -> Self {
        Self {
            value: Curve::To { end, first, second },
        }
    }
    pub const fn by(
        end: CssShapeCoordinatePair,
        first: CssShapeRelativeControlPoint,
        second: Option<CssShapeRelativeControlPoint>,
    ) -> Self {
        Self {
            value: Curve::By { end, first, second },
        }
    }
    pub const fn view(&self) -> CssShapeCurveRef<'_> {
        match &self.value {
            Curve::To { end, first, second } => CssShapeCurveRef::To {
                end,
                first,
                second: second.as_ref(),
            },
            Curve::By { end, first, second } => CssShapeCurveRef::By {
                end,
                first,
                second: second.as_ref(),
            },
        }
    }
}
/// A smooth command whose control is coupled to endpoint affinity.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeSmooth {
    value: Smooth,
}
#[derive(Clone, Debug, PartialEq)]
enum Smooth {
    To {
        end: CssPosition,
        control: Option<CssShapeAbsoluteControlPoint>,
    },
    By {
        end: CssShapeCoordinatePair,
        control: Option<CssShapeRelativeControlPoint>,
    },
}
/// Borrowed affinity-coupled smooth state.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssShapeSmoothRef<'a> {
    To {
        end: &'a CssPosition,
        control: Option<&'a CssShapeAbsoluteControlPoint>,
    },
    By {
        end: &'a CssShapeCoordinatePair,
        control: Option<&'a CssShapeRelativeControlPoint>,
    },
}
impl CssShapeSmooth {
    pub const fn to(end: CssPosition, control: Option<CssShapeAbsoluteControlPoint>) -> Self {
        Self {
            value: Smooth::To { end, control },
        }
    }
    pub const fn by(
        end: CssShapeCoordinatePair,
        control: Option<CssShapeRelativeControlPoint>,
    ) -> Self {
        Self {
            value: Smooth::By { end, control },
        }
    }
    pub const fn view(&self) -> CssShapeSmoothRef<'_> {
        match &self.value {
            Smooth::To { end, control } => CssShapeSmoothRef::To {
                end,
                control: control.as_ref(),
            },
            Smooth::By { end, control } => CssShapeSmoothRef::By {
                end,
                control: control.as_ref(),
            },
        }
    }
}
/// One or two signed radii, retaining authored arity.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssShapeArcRadii {
    One(CssNumeric<CssLengthPercentage>),
    Two {
        horizontal: CssNumeric<CssLengthPercentage>,
        vertical: CssNumeric<CssLengthPercentage>,
    },
}
impl PartialEq for CssShapeArcRadii {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::One(a), Self::One(b)) => a == b,
            (
                Self::Two {
                    horizontal: a,
                    vertical: b,
                },
                Self::Two {
                    horizontal: c,
                    vertical: d,
                },
            ) => a == c && b == d,
            _ => false,
        }
    }
}
/// An arc with required endpoint/radii and independently omitted authored options.
#[derive(Clone, Debug)]
pub struct CssShapeArc {
    endpoint: CssShapeEndpoint,
    radii: CssShapeArcRadii,
    sweep: Option<CssShapeArcSweep>,
    size: Option<CssShapeArcSize>,
    rotation: Option<CssNumeric<CssAngle>>,
}
impl CssShapeArc {
    pub const fn new(
        endpoint: CssShapeEndpoint,
        radii: CssShapeArcRadii,
        sweep: Option<CssShapeArcSweep>,
        size: Option<CssShapeArcSize>,
        rotation: Option<CssNumeric<CssAngle>>,
    ) -> Self {
        Self {
            endpoint,
            radii,
            sweep,
            size,
            rotation,
        }
    }
    pub const fn endpoint(&self) -> &CssShapeEndpoint {
        &self.endpoint
    }
    pub const fn radii(&self) -> &CssShapeArcRadii {
        &self.radii
    }
    pub const fn sweep(&self) -> Option<CssShapeArcSweep> {
        self.sweep
    }
    pub const fn size(&self) -> Option<CssShapeArcSize> {
        self.size
    }
    pub const fn rotation(&self) -> Option<&CssNumeric<CssAngle>> {
        self.rotation.as_ref()
    }
}
impl PartialEq for CssShapeArc {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint == other.endpoint
            && self.radii == other.radii
            && self.sweep == other.sweep
            && self.size == other.size
            && match (&self.rotation, &other.rotation) {
                (Some(a), Some(b)) => a == b,
                (None, None) => true,
                _ => false,
            }
    }
}
