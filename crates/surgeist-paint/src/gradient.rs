use crate::{
    PaintError, PaintOptions, PaintPhase, PaintReport, ResourceKind, WorkKind,
    accounting::{Meter, OperationKind, PhaseMeter, apply, bytes, charge, finish, setup_error},
};
use surgeist_layout::{Point, Size};
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

/// Physical direction of a linear gradient in a box whose y axis points down.
/// Angles run clockwise from up; raw angle validation belongs to the operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LinearGradientDirection {
    AngleDegrees(f64),
    Top,
    Right,
    #[default]
    Bottom,
    Left,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Invalid concrete input or a line that cannot retain finite, noncollapsed
/// positive geometry in f64. Zero dimensions are valid inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientGeometryError {
    InvalidWidth,
    InvalidHeight,
    NonFiniteAngle,
    UnrepresentableLine,
}

/// Validated, box-local directed line. Placement is a consumer transform.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearGradientLine {
    start: Point<f64>,
    end: Point<f64>,
    length: f64,
}
impl LinearGradientLine {
    pub const fn start(self) -> Point<f64> {
        self.start
    }
    pub const fn end(self) -> Point<f64> {
        self.end
    }
    pub const fn length(self) -> f64 {
        self.length
    }
}

/// One complete line or typed refusal, with the operation's accepted prefix.
#[derive(Debug)]
pub struct LinearGradientOutcome {
    pub result: Result<LinearGradientLine, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves centered CSS linear-gradient geometry for a concrete physical size.
/// Endpoints may lie outside the box. Only actual zero projection produces a
/// zero line; numerical overflow or positive collapse produces a typed refusal.
pub fn resolve_linear_gradient(
    size: Size<f64>,
    direction: LinearGradientDirection,
    options: PaintOptions,
) -> LinearGradientOutcome {
    let mut meter = match options.operation(OperationKind::LinearGradient) {
        Ok(meter) => meter,
        Err(error) => {
            return LinearGradientOutcome {
                result: Err(error),
                report: None,
            };
        }
    };
    let root = meter.root();
    let scopes = match setup_scopes(&mut meter) {
        Ok(scopes) => scopes,
        Err(error) => {
            let (result, report) = finish(meter, Err(error));
            return LinearGradientOutcome { result, report };
        }
    };
    let mut retained = false;
    let result = (|| {
        meter
            .with_phase(scopes[0], |p| validate(p, scopes[0], size, direction))
            .map_err(setup_error)??;
        let line = meter
            .with_phase(scopes[1], |p| {
                let mut work = GeometryWork {
                    meter: p,
                    scope: scopes[1],
                };
                let axis = resolve_direction(&mut work, size, direction)?;
                resolve_line(&mut work, size, axis)
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[2], |p| {
                // This fixed value owns its real size, not fictitious heap backing.
                apply(
                    p,
                    root,
                    Change::Acquire(
                        ResourceKind::RetainedResultBytes,
                        bytes::<LinearGradientLine>(1)?,
                    ),
                )?;
                retained = true;
                // One complete returned record. A refusal releases its admitted owner.
                charge(p, scopes[2], WorkKind::OutputRecord, 1)?;
                Ok(line)
            })
            .map_err(setup_error)?
    })();
    let cleanup = meter
        .with_phase(scopes[3], |p| {
            if result.is_err() && retained {
                apply(
                    p,
                    root,
                    Change::Release(
                        ResourceKind::RetainedResultBytes,
                        bytes::<LinearGradientLine>(1)?,
                    ),
                )?;
            }
            Ok(())
        })
        .map_err(setup_error)
        .and_then(|done| done);
    let result = result.and_then(|value| cleanup.map(|()| value));
    let (result, report) = finish(meter, result);
    LinearGradientOutcome { result, report }
}

fn setup_scopes(meter: &mut Meter) -> Result<[Scope; 4], PaintError> {
    let mut scopes = [meter.root(); 4];
    for (slot, phase) in scopes.iter_mut().zip([
        PaintPhase::GradientValidation,
        PaintPhase::GradientGeometry,
        PaintPhase::Publication,
        PaintPhase::Cleanup,
    ]) {
        *slot = meter.child(
            meter.root(),
            phase,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
    }
    Ok(scopes)
}

fn validate(
    p: &mut PhaseMeter<'_, '_>,
    scope: Scope,
    size: Size<f64>,
    direction: LinearGradientDirection,
) -> Result<(), PaintError> {
    // One validation unit per input scalar's full domain check, plus one
    // direction-discriminant inspection. An angle contributes another scalar.
    charge(p, scope, WorkKind::GradientValidation, 1)?;
    if !size.width.is_finite() || size.width < 0.0 {
        return Err(PaintError::Gradient(GradientGeometryError::InvalidWidth));
    }
    charge(p, scope, WorkKind::GradientValidation, 1)?;
    if !size.height.is_finite() || size.height < 0.0 {
        return Err(PaintError::Gradient(GradientGeometryError::InvalidHeight));
    }
    charge(p, scope, WorkKind::GradientValidation, 1)?;
    if let LinearGradientDirection::AngleDegrees(angle) = direction {
        charge(p, scope, WorkKind::GradientValidation, 1)?;
        if !angle.is_finite() {
            return Err(PaintError::Gradient(GradientGeometryError::NonFiniteAngle));
        }
    }
    Ok(())
}

struct GeometryWork<'a, 'b, 'operation> {
    meter: &'a mut PhaseMeter<'b, 'operation>,
    scope: Scope,
}
impl GeometryWork<'_, '_, '_> {
    // Every call admits exactly one action before invoking it. Direction units
    // are enum/scalar dispatches, scalar arithmetic (including a transcendental),
    // or scalar inspections. Coordinate units are scalar arithmetic or scalar
    // inspections. A fused multiply-add and hypot are each one scalar operation.
    // Copies, point/record assembly, boolean combination and control flow are not
    // scalar geometry work. Repeated inspections invoke and charge again.
    fn direction<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientDirection, 1)?;
        Ok(action())
    }
    fn coordinate<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientCoordinate, 1)?;
        Ok(action())
    }
}

#[derive(Clone, Copy)]
enum Cardinal {
    Top,
    Right,
    Bottom,
    Left,
}
enum Axis {
    Cardinal(Cardinal),
    Vector { x: f64, y: f64 },
}

fn resolve_direction(
    work: &mut GeometryWork<'_, '_, '_>,
    size: Size<f64>,
    direction: LinearGradientDirection,
) -> Result<Axis, PaintError> {
    work.direction(|| ())?;
    match direction {
        LinearGradientDirection::Top => Ok(Axis::Cardinal(Cardinal::Top)),
        LinearGradientDirection::Right => Ok(Axis::Cardinal(Cardinal::Right)),
        LinearGradientDirection::Bottom => Ok(Axis::Cardinal(Cardinal::Bottom)),
        LinearGradientDirection::Left => Ok(Axis::Cardinal(Cardinal::Left)),
        LinearGradientDirection::TopLeft => corner_axis(work, size, -1.0, -1.0),
        LinearGradientDirection::TopRight => corner_axis(work, size, 1.0, -1.0),
        LinearGradientDirection::BottomLeft => corner_axis(work, size, -1.0, 1.0),
        LinearGradientDirection::BottomRight => corner_axis(work, size, 1.0, 1.0),
        LinearGradientDirection::AngleDegrees(angle) => {
            let zero_width = work.direction(|| size.width == 0.0)?;
            let zero_height = work.direction(|| size.height == 0.0)?;
            if zero_width && zero_height {
                // Validation already established a finite angle. Every angle
                // projects to exactly zero in an empty box, so an unused tiny
                // radian conversion must not turn that true zero into a refusal.
                Ok(Axis::Cardinal(Cardinal::Top))
            } else {
                angle_axis(work, angle)
            }
        }
    }
}

fn corner_axis(
    work: &mut GeometryWork<'_, '_, '_>,
    size: Size<f64>,
    x_sign: f64,
    y_sign: f64,
) -> Result<Axis, PaintError> {
    // (H,W), not (W,H): the axis is perpendicular to the neighboring-corner
    // diagonal. Scaling first keeps the hypotenuse finite even for maximum sizes.
    let scale = work.direction(|| size.width.max(size.height))?;
    let (x, y) = if work.direction(|| scale == 0.0)? {
        (
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        )
    } else {
        let x = work.direction(|| size.height / scale)?;
        let y = work.direction(|| size.width / scale)?;
        let positive_height = work.direction(|| size.height > 0.0)?;
        let positive_width = work.direction(|| size.width > 0.0)?;
        let lost_x = work.direction(|| x == 0.0)?;
        let lost_y = work.direction(|| y == 0.0)?;
        if (positive_height && lost_x) || (positive_width && lost_y) {
            return unrepresentable();
        }
        let norm = work.direction(|| x.hypot(y))?;
        let x = work.direction(|| x / norm)?;
        let y = work.direction(|| y / norm)?;
        let lost_x = work.direction(|| x == 0.0)?;
        let lost_y = work.direction(|| y == 0.0)?;
        if (positive_height && lost_x) || (positive_width && lost_y) {
            return unrepresentable();
        }
        (x, y)
    };
    let x = work.direction(|| x.copysign(x_sign))?;
    let y = work.direction(|| y.copysign(y_sign))?;
    Ok(Axis::Vector { x, y })
}

fn angle_axis(work: &mut GeometryWork<'_, '_, '_>, angle: f64) -> Result<Axis, PaintError> {
    // Signed remainder retains tiny negative angles: adding a full turn could
    // round them onto a cardinal. Reduce before converting degrees to radians.
    let reduced = work.direction(|| angle % 360.0)?;
    let cardinal = work.direction(|| match reduced {
        0.0 => Some(Cardinal::Top),
        90.0 | -270.0 => Some(Cardinal::Right),
        180.0 | -180.0 => Some(Cardinal::Bottom),
        270.0 | -90.0 => Some(Cardinal::Left),
        _ => None,
    })?;
    if let Some(cardinal) = cardinal {
        return Ok(Axis::Cardinal(cardinal));
    }
    // Use a residual within 45 degrees of a cardinal, so trigonometry never
    // subtracts a tiny result from a rounded pi/2 or pi.
    let magnitude = work.direction(|| reduced.abs())?;
    let (residual, quadrant) = if work.direction(|| magnitude <= 45.0)? {
        (magnitude, 0)
    } else if work.direction(|| magnitude <= 135.0)? {
        (work.direction(|| magnitude - 90.0)?, 1)
    } else if work.direction(|| magnitude <= 225.0)? {
        (work.direction(|| magnitude - 180.0)?, 2)
    } else if work.direction(|| magnitude <= 315.0)? {
        (work.direction(|| magnitude - 270.0)?, 3)
    } else {
        (work.direction(|| magnitude - 360.0)?, 0)
    };
    let radians = work.direction(|| residual.to_radians())?;
    if work.direction(|| radians == 0.0)? {
        return unrepresentable();
    }
    let sin = work.direction(|| radians.sin())?;
    if work.direction(|| sin == 0.0)? {
        return unrepresentable();
    }
    let cos = work.direction(|| radians.cos())?;
    work.direction(|| ())?;
    let (x, y) = match quadrant {
        0 => (sin, work.direction(|| -cos)?),
        1 => (cos, sin),
        2 => (work.direction(|| -sin)?, cos),
        _ => (work.direction(|| -cos)?, work.direction(|| -sin)?),
    };
    let x = if work.direction(|| reduced < 0.0)? {
        work.direction(|| -x)?
    } else {
        x
    };
    Ok(Axis::Vector { x, y })
}

fn resolve_line(
    work: &mut GeometryWork<'_, '_, '_>,
    size: Size<f64>,
    axis: Axis,
) -> Result<LinearGradientLine, PaintError> {
    let center = Point::new(
        work.coordinate(|| size.width * 0.5)?,
        work.coordinate(|| size.height * 0.5)?,
    );
    let line = match axis {
        // Keep exact original edges, including the smallest positive subnormal:
        // computing half a positive dimension and doubling it can fabricate zero.
        Axis::Cardinal(Cardinal::Top) => LinearGradientLine {
            start: Point::new(center.x, size.height),
            end: Point::new(center.x, 0.0),
            length: size.height,
        },
        Axis::Cardinal(Cardinal::Right) => LinearGradientLine {
            start: Point::new(0.0, center.y),
            end: Point::new(size.width, center.y),
            length: size.width,
        },
        Axis::Cardinal(Cardinal::Bottom) => LinearGradientLine {
            start: Point::new(center.x, 0.0),
            end: Point::new(center.x, size.height),
            length: size.height,
        },
        Axis::Cardinal(Cardinal::Left) => LinearGradientLine {
            start: Point::new(size.width, center.y),
            end: Point::new(0.0, center.y),
            length: size.width,
        },
        Axis::Vector { x, y } => projected_line(work, size, center, x, y)?,
    };
    inspect_line(work, line)?;
    Ok(line)
}

fn projected_line(
    work: &mut GeometryWork<'_, '_, '_>,
    size: Size<f64>,
    center: Point<f64>,
    x: f64,
    y: f64,
) -> Result<LinearGradientLine, PaintError> {
    let abs_x = work.coordinate(|| x.abs())?;
    let abs_y = work.coordinate(|| y.abs())?;
    let positive_width = work.coordinate(|| size.width > 0.0)?;
    let positive_height = work.coordinate(|| size.height > 0.0)?;
    let nonzero_x = work.coordinate(|| abs_x > 0.0)?;
    let nonzero_y = work.coordinate(|| abs_y > 0.0)?;
    if !(positive_width && nonzero_x || positive_height && nonzero_y) {
        return Ok(LinearGradientLine {
            start: center,
            end: center,
            length: 0.0,
        });
    }
    let height_projection = work.coordinate(|| size.height * abs_y)?;
    let length = work.coordinate(|| size.width.mul_add(abs_x, height_projection))?;
    let finite = work.coordinate(|| length.is_finite())?;
    let positive = work.coordinate(|| length > 0.0)?;
    if !finite || !positive {
        return unrepresentable();
    }
    let half_length = work.coordinate(|| length * 0.5)?;
    let negative_half = work.coordinate(|| -half_length)?;
    // Fused endpoint calculation avoids a separately rounded offset followed by
    // cancellation against the center; halving first avoids offset overflow.
    let start = Point::new(
        work.coordinate(|| negative_half.mul_add(x, center.x))?,
        work.coordinate(|| negative_half.mul_add(y, center.y))?,
    );
    let end = Point::new(
        work.coordinate(|| half_length.mul_add(x, center.x))?,
        work.coordinate(|| half_length.mul_add(y, center.y))?,
    );
    Ok(LinearGradientLine { start, end, length })
}

fn inspect_line(
    work: &mut GeometryWork<'_, '_, '_>,
    line: LinearGradientLine,
) -> Result<(), PaintError> {
    // Each actual inspected scalar is charged, even though previous steps have
    // already inspected the length. Stop on the first observed numeric refusal.
    for value in [
        line.start.x,
        line.start.y,
        line.end.x,
        line.end.y,
        line.length,
    ] {
        if !work.coordinate(|| value.is_finite())? {
            return unrepresentable();
        }
    }
    if work.coordinate(|| line.length > 0.0)? {
        let same_x = work.coordinate(|| line.start.x == line.end.x)?;
        let same_y = work.coordinate(|| line.start.y == line.end.y)?;
        if same_x && same_y {
            return unrepresentable();
        }
    }
    Ok(())
}

fn unrepresentable<T>() -> Result<T, PaintError> {
    Err(PaintError::Gradient(
        GradientGeometryError::UnrepresentableLine,
    ))
}
