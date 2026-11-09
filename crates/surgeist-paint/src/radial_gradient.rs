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
        let request = meter
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
                geometry(&mut RadialWork::new(p, scopes[1]), request)
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

// Validation carries its facts into geometry, including each affine radius's
// matching full box dimension. No raw size variant survives this boundary.
struct ValidatedRequest {
    box_size: Size<qualified::Finite>,
    center: Option<Point<qualified::Finite>>,
    ending_size: ValidatedSize,
}

enum ValidatedSize {
    CircleRadius(qualified::Finite),
    EllipseRadii(Size<AffineRadius>),
    CircleExtent(RadialGradientExtent),
    EllipseExtent(RadialGradientExtent),
}

#[derive(Clone, Copy)]
struct AffineRadius {
    fraction: qualified::Finite,
    dimension: qualified::Finite,
    length: qualified::Finite,
}

fn validate(
    work: &mut RadialWork<'_, '_, '_>,
    box_size: Size<f64>,
    center: Option<Point<f64>>,
    ending_size: RadialGradientSize,
) -> Result<ValidatedRequest, PaintError> {
    use qualified::Finite;
    let width = Finite::nonnegative(work, box_size.width, RadialGradientError::InvalidWidth)?;
    let height = Finite::nonnegative(work, box_size.height, RadialGradientError::InvalidHeight)?;
    work.validation(|| ())?;
    let center = if let Some(center) = center {
        let x = Finite::new(
            work,
            center.x,
            RadialGradientError::NonFiniteCenter {
                axis: PhysicalAxis::Horizontal,
            },
        )?;
        let y = Finite::new(
            work,
            center.y,
            RadialGradientError::NonFiniteCenter {
                axis: PhysicalAxis::Vertical,
            },
        )?;
        Some(Point::new(x, y))
    } else {
        None
    };
    work.validation(|| ())?;
    let ending_size = match ending_size {
        RadialGradientSize::CircleRadius(radius) => ValidatedSize::CircleRadius(
            Finite::nonnegative(work, radius, RadialGradientError::InvalidCircleRadius)?,
        ),
        RadialGradientSize::EllipseRadii(radii) => {
            let x = validate_position(work, PhysicalAxis::Horizontal, radii.width, width)?;
            let y = validate_position(work, PhysicalAxis::Vertical, radii.height, height)?;
            ValidatedSize::EllipseRadii(Size::new(x, y))
        }
        RadialGradientSize::CircleExtent(extent) => ValidatedSize::CircleExtent(extent),
        RadialGradientSize::EllipseExtent(extent) => ValidatedSize::EllipseExtent(extent),
    };
    Ok(ValidatedRequest {
        box_size: Size::new(width, height),
        center,
        ending_size,
    })
}

fn validate_position(
    work: &mut RadialWork<'_, '_, '_>,
    axis: PhysicalAxis,
    position: GradientPosition,
    dimension: qualified::Finite,
) -> Result<AffineRadius, PaintError> {
    let length = qualified::Finite::new(
        work,
        position.length,
        RadialGradientError::NonFiniteRadiusComponent {
            axis,
            component: GradientPositionComponent::Length,
        },
    )?;
    let fraction = qualified::Finite::new(
        work,
        position.fraction,
        RadialGradientError::NonFiniteRadiusComponent {
            axis,
            component: GradientPositionComponent::Fraction,
        },
    )?;
    Ok(AffineRadius {
        fraction,
        dimension,
        length,
    })
}

fn geometry(
    work: &mut RadialWork<'_, '_, '_>,
    request: ValidatedRequest,
) -> Result<RadialGradientGeometry, PaintError> {
    let box_size = Size::new(request.box_size.width.get(), request.box_size.height.get());
    let center = match request.center {
        Some(center) => Point::new(center.x.get(), center.y.get()),
        None => Point::new(
            center_half(work, box_size.width)?,
            center_half(work, box_size.height)?,
        ),
    };
    let case = match request.ending_size {
        ValidatedSize::CircleRadius(radius) => {
            let radius = radius.get();
            if work.numeric(|| radius == 0.0)? {
                ResolvedCase::ZeroCircle
            } else {
                positive_ellipse(work, radius, radius)?
            }
        }
        ValidatedSize::EllipseRadii(radii) => {
            let x = affine_radius(work, radii.width, PhysicalAxis::Horizontal)?;
            let y = affine_radius(work, radii.height, PhysicalAxis::Vertical)?;
            if work.numeric(|| x == 0.0)? {
                ResolvedCase::ZeroWidth
            } else if work.numeric(|| y == 0.0)? {
                ResolvedCase::ZeroHeight
            } else {
                positive_ellipse(work, x, y)?
            }
        }
        ValidatedSize::CircleExtent(extent) => circle_extent(work, box_size, center, extent)?,
        ValidatedSize::EllipseExtent(extent) => ellipse_extent(work, box_size, center, extent)?,
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
    terms: AffineRadius,
    axis: PhysicalAxis,
) -> Result<f64, PaintError> {
    let radius = work.numeric(|| {
        terms
            .fraction
            .get()
            .mul_add(terms.dimension.get(), terms.length.get())
    })?;
    if !work.numeric(|| radius.is_finite())? {
        return unrepresentable();
    }
    if work.numeric(|| radius < 0.0)? {
        return Err(RadialGradientError::NegativeRadius { axis }.into());
    }
    if work.numeric(|| radius == 0.0)? && !exact_affine_zero(work, terms)? {
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

// This owner alone issues finite values and classifies them as zero/nonzero.
// Its private fields prevent raw values from bypassing either boundary.
mod qualified {
    use super::{Dyadic, PaintError, RadialGradientError, RadialWork};

    #[derive(Clone, Copy)]
    pub(super) struct Finite(f64);

    pub(super) enum ZeroClass {
        Zero,
        Nonzero(Nonzero),
    }

    #[derive(Clone, Copy)]
    pub(super) struct Nonzero(Finite);

    impl Finite {
        pub(super) fn new(
            work: &mut RadialWork<'_, '_, '_>,
            value: f64,
            error: RadialGradientError,
        ) -> Result<Self, PaintError> {
            work.validation(|| {
                if value.is_finite() {
                    Some(Self(value))
                } else {
                    None
                }
            })?
            .ok_or_else(|| error.into())
        }

        pub(super) fn nonnegative(
            work: &mut RadialWork<'_, '_, '_>,
            value: f64,
            error: RadialGradientError,
        ) -> Result<Self, PaintError> {
            work.validation(|| {
                if value.is_finite() && value >= 0.0 {
                    Some(Self(value))
                } else {
                    None
                }
            })?
            .ok_or_else(|| error.into())
        }

        pub(super) fn get(self) -> f64 {
            self.0
        }

        pub(super) fn classify_zero(
            self,
            work: &mut RadialWork<'_, '_, '_>,
        ) -> Result<ZeroClass, PaintError> {
            if work.numeric(|| self.0 == 0.0)? {
                Ok(ZeroClass::Zero)
            } else {
                Ok(ZeroClass::Nonzero(Nonzero(self)))
            }
        }
    }

    impl Nonzero {
        pub(super) fn get(self) -> f64 {
            self.0.get()
        }

        /// Decodes this finite nonzero value as odd-significand * 2^exponent.
        pub(super) fn dyadic(
            self,
            work: &mut RadialWork<'_, '_, '_>,
        ) -> Result<Dyadic, PaintError> {
            let bits = work.numeric(|| self.get().to_bits())?;
            let exponent_field = work.numeric(|| (bits >> 52) & 0x7ff)?;
            let fraction = work.numeric(|| bits & 0x000f_ffff_ffff_ffff)?;
            let (significand, exponent) = if work.numeric(|| exponent_field == 0)? {
                // Finite nonzero subnormals have significand 1..2^52-1.
                (fraction, -1074)
            } else {
                // Normals have significand 2^52..2^53-1. The 11-bit
                // exponent field is at most 2046 for this finite value.
                let significand = work.numeric(|| fraction | (1_u64 << 52))?;
                let exponent = work.numeric(|| exponent_field as i32)?;
                (significand, work.numeric(|| exponent - 1075)?)
            };
            // The nonzero significand has at most 52 trailing zeros (51 for
            // subnormals), so the shift is <64 and the conversion fits i32.
            let zeros = work.numeric(|| significand.trailing_zeros())?;
            let significand = work.numeric(|| significand >> zeros)?;
            let zeros = work.numeric(|| zeros as i32)?;
            // Normalized exponents lie in -1074..1023; the subnormal upper
            // bound is -1023. All normalization arithmetic fits i32.
            let exponent = work.numeric(|| exponent + zeros)?;
            Ok(Dyadic {
                significand,
                exponent,
            })
        }
    }
}

fn exact_affine_zero(
    work: &mut RadialWork<'_, '_, '_>,
    terms: AffineRadius,
) -> Result<bool, PaintError> {
    use qualified::ZeroClass::{Nonzero, Zero};
    // No product, decode, or zero shift is needed for a zero factor.
    let a = match terms.fraction.classify_zero(work)? {
        Zero => return Ok(matches!(terms.length.classify_zero(work)?, Zero)),
        Nonzero(value) => value,
    };
    let b = match terms.dimension.classify_zero(work)? {
        Zero => return Ok(matches!(terms.length.classify_zero(work)?, Zero)),
        Nonzero(value) => value,
    };
    let c = match terms.length.classify_zero(work)? {
        Zero => return Ok(false),
        Nonzero(value) => value,
    };
    let negative_a = work.numeric(|| a.get().is_sign_negative())?;
    let negative_b = work.numeric(|| b.get().is_sign_negative())?;
    let negative_product = work.numeric(|| negative_a ^ negative_b)?;
    let negative_c = work.numeric(|| c.get().is_sign_negative())?;
    if work.numeric(|| negative_product == negative_c)? {
        return Ok(false);
    }
    let a = a.dyadic(work)?;
    let b = b.dyadic(work)?;
    let c = c.dyadic(work)?;
    let a_significand = work.numeric(|| u128::from(a.significand))?;
    let b_significand = work.numeric(|| u128::from(b.significand))?;
    // Each normalized significand is odd and at most 53 bits: the exact odd
    // product fits below 2^106, with no variable shift/alignment or allocation.
    let product = work.numeric(|| a_significand * b_significand)?;
    // The sum lies in -2148..2046, well within i32.
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
        let terms = validated_terms(&mut meter, scopes[0], a, b, c);
        let result = meter
            .with_phase(scopes[1], |p| {
                exact_affine_zero(&mut RadialWork::new(p, scopes[1]), terms)
            })
            .unwrap();
        (result, meter.finish().unwrap().unwrap())
    }

    fn validated_terms(meter: &mut Meter, scope: Scope, a: f64, b: f64, c: f64) -> AffineRadius {
        let request = meter
            .with_phase(scope, |p| {
                validate(
                    &mut RadialWork::new(p, scope),
                    Size::new(b, 1.0),
                    Some(Point::new(0.0, 0.0)),
                    RadialGradientSize::EllipseRadii(Size::new(
                        GradientPosition {
                            length: c,
                            fraction: a,
                        },
                        GradientPosition {
                            length: 1.0,
                            fraction: 0.0,
                        },
                    )),
                )
            })
            .unwrap()
            .unwrap();
        match request.ending_size {
            ValidatedSize::EllipseRadii(radii) => radii.width,
            _ => panic!("validated ellipse must carry its affine terms"),
        }
    }

    fn decode(value: f64) -> (Dyadic, PaintReport) {
        let mut meter = options(PaintLimits::unlimited())
            .operation(OperationKind::RadialGradient)
            .unwrap();
        let scopes = setup_scopes(&mut meter).unwrap();
        let value = validated_terms(&mut meter, scopes[0], 0.0, 1.0, value).length;
        let result = meter
            .with_phase(scopes[1], |p| {
                let mut work = RadialWork::new(p, scopes[1]);
                match value.classify_zero(&mut work)? {
                    qualified::ZeroClass::Nonzero(value) => value.dyadic(&mut work),
                    qualified::ZeroClass::Zero => panic!("boundary fixture must be nonzero"),
                }
            })
            .unwrap()
            .unwrap();
        (result, meter.finish().unwrap().unwrap())
    }

    #[test]
    fn finite_nonzero_boundaries_decode_to_exact_odd_dyadics() {
        // IEEE binary64 encodings give these independent exact values. The
        // powers of two exercise all 52 possible normal trailing zero bits.
        for (value, significand, exponent, work) in [
            (f64::from_bits(1), 1, -1074, 8),
            (f64::from_bits(1_u64 << 51), 1, -1023, 8),
            (
                f64::from_bits(0x000f_ffff_ffff_ffff),
                (1_u64 << 52) - 1,
                -1074,
                8,
            ),
            (f64::MIN_POSITIVE, 1, -1022, 11),
            (1.0, 1, 0, 11),
            (f64::from_bits(0x7fe0_0000_0000_0000), 1, 1023, 11),
            (f64::MAX, (1_u64 << 53) - 1, 971, 11),
        ] {
            for signed in [value, -value] {
                let (actual, report) = decode(signed);
                assert_eq!(actual.significand, significand);
                assert_eq!(actual.exponent, exponent);
                // One preceding charged zero classification issues Nonzero.
                assert_eq!(report.work(WorkKind::GradientCoordinate), work + 1);
            }
        }
    }

    #[test]
    fn signed_zero_guards_short_circuit_before_decode_and_preserve_opposing_signs() {
        for zero in [0.0, -0.0] {
            for (a, b, c, expected, work) in [
                (zero, f64::MAX, -0.0, true, 2),
                (f64::MAX, zero, 0.0, true, 3),
                (zero, f64::MAX, f64::MIN_POSITIVE, false, 2),
                (f64::MAX, 1.0, zero, false, 3),
            ] {
                let (result, report) = proof(PaintLimits::unlimited(), a, b, c);
                assert_eq!(result.unwrap(), expected);
                assert_eq!(report.work(WorkKind::GradientCoordinate), work);
            }
        }
        // A negative fraction and positive dimension oppose a positive length.
        // The same signs cannot cancel, and stop before any dyadic decode.
        for (c, expected, work) in [(3.0, true, 48), (-3.0, false, 8)] {
            let (result, report) = proof(PaintLimits::unlimited(), -1.5, 2.0, c);
            assert_eq!(result.unwrap(), expected);
            assert_eq!(report.work(WorkKind::GradientCoordinate), work);
        }
    }

    #[test]
    fn subnormal_cancellation_charges_literal_work_and_refuses_before_proof_actions() {
        let tiny = f64::from_bits(1);
        let (result, report) = proof(PaintLimits::unlimited(), tiny, 1.0, -tiny);
        assert!(result.unwrap());
        // Guards3 + signs5 + subnormal8/normal11/subnormal8 + product and
        // final comparisons7 =42. After the guards/signs, the first decode
        // starts at9, its normalization shift at14, and multiplication at38.
        assert_eq!(report.work(WorkKind::GradientCoordinate), 42);
        for limit in [8, 13, 37] {
            let (result, report) = proof(
                PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, limit),
                tiny,
                1.0,
                -tiny,
            );
            let Err(PaintError::Budget(refusal)) = result else {
                panic!("the next exact-proof action must refuse");
            };
            assert_eq!(refusal.origin, PaintPhase::GradientGeometry);
            assert_eq!(refusal.current, limit);
            assert_eq!(
                refusal.change,
                PaintChange::Work {
                    kind: WorkKind::GradientCoordinate,
                    amount: 1,
                }
            );
            assert_eq!(report.work(WorkKind::GradientCoordinate), limit);
            for &kind in ResourceKind::ALL {
                assert_eq!(report.live(kind), 0);
            }
        }
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
