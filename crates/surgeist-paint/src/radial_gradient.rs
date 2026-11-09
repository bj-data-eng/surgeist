use crate::{
    GradientPosition, GradientPositionComponent, PaintError, PaintOptions, PaintPhase, PaintReport,
    ResourceKind, WorkKind,
    accounting::{Meter, OperationKind, PhaseMeter, apply, bytes, charge, finish, setup_error},
};
use surgeist_layout::{PhysicalAxis, Point, Size};
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

/// Physical side/corner extent of a radial ending shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadialGradientExtent {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
}

/// Raw coupled shape/size request. Ellipse fractions use the matching full box
/// dimensions; a circle radius is an already-computed absolute length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadialGradientSize {
    CircleRadius(f64),
    EllipseRadii(Size<GradientPosition>),
    CircleExtent(RadialGradientExtent),
    EllipseExtent(RadialGradientExtent),
}
impl Default for RadialGradientSize {
    fn default() -> Self {
        Self::EllipseExtent(RadialGradientExtent::FarthestCorner)
    }
}

/// Native input or numerical-representation refusal. CSS range evaluation and
/// authored declaration validity remain with the supplying adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadialGradientError {
    InvalidWidth,
    InvalidHeight,
    NonFiniteCenter {
        axis: PhysicalAxis,
    },
    InvalidCircleRadius,
    NonFiniteRadiusComponent {
        axis: PhysicalAxis,
        component: GradientPositionComponent,
    },
    NegativeRadius {
        axis: PhysicalAxis,
    },
    UnrepresentableGeometry,
}

/// Resolved geometry or the ordered radial semantic case. Only Ellipse supplies
/// positive finite radii and a normal rightward ray basis (its horizontal radius).
/// Zero cases do not invent replacement radii or a universal used-stop basis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadialGradientCase {
    Ellipse { radii: Size<f64> },
    ZeroCircle,
    ZeroWidth,
    ZeroHeight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ResolvedCase {
    Ellipse(Size<f64>),
    ZeroCircle,
    ZeroWidth,
    ZeroHeight,
}

/// Complete box-local finite center and radial geometry/case. Construction is
/// private; returned case views cannot be passed back as invariant proofs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadialGradientGeometry {
    center: Point<f64>,
    case: ResolvedCase,
}
impl RadialGradientGeometry {
    pub const fn center(self) -> Point<f64> {
        self.center
    }
    pub const fn case(self) -> RadialGradientCase {
        match self.case {
            ResolvedCase::Ellipse(radii) => RadialGradientCase::Ellipse { radii },
            ResolvedCase::ZeroCircle => RadialGradientCase::ZeroCircle,
            ResolvedCase::ZeroWidth => RadialGradientCase::ZeroWidth,
            ResolvedCase::ZeroHeight => RadialGradientCase::ZeroHeight,
        }
    }
}

/// One complete result or typed refusal with the operation's accepted prefix.
#[derive(Debug)]
pub struct RadialGradientOutcome {
    pub result: Result<RadialGradientGeometry, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves native radial ending geometry and ordered zero cases. None selects
/// box center; Some is an already-used physical point, including outside points.
/// This evaluates neither CSS Position/calculation trees nor degenerate fields.
pub fn resolve_radial_gradient(
    box_size: Size<f64>,
    center: Option<Point<f64>>,
    ending_size: RadialGradientSize,
    options: PaintOptions,
) -> RadialGradientOutcome {
    let mut meter = match options.operation(OperationKind::RadialGradient) {
        Ok(meter) => meter,
        Err(error) => {
            return RadialGradientOutcome {
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
            return RadialGradientOutcome { result, report };
        }
    };
    let mut retained = false;
    let result = (|| {
        meter
            .with_phase(scopes[0], |p| {
                validate(
                    &mut RadialWork::new(p, scopes[0]),
                    box_size,
                    center,
                    ending_size,
                )
            })
            .map_err(setup_error)??;
        let geometry = meter
            .with_phase(scopes[1], |p| {
                geometry(
                    &mut RadialWork::new(p, scopes[1]),
                    box_size,
                    center,
                    ending_size,
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[2], |p| {
                apply(
                    p,
                    root,
                    Change::Acquire(
                        ResourceKind::RetainedResultBytes,
                        bytes::<RadialGradientGeometry>(1)?,
                    ),
                )?;
                retained = true;
                charge(p, scopes[2], WorkKind::OutputRecord, 1)?;
                Ok(geometry)
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
                        bytes::<RadialGradientGeometry>(1)?,
                    ),
                )?;
            }
            Ok(())
        })
        .map_err(setup_error)
        .and_then(|done| done);
    let result = result.and_then(|value| cleanup.map(|()| value));
    let (result, report) = finish(meter, result);
    RadialGradientOutcome { result, report }
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

struct RadialWork<'a, 'b, 'operation> {
    meter: &'a mut PhaseMeter<'b, 'operation>,
    scope: Scope,
}
impl<'a, 'b, 'operation> RadialWork<'a, 'b, 'operation> {
    fn new(meter: &'a mut PhaseMeter<'b, 'operation>, scope: Scope) -> Self {
        Self { meter, scope }
    }
    // One validation unit admits an input scalar's full domain check or one
    // input-option/size discriminant inspection. No input is certified by a copy.
    fn validation<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientValidation, 1)?;
        Ok(action())
    }
    // One numeric unit admits one scalar operation or inspection, including
    // FMA/hypot, comparison, conversion, bit extraction, shift, normalization,
    // and exact-proof arithmetic. A fixed primitive field mask/extraction is
    // one scalar action, not CPU instructions. Each repeated check charges anew.
    // Copies, enum dispatch, boolean control and result assembly are not extra
    // numeric work. There is no hidden variable traversal in the proof.
    fn numeric<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientCoordinate, 1)?;
        Ok(action())
    }
}

fn validate(
    work: &mut RadialWork<'_, '_, '_>,
    box_size: Size<f64>,
    center: Option<Point<f64>>,
    ending_size: RadialGradientSize,
) -> Result<(), PaintError> {
    if !work.validation(|| box_size.width.is_finite() && box_size.width >= 0.0)? {
        return Err(RadialGradientError::InvalidWidth.into());
    }
    if !work.validation(|| box_size.height.is_finite() && box_size.height >= 0.0)? {
        return Err(RadialGradientError::InvalidHeight.into());
    }
    work.validation(|| ())?;
    if let Some(center) = center {
        if !work.validation(|| center.x.is_finite())? {
            return Err(RadialGradientError::NonFiniteCenter {
                axis: PhysicalAxis::Horizontal,
            }
            .into());
        }
        if !work.validation(|| center.y.is_finite())? {
            return Err(RadialGradientError::NonFiniteCenter {
                axis: PhysicalAxis::Vertical,
            }
            .into());
        }
    }
    work.validation(|| ())?;
    match ending_size {
        RadialGradientSize::CircleRadius(radius) => {
            if !work.validation(|| radius.is_finite() && radius >= 0.0)? {
                return Err(RadialGradientError::InvalidCircleRadius.into());
            }
        }
        RadialGradientSize::EllipseRadii(radii) => {
            validate_position(work, PhysicalAxis::Horizontal, radii.width)?;
            validate_position(work, PhysicalAxis::Vertical, radii.height)?;
        }
        RadialGradientSize::CircleExtent(_) | RadialGradientSize::EllipseExtent(_) => {}
    }
    Ok(())
}

fn validate_position(
    work: &mut RadialWork<'_, '_, '_>,
    axis: PhysicalAxis,
    position: GradientPosition,
) -> Result<(), PaintError> {
    if !work.validation(|| position.length.is_finite())? {
        return Err(RadialGradientError::NonFiniteRadiusComponent {
            axis,
            component: GradientPositionComponent::Length,
        }
        .into());
    }
    if !work.validation(|| position.fraction.is_finite())? {
        return Err(RadialGradientError::NonFiniteRadiusComponent {
            axis,
            component: GradientPositionComponent::Fraction,
        }
        .into());
    }
    Ok(())
}

fn geometry(
    work: &mut RadialWork<'_, '_, '_>,
    box_size: Size<f64>,
    center: Option<Point<f64>>,
    ending_size: RadialGradientSize,
) -> Result<RadialGradientGeometry, PaintError> {
    let center = match center {
        Some(center) => center,
        None => Point::new(
            center_half(work, box_size.width)?,
            center_half(work, box_size.height)?,
        ),
    };
    let case = match ending_size {
        RadialGradientSize::CircleRadius(radius) => {
            if work.numeric(|| radius == 0.0)? {
                ResolvedCase::ZeroCircle
            } else {
                positive_ellipse(work, radius, radius)?
            }
        }
        RadialGradientSize::EllipseRadii(radii) => {
            let x = affine_radius(work, radii.width, box_size.width, PhysicalAxis::Horizontal)?;
            let y = affine_radius(work, radii.height, box_size.height, PhysicalAxis::Vertical)?;
            if work.numeric(|| x == 0.0)? {
                ResolvedCase::ZeroWidth
            } else if work.numeric(|| y == 0.0)? {
                ResolvedCase::ZeroHeight
            } else {
                positive_ellipse(work, x, y)?
            }
        }
        RadialGradientSize::CircleExtent(extent) => circle_extent(work, box_size, center, extent)?,
        RadialGradientSize::EllipseExtent(extent) => {
            ellipse_extent(work, box_size, center, extent)?
        }
    };
    Ok(RadialGradientGeometry { center, case })
}

fn center_half(work: &mut RadialWork<'_, '_, '_>, dimension: f64) -> Result<f64, PaintError> {
    let half = work.numeric(|| dimension * 0.5)?;
    if work.numeric(|| dimension > 0.0)? && work.numeric(|| half == 0.0)? {
        return unrepresentable();
    }
    Ok(half)
}

fn positive_ellipse(
    work: &mut RadialWork<'_, '_, '_>,
    x: f64,
    y: f64,
) -> Result<ResolvedCase, PaintError> {
    if !work.numeric(|| x.is_finite())? || !work.numeric(|| x > 0.0)? {
        return unrepresentable();
    }
    if !work.numeric(|| y.is_finite())? || !work.numeric(|| y > 0.0)? {
        return unrepresentable();
    }
    Ok(ResolvedCase::Ellipse(Size::new(x, y)))
}

fn affine_radius(
    work: &mut RadialWork<'_, '_, '_>,
    position: GradientPosition,
    dimension: f64,
    axis: PhysicalAxis,
) -> Result<f64, PaintError> {
    let radius = work.numeric(|| position.fraction.mul_add(dimension, position.length))?;
    if !work.numeric(|| radius.is_finite())? {
        return unrepresentable();
    }
    if work.numeric(|| radius < 0.0)? {
        return Err(RadialGradientError::NegativeRadius { axis }.into());
    }
    if work.numeric(|| radius == 0.0)?
        && !exact_affine_zero(work, position.fraction, dimension, position.length)?
    {
        return unrepresentable();
    }
    Ok(radius)
}

fn near(extent: RadialGradientExtent) -> bool {
    matches!(
        extent,
        RadialGradientExtent::ClosestSide | RadialGradientExtent::ClosestCorner
    )
}
fn corner(extent: RadialGradientExtent) -> bool {
    matches!(
        extent,
        RadialGradientExtent::ClosestCorner | RadialGradientExtent::FarthestCorner
    )
}

fn zero_axis(
    work: &mut RadialWork<'_, '_, '_>,
    dimension: f64,
    center: f64,
    near: bool,
) -> Result<bool, PaintError> {
    if near {
        Ok(work.numeric(|| center == 0.0)? || work.numeric(|| center == dimension)?)
    } else {
        Ok(work.numeric(|| dimension == 0.0)? && work.numeric(|| center == 0.0)?)
    }
}

fn side_distance(
    work: &mut RadialWork<'_, '_, '_>,
    dimension: f64,
    center: f64,
    near: bool,
) -> Result<f64, PaintError> {
    // Choose the needed distance before subtracting: MAX-(-MAX) is irrelevant
    // to a closest-side request whose selected distance is exactly MAX.
    let distance = if work.numeric(|| center < 0.0)? {
        if near {
            work.numeric(|| -center)?
        } else {
            work.numeric(|| dimension - center)?
        }
    } else if work.numeric(|| center > dimension)? {
        if near {
            work.numeric(|| center - dimension)?
        } else {
            center
        }
    } else {
        let opposite = work.numeric(|| dimension - center)?;
        if near {
            work.numeric(|| center.min(opposite))?
        } else {
            work.numeric(|| center.max(opposite))?
        }
    };
    if !work.numeric(|| distance.is_finite())? {
        return unrepresentable();
    }
    Ok(distance)
}

fn circle_extent(
    work: &mut RadialWork<'_, '_, '_>,
    size: Size<f64>,
    center: Point<f64>,
    extent: RadialGradientExtent,
) -> Result<ResolvedCase, PaintError> {
    let near = near(extent);
    let zero_x = zero_axis(work, size.width, center.x, near)?;
    if extent == RadialGradientExtent::ClosestSide {
        if zero_x || zero_axis(work, size.height, center.y, near)? {
            return Ok(ResolvedCase::ZeroCircle);
        }
    } else if zero_x && zero_axis(work, size.height, center.y, near)? {
        return Ok(ResolvedCase::ZeroCircle);
    }
    let x = side_distance(work, size.width, center.x, near)?;
    let y = side_distance(work, size.height, center.y, near)?;
    let radius = if corner(extent) {
        work.numeric(|| x.hypot(y))?
    } else if near {
        work.numeric(|| x.min(y))?
    } else {
        work.numeric(|| x.max(y))?
    };
    positive_ellipse(work, radius, radius)
}

fn ellipse_extent(
    work: &mut RadialWork<'_, '_, '_>,
    size: Size<f64>,
    center: Point<f64>,
    extent: RadialGradientExtent,
) -> Result<ResolvedCase, PaintError> {
    let near = near(extent);
    // Exact zero-axis cases precede unused distance, aspect and corner arithmetic.
    if zero_axis(work, size.width, center.x, near)? {
        return Ok(ResolvedCase::ZeroWidth);
    }
    if zero_axis(work, size.height, center.y, near)? {
        return Ok(ResolvedCase::ZeroHeight);
    }
    let x = side_distance(work, size.width, center.x, near)?;
    let y = side_distance(work, size.height, center.y, near)?;
    let (x, y) = if corner(extent) {
        (
            work.numeric(|| x * std::f64::consts::SQRT_2)?,
            work.numeric(|| y * std::f64::consts::SQRT_2)?,
        )
    } else {
        (x, y)
    };
    positive_ellipse(work, x, y)
}

#[derive(Clone, Copy)]
struct Dyadic {
    significand: u64,
    exponent: i32,
}

/// Decodes a finite nonzero f64 as odd-significand * 2^exponent. The input
/// precondition is established by validation and exact_affine_zero's zero guards.
fn dyadic(work: &mut RadialWork<'_, '_, '_>, value: f64) -> Result<Dyadic, PaintError> {
    let bits = work.numeric(|| value.to_bits())?;
    let exponent_field = work.numeric(|| (bits >> 52) & 0x7ff)?;
    let fraction = work.numeric(|| bits & 0x000f_ffff_ffff_ffff)?;
    let (significand, exponent) = if work.numeric(|| exponent_field == 0)? {
        (fraction, -1074)
    } else {
        let significand = work.numeric(|| fraction | (1_u64 << 52))?;
        let exponent = work.numeric(|| i32::try_from(exponent_field).expect("11-bit exponent"))?;
        (significand, work.numeric(|| exponent - 1075)?)
    };
    let zeros = work.numeric(|| significand.trailing_zeros())?;
    let significand = work.numeric(|| significand >> zeros)?;
    let zeros = work.numeric(|| i32::try_from(zeros).expect("nonzero u64 trailing zeros"))?;
    let exponent = work.numeric(|| exponent + zeros)?;
    Ok(Dyadic {
        significand,
        exponent,
    })
}

fn exact_affine_zero(
    work: &mut RadialWork<'_, '_, '_>,
    a: f64,
    b: f64,
    c: f64,
) -> Result<bool, PaintError> {
    // No product, decode, or zero shift is needed for a zero factor.
    if work.numeric(|| a == 0.0)? || work.numeric(|| b == 0.0)? {
        return work.numeric(|| c == 0.0);
    }
    if work.numeric(|| c == 0.0)? {
        return Ok(false);
    }
    let negative_a = work.numeric(|| a.is_sign_negative())?;
    let negative_b = work.numeric(|| b.is_sign_negative())?;
    let negative_product = work.numeric(|| negative_a ^ negative_b)?;
    let negative_c = work.numeric(|| c.is_sign_negative())?;
    if work.numeric(|| negative_product == negative_c)? {
        return Ok(false);
    }
    let a = dyadic(work, a)?;
    let b = dyadic(work, b)?;
    let c = dyadic(work, c)?;
    let a_significand = work.numeric(|| u128::from(a.significand))?;
    let b_significand = work.numeric(|| u128::from(b.significand))?;
    // Each normalized significand is odd and at most 53 bits: the exact odd
    // product fits below 2^106, with no variable shift/alignment or allocation.
    let product = work.numeric(|| a_significand * b_significand)?;
    let exponent = work.numeric(|| a.exponent + b.exponent)?;
    if !work.numeric(|| exponent == c.exponent)? {
        return Ok(false);
    }
    let c_significand = work.numeric(|| u128::from(c.significand))?;
    work.numeric(|| product == c_significand)
}

fn unrepresentable<T>() -> Result<T, PaintError> {
    Err(RadialGradientError::UnrepresentableGeometry.into())
}
impl From<RadialGradientError> for PaintError {
    fn from(value: RadialGradientError) -> Self {
        Self::RadialGradient(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PaintChange, PaintLimits, PaintPhase, ProfileMode};
    use std::mem::size_of;

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }

    fn proof(
        limits: PaintLimits,
        a: f64,
        b: f64,
        c: f64,
    ) -> (Result<bool, PaintError>, PaintReport) {
        let mut meter = options(limits)
            .operation(OperationKind::RadialGradient)
            .unwrap();
        let scopes = setup_scopes(&mut meter).unwrap();
        let result = meter
            .with_phase(scopes[1], |p| {
                exact_affine_zero(&mut RadialWork::new(p, scopes[1]), a, b, c)
            })
            .unwrap();
        (result, meter.finish().unwrap().unwrap())
    }

    #[test]
    fn dyadic_zero_proof_distinguishes_subnormal_residue_from_exact_cancellation() {
        let tiny = f64::from_bits(1);
        // Exact sums are respectively 0,0,0,+2^-1075,-2^-1075,+2^-1075.
        // All three nonzero residues round to signed zero in the FMA, so a
        // rounded scalar alone cannot authorize a degenerate semantic case.
        for (a, b, c, expected) in [
            (1.5, 2.0, -3.0, true),
            (tiny, 1.0, -tiny, true),
            (f64::MAX, 1.0, -f64::MAX, true),
            (tiny, 0.5, 0.0, false),
            (-tiny, 0.5, 0.0, false),
            (tiny, 1.5, -tiny, false),
        ] {
            assert_eq!(a.mul_add(b, c), 0.0);
            let (result, _) = proof(PaintLimits::unlimited(), a, b, c);
            assert_eq!(result.unwrap(), expected);
        }
        // Both 53-bit significands exercise the full bounded product. Their
        // product differs from 1 by 2^-53 - 2^-105, so cancellation is not exact.
        let a = f64::from_bits(1.0_f64.to_bits() + 1);
        let b = f64::from_bits(1.0_f64.to_bits() - 1);
        let (result, _) = proof(PaintLimits::unlimited(), a, b, -1.0);
        assert!(!result.unwrap());
    }

    #[test]
    fn proof_charges_the_literal_normal_path_and_stops_before_refused_comparison() {
        let (result, report) = proof(PaintLimits::unlimited(), 1.5, 2.0, -3.0);
        assert!(result.unwrap());
        // Guards3 + sign work5 + three normal dyadic decodes11 each +
        // product conversions2/multiply1/exponent1/compare1 + c conversion1/
        // final significand comparison1 =48 actual scalar actions.
        assert_eq!(report.work(WorkKind::GradientCoordinate), 48);
        let (result, report) = proof(
            PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 47),
            1.5,
            2.0,
            -3.0,
        );
        let Err(PaintError::Budget(refusal)) = result else {
            panic!("final proof comparison must refuse");
        };
        assert_eq!(refusal.origin, PaintPhase::GradientGeometry);
        assert_eq!(refusal.current, 47);
        assert_eq!(
            refusal.change,
            PaintChange::Work {
                kind: WorkKind::GradientCoordinate,
                amount: 1
            }
        );
        assert_eq!(report.work(WorkKind::GradientCoordinate), 47);
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }
        let (result, report) = proof(PaintLimits::unlimited(), 0.0, 2.0, -0.0);
        assert!(result.unwrap());
        assert_eq!(report.work(WorkKind::GradientCoordinate), 2);
    }

    fn zero_circle(limits: PaintLimits) -> RadialGradientOutcome {
        resolve_radial_gradient(
            Size::new(200.0, 100.0),
            Some(Point::new(0.0, 0.0)),
            RadialGradientSize::CircleRadius(0.0),
            options(limits),
        )
    }

    #[test]
    fn fixed_result_is_real_custody_and_output_refusal_releases_it() {
        let outcome = zero_circle(PaintLimits::unlimited());
        assert_eq!(
            outcome.result.unwrap().case(),
            RadialGradientCase::ZeroCircle
        );
        let report = outcome.report.unwrap();
        let fixed = size_of::<RadialGradientGeometry>() as u64;
        assert_eq!(report.live(ResourceKind::RetainedResultBytes), fixed);
        assert_eq!(report.peak(ResourceKind::RetainedResultBytes), fixed);
        assert_eq!(report.peak(ResourceKind::ScratchBytes), 0);
        // W/H, center discriminant/x/y, size discriminant, circle domain.
        assert_eq!(report.work(WorkKind::GradientValidation), 7);
        assert_eq!(report.work(WorkKind::GradientCoordinate), 1);
        assert_eq!(report.work(WorkKind::OutputRecord), 1);

        let outcome = zero_circle(
            PaintLimits::unlimited()
                .with_resource_limit(ResourceKind::RetainedResultBytes, fixed - 1),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("actual fixed owner must refuse");
        };
        assert_eq!(
            refusal.change,
            PaintChange::Acquire {
                kind: ResourceKind::RetainedResultBytes,
                amount: fixed
            }
        );
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::OutputRecord), 0);
        assert_eq!(report.peak(ResourceKind::RetainedResultBytes), 0);
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }

        let outcome =
            zero_circle(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 0));
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("output must refuse after owner admission");
        };
        assert_eq!(refusal.origin, PaintPhase::Publication);
        let report = outcome.report.unwrap();
        assert_eq!(report.peak(ResourceKind::RetainedResultBytes), fixed);
        assert_eq!(report.work(WorkKind::GradientValidation), 7);
        assert_eq!(report.work(WorkKind::GradientCoordinate), 1);
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }

        let outcome =
            zero_circle(PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 0));
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("case inspection must refuse first");
        };
        assert_eq!(refusal.origin, PaintPhase::GradientGeometry);
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::GradientValidation), 7);
        assert_eq!(report.work(WorkKind::GradientCoordinate), 0);
        assert_eq!(report.peak(ResourceKind::RetainedResultBytes), 0);
    }
}
