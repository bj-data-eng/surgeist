//! Checked authored shape commands; geometric interpretation remains downstream.
use super::*;
use std::fmt;

/// Failure to construct an ordered authored shape command list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssShapeConstructionError {
    EmptyCommands,
}
impl fmt::Display for CssShapeConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("shape command list is empty")
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
            Err(CssShapeConstructionError::EmptyCommands)
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
    x: CssSpecifiedLengthPercentage,
    y: CssSpecifiedLengthPercentage,
}
impl CssShapeCoordinatePair {
    pub const fn new(x: CssSpecifiedLengthPercentage, y: CssSpecifiedLengthPercentage) -> Self {
        Self { x, y }
    }
    pub const fn x(&self) -> &CssSpecifiedLengthPercentage {
        &self.x
    }
    pub const fn y(&self) -> &CssSpecifiedLengthPercentage {
        &self.y
    }
}
impl PartialEq for CssShapeCoordinatePair {
    fn eq(&self, other: &Self) -> bool {
        self.x.structural_eq(&other.x) && self.y.structural_eq(&other.y)
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
    ToOffset(CssSpecifiedLengthPercentage),
    ToKeyword(CssHorizontalPositionKeyword),
    By(CssSpecifiedLengthPercentage),
}
impl PartialEq for CssShapeHorizontalLine {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ToOffset(a), Self::ToOffset(b)) | (Self::By(a), Self::By(b)) => {
                a.structural_eq(b)
            }
            (Self::ToKeyword(a), Self::ToKeyword(b)) => a == b,
            _ => false,
        }
    }
}
/// A checked vertical line affinity and operand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssShapeVerticalLine {
    ToOffset(CssSpecifiedLengthPercentage),
    ToKeyword(CssVerticalPositionKeyword),
    By(CssSpecifiedLengthPercentage),
}
impl PartialEq for CssShapeVerticalLine {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ToOffset(a), Self::ToOffset(b)) | (Self::By(a), Self::By(b)) => {
                a.structural_eq(b)
            }
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
    One(CssSpecifiedLengthPercentage),
    Two {
        horizontal: CssSpecifiedLengthPercentage,
        vertical: CssSpecifiedLengthPercentage,
    },
}
impl PartialEq for CssShapeArcRadii {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::One(a), Self::One(b)) => a.structural_eq(b),
            (
                Self::Two {
                    horizontal: a,
                    vertical: b,
                },
                Self::Two {
                    horizontal: c,
                    vertical: d,
                },
            ) => a.structural_eq(c) && b.structural_eq(d),
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
    rotation: Option<CssAngleValue>,
}
impl CssShapeArc {
    pub const fn new(
        endpoint: CssShapeEndpoint,
        radii: CssShapeArcRadii,
        sweep: Option<CssShapeArcSweep>,
        size: Option<CssShapeArcSize>,
        rotation: Option<CssAngleValue>,
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
    pub const fn rotation(&self) -> Option<&CssAngleValue> {
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
                (Some(a), Some(b)) => a.structural_eq(b),
                (None, None) => true,
                _ => false,
            }
    }
}
