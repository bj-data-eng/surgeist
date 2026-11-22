//! Computed command structure, independent of authored CSS and resolved geometry.

use super::*;

/// An absolute target or a displacement from the command's starting point.
///
/// Both axes are already normalized rightward/downward length-percentages.
/// Percentages in `By` still use the reference box's width/height.
#[derive(Debug, Clone, PartialEq)]
pub enum Endpoint {
    To(ComputedPosition),
    By(ComputedPosition),
}

impl Endpoint {
    fn coordinates(&self) -> &ComputedPosition {
        match self {
            Self::To(point) | Self::By(point) => point,
        }
    }
    fn same_form(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::To(_), Self::To(_)) | (Self::By(_), Self::By(_))
        )
    }
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShapeError> {
        let point = self
            .coordinates()
            .interpolate(other.coordinates(), progress)
            .map_err(ShapeError::numeric)?;
        Ok(match self {
            Self::To(_) => Self::To(point),
            Self::By(_) => Self::By(point),
        })
    }
    fn path_anchor(&self) -> ControlAnchor {
        match self {
            Self::To(_) => ControlAnchor::Origin,
            Self::By(_) => ControlAnchor::Start,
        }
    }
}

/// An absolute horizontal/vertical offset or a relative displacement.
#[derive(Debug, Clone, PartialEq)]
pub enum AxisCommand {
    To(LengthPercentage),
    By(LengthPercentage),
}
impl AxisCommand {
    fn offset(&self) -> &LengthPercentage {
        match self {
            Self::To(value) | Self::By(value) => value,
        }
    }
    fn same_form(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::To(_), Self::To(_)) | (Self::By(_), Self::By(_))
        )
    }
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShapeError> {
        let value = self.offset().interpolate(other.offset(), progress)?;
        Ok(match self {
            Self::To(_) => Self::To(value),
            Self::By(_) => Self::By(value),
        })
    }
}

/// The effective origin of an explicit control point's x/y offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlAnchor {
    Start,
    End,
    Origin,
}

/// A computed control point with an explicit effective anchor.
///
/// Callers lower an absolute bare position to `Origin`, and a relative control
/// with omitted anchor to `Start`. Different effective anchors are incompatible.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlPoint {
    anchor: ControlAnchor,
    offset: ComputedPosition,
}
impl ControlPoint {
    pub const fn new(anchor: ControlAnchor, offset: ComputedPosition) -> Self {
        Self { anchor, offset }
    }
    pub const fn anchor(&self) -> ControlAnchor {
        self.anchor
    }
    pub const fn offset(&self) -> &ComputedPosition {
        &self.offset
    }
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShapeError> {
        Ok(Self::new(
            self.anchor,
            self.offset
                .interpolate(&other.offset, progress)
                .map_err(ShapeError::numeric)?,
        ))
    }
}

/// Explicit controls of a quadratic or cubic curve, retaining the curve degree.
#[derive(Debug, Clone, PartialEq)]
pub enum CurveControls {
    Quadratic(ControlPoint),
    Cubic(ControlPoint, ControlPoint),
}
impl CurveControls {
    fn compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Quadratic(first), Self::Quadratic(second)) => first.anchor == second.anchor,
            (Self::Cubic(a, b), Self::Cubic(c, d)) => a.anchor == c.anchor && b.anchor == d.anchor,
            _ => false,
        }
    }
}

/// Effective arc sweep, with omitted authored sweep lowered to `Ccw`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArcSweep {
    Ccw,
    Cw,
}
/// Effective arc size, with omitted authored size lowered to `Small`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArcSize {
    Small,
    Large,
}

/// An elliptical arc's computed operands without geometric correction.
///
/// Radii use their respective axis percentage bases. A caller lowering one
/// authored radius must first preserve its direction-agnostic percentage basis:
/// supply a pure-Length symbolic source for that quantity in both axes, or
/// equivalent already resolved lengths. Copying a diagonal-based percentage
/// into two ordinary axis percentages changes its meaning on non-square boxes.
/// Negative/zero radii, coincident endpoints and radius enlargement remain with
/// the geometry consumer. Rotation uses canonical computed Angle units.
#[derive(Debug, Clone, PartialEq)]
pub struct ArcCommand {
    end: Endpoint,
    horizontal_radius: LengthPercentage,
    vertical_radius: LengthPercentage,
    rotation: ComputedNumeric,
    sweep: ArcSweep,
    size: ArcSize,
}
impl ArcCommand {
    pub fn new(
        end: Endpoint,
        horizontal_radius: LengthPercentage,
        vertical_radius: LengthPercentage,
        rotation: ComputedNumeric,
        sweep: ArcSweep,
        size: ArcSize,
    ) -> Result<Self, ShapeError> {
        if rotation.numeric_type() != NumericType::Dimension(DimensionKind::Angle) {
            return Err(ShapeError::invalid_type(rotation.numeric_type()));
        }
        Ok(Self {
            end,
            horizontal_radius,
            vertical_radius,
            rotation,
            sweep,
            size,
        })
    }
    pub const fn end(&self) -> &Endpoint {
        &self.end
    }
    pub const fn horizontal_radius(&self) -> &LengthPercentage {
        &self.horizontal_radius
    }
    pub const fn vertical_radius(&self) -> &LengthPercentage {
        &self.vertical_radius
    }
    pub const fn rotation(&self) -> &ComputedNumeric {
        &self.rotation
    }
    pub const fn sweep(&self) -> ArcSweep {
        self.sweep
    }
    pub const fn size(&self) -> ArcSize {
        self.size
    }
}

/// Native computed commands, preserving kind, `to`/`by` and explicit controls.
///
/// `Smooth` with no control is quadratic; one explicit control makes it cubic.
/// Its implied control remains implicit. Geometry reflects the previous control
/// only within the corresponding quadratic/cubic family, otherwise using the
/// segment start. `Close` resets geometry's current point to the subpath start.
/// Neither reflection nor command execution is needed to interpolate operands.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeCommand {
    Move(Endpoint),
    Line(Endpoint),
    HorizontalLine(AxisCommand),
    VerticalLine(AxisCommand),
    Curve {
        end: Endpoint,
        controls: CurveControls,
    },
    Smooth {
        end: Endpoint,
        control: Option<ControlPoint>,
    },
    Arc(ArcCommand),
    Close,
}

/// An operand in a computed command, for constructor or sampling diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandComponent {
    Endpoint,
    FirstControl,
    SecondControl,
    HorizontalRadius,
    VerticalRadius,
    Rotation,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CommandList {
    initial: ComputedPosition,
    commands: Arc<[ShapeCommand]>,
    fill_rule: FillRule,
}

/// A computed `shape()` command list with at least one command after its origin.
///
/// The caller selects defaults, resolves authored positions and supplies numeric
/// operands. This type neither parses CSS nor executes path geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandShape {
    data: Arc<CommandList>,
}
impl CommandShape {
    pub fn new(
        initial: ComputedPosition,
        commands: Vec<ShapeCommand>,
        fill_rule: FillRule,
    ) -> Result<Self, ShapeError> {
        if commands.is_empty() {
            return Err(ShapeError::new(ShapeErrorKind::EmptyCommands));
        }
        Ok(Self {
            data: Arc::new(CommandList {
                initial,
                commands: commands.into(),
                fill_rule,
            }),
        })
    }
    pub fn initial(&self) -> &ComputedPosition {
        &self.data.initial
    }
    pub fn commands(&self) -> &[ShapeCommand] {
        &self.data.commands
    }
    pub fn fill_rule(&self) -> FillRule {
        self.data.fill_rule
    }
}

/// A computed `path()` list with an explicit initial absolute move.
///
/// All coordinate/radius operands must have the pure Length domain (canonical
/// pixels), and controls must use Origin for absolute commands or Start for
/// relative commands. Initial relative SVG moveto is supplied as its absolute
/// initial point. The supplying owner expands implicit SVG parameter groups,
/// retaining subsequent command kinds and relative/absolute forms. A list with
/// only the initial move is valid. No SVG string parser is provided here.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandPath {
    data: Arc<CommandList>,
}
impl CommandPath {
    pub fn new(
        initial: ComputedPosition,
        commands: Vec<ShapeCommand>,
        fill_rule: FillRule,
    ) -> Result<Self, ShapeError> {
        check_path_point(&initial).map_err(|error| error.at(ShapeComponent::InitialPoint))?;
        for (index, command) in commands.iter().enumerate() {
            command.validate_path(index)?;
        }
        Ok(Self {
            data: Arc::new(CommandList {
                initial,
                commands: commands.into(),
                fill_rule,
            }),
        })
    }
    pub fn initial(&self) -> &ComputedPosition {
        &self.data.initial
    }
    pub fn commands(&self) -> &[ShapeCommand] {
        &self.data.commands
    }
    pub fn fill_rule(&self) -> FillRule {
        self.data.fill_rule
    }
}

impl BasicShape {
    fn command_list(&self) -> Option<&Arc<CommandList>> {
        match self {
            Self::Shape(value) => Some(&value.data),
            Self::Path(value) => Some(&value.data),
            _ => None,
        }
    }
    pub(super) fn commands_compatible(&self, other: &Self) -> bool {
        match (self.command_list(), other.command_list()) {
            (Some(first), Some(second)) => {
                first.fill_rule == second.fill_rule
                    && first.commands.len() == second.commands.len()
                    && first
                        .commands
                        .iter()
                        .zip(second.commands.iter())
                        .all(|(a, b)| a.compatible(b))
            }
            _ => false,
        }
    }
    pub(super) fn interpolate_commands(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Option<Self>, ShapeError> {
        let (Some(first), Some(second)) = (self.command_list(), other.command_list()) else {
            return Ok(None);
        };
        // Preserve source identity at exact endpoints, but still promote mixed
        // Path/Shape pairs to Shape without evaluating any numeric components.
        let data = if progress.value() == 0. {
            Arc::clone(first)
        } else if progress.value() == 1. {
            Arc::clone(second)
        } else {
            let initial = first
                .initial
                .interpolate(&second.initial, progress)
                .map_err(|error| ShapeError::numeric(error).at(ShapeComponent::InitialPoint))?;
            let mut commands = Vec::new();
            commands
                .try_reserve_exact(first.commands.len())
                .map_err(|_| ShapeError::new(ShapeErrorKind::Capacity))?;
            for (index, (a, b)) in first
                .commands
                .iter()
                .zip(second.commands.iter())
                .enumerate()
            {
                let Some(command) = a.interpolate(b, progress, index)? else {
                    return Ok(None);
                };
                commands.push(command);
            }
            Arc::new(CommandList {
                initial,
                commands: commands.into(),
                fill_rule: first.fill_rule,
            })
        };
        Ok(Some(
            if matches!((self, other), (Self::Path(_), Self::Path(_))) {
                Self::Path(CommandPath { data })
            } else {
                Self::Shape(CommandShape { data })
            },
        ))
    }
}

fn at(index: usize, component: CommandComponent) -> impl FnOnce(ShapeError) -> ShapeError {
    move |error| error.at(ShapeComponent::Command { index, component })
}
fn check_path_numeric(value: &ComputedNumeric) -> Result<(), ShapeError> {
    if value.numeric_type() == NumericType::Dimension(DimensionKind::Length) {
        Ok(())
    } else {
        Err(ShapeError::invalid_type(value.numeric_type()))
    }
}
fn check_path_point(point: &ComputedPosition) -> Result<(), ShapeError> {
    check_path_numeric(point.x())?;
    check_path_numeric(point.y())
}
fn check_path_control(point: &ControlPoint, expected: ControlAnchor) -> Result<(), ShapeError> {
    if point.anchor != expected {
        return Err(ShapeError::new(ShapeErrorKind::InvalidPathControlAnchor));
    }
    check_path_point(&point.offset)
}
impl ShapeCommand {
    fn compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Move(a), Self::Move(b)) | (Self::Line(a), Self::Line(b)) => a.same_form(b),
            (Self::HorizontalLine(a), Self::HorizontalLine(b))
            | (Self::VerticalLine(a), Self::VerticalLine(b)) => a.same_form(b),
            (
                Self::Curve {
                    end: a,
                    controls: ac,
                },
                Self::Curve {
                    end: b,
                    controls: bc,
                },
            ) => a.same_form(b) && ac.compatible(bc),
            (
                Self::Smooth {
                    end: a,
                    control: ac,
                },
                Self::Smooth {
                    end: b,
                    control: bc,
                },
            ) => {
                a.same_form(b)
                    && match (ac, bc) {
                        (None, None) => true,
                        (Some(a), Some(b)) => a.anchor == b.anchor,
                        _ => false,
                    }
            }
            (Self::Arc(a), Self::Arc(b)) => a.end.same_form(&b.end),
            (Self::Close, Self::Close) => true,
            _ => false,
        }
    }
    fn validate_path(&self, index: usize) -> Result<(), ShapeError> {
        use CommandComponent as C;
        let point =
            |end: &Endpoint| check_path_point(end.coordinates()).map_err(at(index, C::Endpoint));
        match self {
            Self::Move(end) | Self::Line(end) => point(end),
            Self::HorizontalLine(axis) | Self::VerticalLine(axis) => {
                check_path_numeric(axis.offset().numeric()).map_err(at(index, C::Endpoint))
            }
            Self::Curve { end, controls } => {
                point(end)?;
                match controls {
                    CurveControls::Quadratic(control) => {
                        check_path_control(control, end.path_anchor())
                            .map_err(at(index, C::FirstControl))
                    }
                    CurveControls::Cubic(first, second) => {
                        check_path_control(first, end.path_anchor())
                            .map_err(at(index, C::FirstControl))?;
                        check_path_control(second, end.path_anchor())
                            .map_err(at(index, C::SecondControl))
                    }
                }
            }
            Self::Smooth { end, control } => {
                point(end)?;
                if let Some(control) = control {
                    check_path_control(control, end.path_anchor())
                        .map_err(at(index, C::FirstControl))?;
                }
                Ok(())
            }
            Self::Arc(arc) => {
                point(&arc.end)?;
                check_path_numeric(arc.horizontal_radius.numeric())
                    .map_err(at(index, C::HorizontalRadius))?;
                check_path_numeric(arc.vertical_radius.numeric())
                    .map_err(at(index, C::VerticalRadius))
            }
            Self::Close => Ok(()),
        }
    }
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
        index: usize,
    ) -> Result<Option<Self>, ShapeError> {
        use CommandComponent as C;
        let endpoint =
            |a: &Endpoint, b: &Endpoint| a.interpolate(b, progress).map_err(at(index, C::Endpoint));
        let control = |a: &ControlPoint, b: &ControlPoint, component| {
            a.interpolate(b, progress).map_err(at(index, component))
        };
        Ok(Some(match (self, other) {
            (Self::Move(a), Self::Move(b)) => Self::Move(endpoint(a, b)?),
            (Self::Line(a), Self::Line(b)) => Self::Line(endpoint(a, b)?),
            (Self::HorizontalLine(a), Self::HorizontalLine(b)) => {
                Self::HorizontalLine(a.interpolate(b, progress).map_err(at(index, C::Endpoint))?)
            }
            (Self::VerticalLine(a), Self::VerticalLine(b)) => {
                Self::VerticalLine(a.interpolate(b, progress).map_err(at(index, C::Endpoint))?)
            }
            (
                Self::Curve {
                    end: a,
                    controls: ac,
                },
                Self::Curve {
                    end: b,
                    controls: bc,
                },
            ) => {
                let controls = match (ac, bc) {
                    (CurveControls::Quadratic(a), CurveControls::Quadratic(b)) => {
                        CurveControls::Quadratic(control(a, b, C::FirstControl)?)
                    }
                    (CurveControls::Cubic(a, b), CurveControls::Cubic(c, d)) => {
                        CurveControls::Cubic(
                            control(a, c, C::FirstControl)?,
                            control(b, d, C::SecondControl)?,
                        )
                    }
                    _ => return Ok(None),
                };
                Self::Curve {
                    end: endpoint(a, b)?,
                    controls,
                }
            }
            (
                Self::Smooth {
                    end: a,
                    control: ac,
                },
                Self::Smooth {
                    end: b,
                    control: bc,
                },
            ) => {
                let explicit = match (ac, bc) {
                    (None, None) => None,
                    (Some(a), Some(b)) => Some(control(a, b, C::FirstControl)?),
                    _ => return Ok(None),
                };
                Self::Smooth {
                    end: endpoint(a, b)?,
                    control: explicit,
                }
            }
            (Self::Arc(a), Self::Arc(b)) => Self::Arc(ArcCommand {
                end: endpoint(&a.end, &b.end)?,
                horizontal_radius: a
                    .horizontal_radius
                    .interpolate(&b.horizontal_radius, progress)
                    .map_err(at(index, C::HorizontalRadius))?,
                vertical_radius: a
                    .vertical_radius
                    .interpolate(&b.vertical_radius, progress)
                    .map_err(at(index, C::VerticalRadius))?,
                rotation: a
                    .rotation
                    .interpolate(&b.rotation, progress)
                    .map_err(ShapeError::numeric)
                    .map_err(at(index, C::Rotation))?,
                sweep: if progress.value() <= 0. {
                    a.sweep
                } else if progress.value() >= 1. {
                    b.sweep
                } else if a.sweep == b.sweep {
                    a.sweep
                } else {
                    ArcSweep::Cw
                },
                size: if progress.value() <= 0. {
                    a.size
                } else if progress.value() >= 1. {
                    b.size
                } else if a.size == b.size {
                    a.size
                } else {
                    ArcSize::Large
                },
            }),
            (Self::Close, Self::Close) => Self::Close,
            _ => return Ok(None),
        }))
    }
}
