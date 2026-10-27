//! Checked authored shape commands; geometric interpretation remains downstream.
use crate::numeric::{CssAngle, CssLengthPercentage, CssNumeric};
use crate::values::position::{
    CssCartesianPosition, CssHorizontalPosition, CssHorizontalPositionKeyword, CssPosition,
    CssVerticalPosition, CssVerticalPositionKeyword,
};
use crate::values::shape::path_data;
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
    origin: crate::component::CssValueOrigin,
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
        origin: crate::component::CssValueOrigin,
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
    crate::diagnostic::error::kind_predicates! { kind;
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
    pub const fn origin(&self) -> Option<&crate::component::CssValueOrigin> {
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
    /// Creates a command list.
    ///
    /// # Errors
    ///
    /// Returns an error for which `CssShapeConstructionError::is_empty_commands` holds when `commands` is empty.
    pub fn try_new(commands: Vec<CssShapeCommand>) -> Result<Self, CssShapeConstructionError> {
        if commands.is_empty() {
            Err(CssShapeConstructionError::from(
                ShapeConstructionKind::EmptyCommands,
            ))
        } else {
            Ok(Self { commands })
        }
    }
    /// Returns the commands in authored order.
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
    /// Creates a pair from a horizontal and a vertical offset, in that order, each a signed `<length-percentage>`.
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: CssNumeric<CssLengthPercentage>,
    ) -> Self {
        Self { x, y }
    }
    /// Returns the rightward offset.
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }
    /// Returns the downward offset.
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
    /// `to <position>`: an absolute endpoint, relative to the reference box.
    To(CssPosition),
    /// `by <coordinate-pair>`: an endpoint offset from the command's starting point.
    By(CssShapeCoordinatePair),
}
/// One authored command, without implicit moves or geometric defaults.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssShapeCommand {
    /// `move`: adds a `moveto` command that draws nothing and starts a new subpath.
    Move(CssShapeEndpoint),
    /// `line`: draws a straight line to the endpoint.
    Line(CssShapeEndpoint),
    /// `hline`: a horizontal `lineto`, equivalent to `line` with the given horizontal component.
    HorizontalLine(CssShapeHorizontalLine),
    /// `vline`: a vertical `lineto`, equivalent to `line` with the given vertical component.
    VerticalLine(CssShapeVerticalLine),
    /// `curve`: draws a quadratic or cubic Bezier curve.
    Curve(CssShapeCurve),
    /// `smooth`: draws a smooth quadratic or cubic Bezier curve continuing from the previous command.
    Smooth(CssShapeSmooth),
    /// `arc`: draws an elliptical arc.
    Arc(CssShapeArc),
    /// `close`: adds a `closepath` command, similar to a `line` to the subpath's starting point.
    Close,
}
/// A checked horizontal line affinity and operand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssShapeHorizontalLine {
    /// `to <length-percentage>`: an absolute horizontal offset in the reference box.
    ToOffset(CssNumeric<CssLengthPercentage>),
    /// `to` followed by a horizontal position keyword.
    ToKeyword(CssHorizontalPositionKeyword),
    /// `by <length-percentage>`: a horizontal offset from the starting point.
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
    /// `to <length-percentage>`: an absolute vertical offset in the reference box.
    ToOffset(CssNumeric<CssLengthPercentage>),
    /// `to` followed by a vertical position keyword.
    ToKeyword(CssVerticalPositionKeyword),
    /// `by <length-percentage>`: a vertical offset from the starting point.
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
    /// `start`: the offset is relative to the command's starting point.
    Start,
    /// `end`: the offset is relative to the command's end point.
    End,
    /// `origin`: the offset is relative to the reference box.
    Origin,
}
/// An explicitly authored shape arc sweep keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssShapeArcSweep {
    /// `cw`: the arc is drawn in the clockwise direction.
    Cw,
    /// `ccw`: the arc is drawn in the counterclockwise direction.
    Ccw,
}
/// An explicitly authored shape arc size keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssShapeArcSize {
    /// `large`: the larger of the two candidate arcs is drawn.
    Large,
    /// `small`: the smaller of the two candidate arcs is drawn.
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
    /// An absolute `<position>` control point.
    Position(&'a CssPosition),
    /// A coordinate pair control point with an explicit anchor.
    Coordinates {
        /// The authored coordinate pair.
        offset: &'a CssShapeCoordinatePair,
        /// The authored anchor the pair is relative to.
        anchor: CssShapeControlAnchor,
    },
}
impl CssShapeAbsoluteControlPoint {
    /// Creates a control point from an absolute `<position>`.
    pub const fn from_position(position: CssPosition) -> Self {
        Self {
            value: AbsoluteControl::Position(position),
        }
    }
    /// Creates a control point from a coordinate pair.
    /// With an anchor the pair stays anchored; without one it is stored as an absolute `<position>` of two free offsets.
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
    /// Returns a borrowed view of the stored control point.
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
    /// Creates a control point from a coordinate pair and an optional anchor; omission is retained.
    pub const fn new(
        offset: CssShapeCoordinatePair,
        anchor: Option<CssShapeControlAnchor>,
    ) -> Self {
        Self { offset, anchor }
    }
    /// Returns the authored coordinate pair.
    pub const fn offset(&self) -> &CssShapeCoordinatePair {
        &self.offset
    }
    /// Returns the authored anchor, or `None` when `from` was omitted and the pair is relative to the segment's start.
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
    /// A curve to an absolute endpoint.
    To {
        /// The absolute endpoint.
        end: &'a CssPosition,
        /// The first control point.
        first: &'a CssShapeAbsoluteControlPoint,
        /// The second control point, or `None` for a quadratic curve.
        second: Option<&'a CssShapeAbsoluteControlPoint>,
    },
    /// A curve to an endpoint offset from the starting point.
    By {
        /// The endpoint offset.
        end: &'a CssShapeCoordinatePair,
        /// The first relative control point.
        first: &'a CssShapeRelativeControlPoint,
        /// The second relative control point, or `None` for a quadratic curve.
        second: Option<&'a CssShapeRelativeControlPoint>,
    },
}
impl CssShapeCurve {
    /// Creates a curve to an absolute endpoint with one control point for a quadratic curve or two for a cubic curve.
    pub const fn to(
        end: CssPosition,
        first: CssShapeAbsoluteControlPoint,
        second: Option<CssShapeAbsoluteControlPoint>,
    ) -> Self {
        Self {
            value: Curve::To { end, first, second },
        }
    }
    /// Creates a curve to an endpoint offset from the starting point with one relative control point for a quadratic curve or two for a cubic curve.
    pub const fn by(
        end: CssShapeCoordinatePair,
        first: CssShapeRelativeControlPoint,
        second: Option<CssShapeRelativeControlPoint>,
    ) -> Self {
        Self {
            value: Curve::By { end, first, second },
        }
    }
    /// Returns a borrowed view of the endpoint and control points.
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
    /// A smooth curve to an absolute endpoint.
    To {
        /// The absolute endpoint.
        end: &'a CssPosition,
        /// The control point, or `None` for a smooth quadratic curve.
        control: Option<&'a CssShapeAbsoluteControlPoint>,
    },
    /// A smooth curve to an endpoint offset from the starting point.
    By {
        /// The endpoint offset.
        end: &'a CssShapeCoordinatePair,
        /// The relative control point, or `None` for a smooth quadratic curve.
        control: Option<&'a CssShapeRelativeControlPoint>,
    },
}
impl CssShapeSmooth {
    /// Creates a smooth curve to an absolute endpoint; an omitted control point makes it a smooth quadratic curve.
    pub const fn to(end: CssPosition, control: Option<CssShapeAbsoluteControlPoint>) -> Self {
        Self {
            value: Smooth::To { end, control },
        }
    }
    /// Creates a smooth curve to an endpoint offset from the starting point; an omitted control point makes it a smooth quadratic curve.
    pub const fn by(
        end: CssShapeCoordinatePair,
        control: Option<CssShapeRelativeControlPoint>,
    ) -> Self {
        Self {
            value: Smooth::By { end, control },
        }
    }
    /// Returns a borrowed view of the endpoint and control point.
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
    /// A single radius used for both axes.
    One(CssNumeric<CssLengthPercentage>),
    /// Separate horizontal and vertical radii.
    Two {
        /// The horizontal radius.
        horizontal: CssNumeric<CssLengthPercentage>,
        /// The vertical radius.
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
    /// Creates an arc.
    /// Only the endpoint and radii are required; the sweep, size and rotation options may each be omitted.
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
    /// Returns the arc's endpoint.
    pub const fn endpoint(&self) -> &CssShapeEndpoint {
        &self.endpoint
    }
    /// Returns the authored radii.
    pub const fn radii(&self) -> &CssShapeArcRadii {
        &self.radii
    }
    /// Returns the authored sweep direction, or `None` when omitted.
    pub const fn sweep(&self) -> Option<CssShapeArcSweep> {
        self.sweep
    }
    /// Returns the authored arc size, or `None` when omitted.
    pub const fn size(&self) -> Option<CssShapeArcSize> {
        self.size
    }
    /// Returns the authored ellipse rotation, or `None` when omitted.
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
