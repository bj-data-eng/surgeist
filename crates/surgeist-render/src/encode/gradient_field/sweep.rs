// Copyright 2022, 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// Modified native sweep correspondence is documented in ../../../NOTICE-VELLO.md.
//! The actual locked native sweep setup and bounded integer-grid consumer.
//!
//! Zero dependencies remain explicit, while reassociated zero divisions retain
//! their permitted error. The coarse stored coefficient lattice and its small
//! zero-term noise are checked separately; the native origin convention is not
//! an excuse to accept a matrix whose required axis can flush away.
use super::{Account, PhysicalSize, RenderWork, Result, ShaderScalar, invalid_geometry, scalar};

const MIN_NORMAL: f64 = f32::MIN_POSITIVE as f64;
const MAX_DIVISOR: f64 = (1_u128 << 126) as f64;
const FINE_GAMMA_NUMERATOR: f64 = 4.0 * f32::EPSILON as f64;
const FINE_FTZ_ALLOWANCE: f64 = 4.0 * MIN_NORMAL;

#[derive(Clone, Copy)]
enum Value {
    Native(f32),
    Range(ShaderScalar),
}

impl Value {
    const ZERO: Self = Self::Native(0.0);
    const ONE: Self = Self::Native(1.0);

    fn bounds(self, account: &Account) -> Result<ShaderScalar> {
        match self {
            Self::Native(value) => {
                let value = scalar(account, || f64::from(value))?;
                ShaderScalar::checked(value, value, account)
            }
            Self::Range(value) => Ok(value),
        }
    }

    fn zero(self, account: &Account) -> Result<bool> {
        match self {
            Self::Native(value) => scalar(account, || value == 0.0),
            Self::Range(_) => Ok(false),
        }
    }

    fn neg(self, account: &Account) -> Result<Self> {
        match self {
            Self::Native(value) => Ok(Self::Native(scalar(account, || -value)?)),
            Self::Range(value) => Ok(Self::Range(value.neg(account)?)),
        }
    }

    fn exact(value: f64, account: &Account) -> Result<Option<Self>> {
        let native = scalar(account, || value as f32)?;
        let back = scalar(account, || f64::from(native))?;
        if scalar(account, || value == back)?
            && scalar(account, || native.is_finite())?
            && (scalar(account, || native == 0.0)? || scalar(account, || native.is_normal())?)
        {
            Ok(Some(Self::Native(native)))
        } else {
            Ok(None)
        }
    }

    fn mul(self, other: Self, account: &Account) -> Result<Self> {
        if self.zero(account)? || other.zero(account)? {
            return Ok(Self::ZERO);
        }
        if let (Self::Native(a), Self::Native(b)) = (self, other) {
            let a = scalar(account, || f64::from(a))?;
            let b = scalar(account, || f64::from(b))?;
            // A product of two binary32 significands is exact in binary64.
            let product = scalar(account, || a * b)?;
            if let Some(exact) = Self::exact(product, account)? {
                return Ok(exact);
            }
        }
        Ok(Self::Range(
            self.bounds(account)?.mul(other.bounds(account)?, account)?,
        ))
    }

    fn add(self, other: Self, account: &Account) -> Result<Self> {
        if self.zero(account)? {
            return Ok(other);
        }
        if other.zero(account)? {
            return Ok(self);
        }
        if let (Self::Native(a), Self::Native(b)) = (self, other) {
            let a = scalar(account, || f64::from(a))?;
            let b = scalar(account, || f64::from(b))?;
            let sum = scalar(account, || a + b)?;
            // TwoSum also guards a very small addend lost by binary64 itself.
            let z = scalar(account, || sum - a)?;
            let t = scalar(account, || sum - z)?;
            let a_error = scalar(account, || a - t)?;
            let b_error = scalar(account, || b - z)?;
            let error = scalar(account, || a_error + b_error)?;
            if scalar(account, || error == 0.0)?
                && let Some(exact) = Self::exact(sum, account)?
            {
                return Ok(exact);
            }
        }
        Ok(Self::Range(
            self.bounds(account)?.add(other.bounds(account)?, account)?,
        ))
    }

    fn div(self, other: Self, account: &Account) -> Result<Self> {
        // In particular, 0/D is NOT promoted to an exact zero here.
        Ok(Self::Range(
            self.bounds(account)?.div(other.bounds(account)?, account)?,
        ))
    }

    fn hull(self, other: Self, account: &Account) -> Result<Self> {
        if let (Self::Native(a), Self::Native(b)) = (self, other)
            && scalar(account, || a == b)?
        {
            return Ok(self);
        }
        Ok(Self::Range(
            self.bounds(account)?
                .hull(other.bounds(account)?, account)?,
        ))
    }

    fn normal(self, account: &Account) -> Result<Self> {
        let absolute = self.bounds(account)?.abs(account)?;
        if scalar(account, || absolute.lower < MIN_NORMAL)? {
            return Err(invalid_geometry("sweep required inverse term can flush"));
        }
        Ok(self)
    }
}

fn sum3(a: Value, b: Value, c: Value, account: &Account) -> Result<Value> {
    a.add(b, account)?
        .add(c, account)?
        .hull(a.add(b.add(c, account)?, account)?, account)?
        .hull(a.add(c, account)?.add(b, account)?, account)
}

fn required(value: Value, nonzero: bool, account: &Account) -> Result<Value> {
    if nonzero {
        value.normal(account)
    } else {
        Ok(value)
    }
}

// Every placement of the reciprocal/division in a three-factor inverse term.
// Required products must survive before a later division could amplify FTZ.
fn term(r: Value, determinant: Value, a: Value, b: Value, account: &Account) -> Result<Value> {
    let nonzero = !a.zero(account)? && !b.zero(account)?;
    let product = |a: Value, b: Value| required(a.mul(b, account)?, nonzero, account);
    let divide = |a: Value| required(a.div(determinant, account)?, nonzero, account);
    product(product(r, a)?, b)?
        .hull(product(product(r, b)?, a)?, account)?
        .hull(product(r, product(a, b)?)?, account)?
        .hull(product(divide(a)?, b)?, account)?
        .hull(product(divide(b)?, a)?, account)?
        .hull(divide(product(a, b)?)?, account)
}

#[derive(Clone, Copy)]
enum Coefficient {
    // The real cofactor/numerator is exactly zero; division and reassociation
    // may still introduce bounded error into the stored native coefficient.
    Zero(Value),
    Normal(Value),
}

impl Coefficient {
    fn new(value: Value, zero: bool, account: &Account) -> Result<Self> {
        if zero {
            Ok(Self::Zero(value))
        } else {
            Ok(Self::Normal(value.normal(account)?))
        }
    }

    fn bounds(self, account: &Account) -> Result<ShaderScalar> {
        match self {
            Self::Zero(value) | Self::Normal(value) => value.bounds(account)?.flushed(account),
        }
    }

    fn coarse(self, account: &Account) -> Result<ShaderScalar> {
        match self {
            Self::Zero(_) => Ok(ShaderScalar::ZERO),
            Self::Normal(value) => value.bounds(account),
        }
    }
}

fn component(r: Value, det: Value, cofactor: Value, account: &Account) -> Result<Coefficient> {
    let zero = cofactor.zero(account)?;
    let literal = required(r.mul(cofactor, account)?, !zero, account)?;
    let quotient = required(cofactor.div(det, account)?, !zero, account)?;
    Coefficient::new(literal.hull(quotient, account)?, zero, account)
}

fn translation(
    r: Value,
    det: Value,
    cofactors: [Value; 2],
    coordinates: [Value; 2],
    account: &Account,
) -> Result<Coefficient> {
    let [a, c] = cofactors;
    let [x, y] = coordinates;
    let ax = a.mul(x, account)?;
    let cy = c.mul(y, account)?;
    let numerator = ax.add(cy, account)?;
    let zero = numerator.zero(account)?;
    let literal = term(r, det, a, x, account)?.add(term(r, det, c, y, account)?, account)?;
    let divided = numerator.div(det, account)?;
    let factored = r.mul(numerator, account)?;
    let output = literal.hull(divided, account)?.hull(factored, account)?;
    Coefficient::new(output.neg(account)?, zero, account)
}

fn rank(matrix: [ShaderScalar; 4], account: &Account) -> Result<()> {
    let [a, b, c, d] = matrix;
    let determinant = a
        .real_mul(d, account)?
        .real_add(b.real_mul(c, account)?.neg(account)?, account)?;
    if scalar(account, || determinant.lower <= 0.0)?
        && scalar(account, || determinant.upper >= 0.0)?
    {
        return Err(invalid_geometry("sweep inverse loses native field rank"));
    }
    Ok(())
}

fn target_axis(extent: u32, account: &Account) -> Result<Value> {
    if scalar(account, || extent == 0)? {
        return Err(invalid_geometry("empty sweep sampling domain"));
    }
    let padded = scalar(account, || extent.checked_add(15))?
        .ok_or_else(|| invalid_geometry("unrepresentable sweep tile extent"))?;
    let padded = scalar(account, || padded & !15)?;
    let maximum = scalar(account, || padded - 1)?;
    // Fine forms integer lane coordinates in f32, before its final bounds
    // check. Include every padded lane and require exact integer conversion.
    if scalar(account, || maximum > 1 << 24)? {
        return Err(invalid_geometry("unrepresentable sweep pixel coordinates"));
    }
    let maximum = scalar(account, || f64::from(maximum))?;
    Ok(Value::Range(ShaderScalar::checked(0.0, maximum, account)?))
}

fn sampling(field: [Coefficient; 6], extent: PhysicalSize, account: &Account) -> Result<()> {
    let [a, b, c, d, x, y] = field;
    rank(
        [
            a.bounds(account)?,
            b.bounds(account)?,
            c.bounds(account)?,
            d.bounds(account)?,
        ],
        account,
    )?;
    rank(
        [
            a.coarse(account)?,
            b.coarse(account)?,
            c.coarse(account)?,
            d.coarse(account)?,
        ],
        account,
    )?;
    account.work(RenderWork::CoordinateVisits, 2)?;
    let width = extent.width();
    let height = extent.height();
    let horizontal = target_axis(width, account)?;
    let vertical = target_axis(height, account)?;
    let mut quantum = f64::INFINITY;
    for coefficient in field {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if let Coefficient::Normal(value) = coefficient {
            let absolute = value.bounds(account)?.abs(account)?;
            let bits = scalar(account, || absolute.lower.to_bits())?;
            let exponent = scalar(account, || bits >> 52)?;
            let exponent = scalar(account, || exponent & 0x7ff)?;
            let exponent = scalar(account, || exponent as i32)?;
            let exponent = scalar(account, || exponent - 1023)?;
            let exponent = scalar(account, || exponent - 23)?;
            let spacing = scalar(account, || 2.0_f64.powi(exponent))?;
            quantum = scalar(account, || quantum.min(spacing))?;
        }
    }
    for row in [[a, c, x], [b, d, y]] {
        let [a, c, x] = row;
        let terms = [
            Value::Range(a.bounds(account)?).mul(horizontal, account)?,
            Value::Range(c.bounds(account)?).mul(vertical, account)?,
            Value::Range(x.bounds(account)?),
        ];
        let [first, second, last] = terms;
        let result = sum3(first, second, last, account)?;
        let magnitude = result.bounds(account)?.magnitude(account)?;
        if scalar(account, || magnitude > MAX_DIVISOR)? {
            return Err(invalid_geometry(
                "sweep angle divisor exceeds native domain",
            ));
        }
        let mut noise = ShaderScalar::ZERO;
        for (coefficient, coordinate) in row.into_iter().zip([horizontal, vertical, Value::ONE]) {
            account.work(RenderWork::CoordinateVisits, 1)?;
            if let Coefficient::Zero(value) = coefficient {
                let bound = value.bounds(account)?.abs(account)?;
                let coordinate = coordinate.bounds(account)?.abs(account)?;
                noise = noise.real_add(bound.real_mul(coordinate, account)?, account)?;
            }
        }
        // Four fine multiply/add roundings can round the zero-term noise.
        // Coarse stored f32 coefficients lie on the minimum binary32 lattice
        // quantum above; finite integer products/sums, fused or unfused, retain
        // that lattice. Outside its rounded native zero neighborhood, the
        // noise cannot turn the next coarse lattice value into a subnormal.
        let gamma = ShaderScalar::exact(FINE_GAMMA_NUMERATOR);
        let factor = ShaderScalar::ONE.real_div(
            ShaderScalar::ONE.real_add(gamma.neg(account)?, account)?,
            account,
        )?;
        let noise = noise
            .real_add(ShaderScalar::exact(FINE_FTZ_ALLOWANCE), account)?
            .real_mul(factor, account)?;
        let margin = ShaderScalar::exact(quantum).real_add(noise.neg(account)?, account)?;
        if scalar(account, || margin.lower < MIN_NORMAL)? {
            return Err(invalid_geometry(
                "sweep fine lattice cannot separate native zero",
            ));
        }
    }
    // With a zero or normal denominator <=2^126, min(abs)/max(abs) is
    // bounded by one plus its 2.5-ULP allowance. The fixed seventh-degree
    // polynomial, quarter-turn selections, normalized full-turn Pad and
    // 512-entry ramp index then have bounded finite intermediates. The sole
    // native zero neighborhood retains the pinned phi!=phi origin fallback;
    // this certificate makes no exact angle, tie or NaN portability promise.
    Ok(())
}

pub(in crate::encode) fn qualify_sweep(
    native: vello_encoding::Transform,
    sweep: peniko::SweepGradientPosition,
    extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::CoordinateVisits, 10)?;
    let values = native.matrix.into_iter().chain(native.translation);
    let mut matrix = [Value::ZERO; 6];
    for (output, value) in matrix.iter_mut().zip(values) {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if !scalar(account, || value.is_finite())?
            || (scalar(account, || value != 0.0)? && !scalar(account, || value.is_normal())?)
        {
            return Err(invalid_geometry("unrepresentable native sweep factor"));
        }
        *output = Value::Native(value);
    }
    let [a, b, c, d, tx, ty] = matrix;
    let center_x = scalar(account, || sweep.center.x as f32)?;
    let center_y = scalar(account, || sweep.center.y as f32)?;
    for value in [center_x, center_y] {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if !scalar(account, || value.is_finite())?
            || (scalar(account, || value != 0.0)? && !scalar(account, || value.is_normal())?)
        {
            return Err(invalid_geometry("unrepresentable native sweep center"));
        }
    }
    let cx = Value::Native(center_x);
    let cy = Value::Native(center_y);
    let x = sum3(a.mul(cx, account)?, c.mul(cy, account)?, tx, account)?;
    let y = sum3(b.mul(cx, account)?, d.mul(cy, account)?, ty, account)?;
    // transform_mul with the center translation retains this actual matrix.
    let determinant = a
        .mul(d, account)?
        .add(b.mul(c, account)?.neg(account)?, account)?;
    let reciprocal = Value::ONE.div(determinant, account)?.normal(account)?;
    let minus_b = b.neg(account)?;
    let minus_c = c.neg(account)?;
    let inverse = [
        component(reciprocal, determinant, d, account)?,
        component(reciprocal, determinant, minus_b, account)?,
        component(reciprocal, determinant, minus_c, account)?,
        component(reciprocal, determinant, a, account)?,
        translation(reciprocal, determinant, [d, minus_c], [x, y], account)?,
        translation(reciprocal, determinant, [minus_b, a], [x, y], account)?,
    ];
    let start = scalar(account, || sweep.start_angle / std::f32::consts::TAU)?;
    let end = scalar(account, || sweep.end_angle / std::f32::consts::TAU)?;
    if scalar(account, || start != 0.0)? || scalar(account, || end != 1.0)? {
        return Err(invalid_geometry(
            "native sweep is not a normalized full turn",
        ));
    }
    sampling(inverse, extent, account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ErrorCode, GradientRealizationError, RenderAccountingContext, RenderAccountingOptions,
        RenderAttemptKind, RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile,
        RenderResource,
    };

    fn sweep(center: (f64, f64)) -> peniko::SweepGradientPosition {
        peniko::SweepGradientPosition {
            center: center.into(),
            start_angle: 0.0,
            end_angle: std::f32::consts::TAU,
        }
    }

    fn qualify(
        native: vello_encoding::Transform,
        center: (f64, f64),
        extent: PhysicalSize,
    ) -> Result<()> {
        crate::encode::qualify_native_sweep(native, sweep(center), extent, &Account::for_test())
    }

    fn assert_capacity(error: crate::Error) {
        assert_eq!(error.code(), ErrorCode::GradientRealization);
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&GradientRealizationError::InsufficientNativeCapacity)
        );
        assert!(error.accounting_diagnostic().is_none());
        assert!(error.geometry_realization_diagnostic().is_none());
    }

    #[test]
    fn ordinary_centered_translated_and_sheared_fields_retain_required_axes() {
        for matrix in [
            [1.0, 0.0, 0.0, 1.0],
            [2.0, 0.0, 0.0, 4.0],
            [1.0, 0.25, 0.5, 1.0],
            [0.8, 0.6, -0.6, 0.8],
        ] {
            for (translation, center) in [([0.0, 0.0], (0.0, 0.0)), ([2.0, 3.0], (16.0, 8.0))] {
                qualify(
                    vello_encoding::Transform {
                        matrix,
                        translation,
                    },
                    center,
                    PhysicalSize::new(32, 16),
                )
                .unwrap();
            }
        }
        // The two native center terms cancel exactly before inverse setup.
        qualify(
            vello_encoding::Transform {
                matrix: [1.0, 0.0, 0.0, 1.0],
                translation: [-16.0, -8.0],
            },
            (16.0, 8.0),
            PhysicalSize::new(32, 16),
        )
        .unwrap();
    }

    #[test]
    fn nonzero_forward_rank_does_not_admit_underflowed_determinant_or_inverse_axis() {
        for matrix in [
            [2.0_f32.powi(-80), 0.0, 0.0, 2.0_f32.powi(-80)],
            // det=8 and reciprocal=1/8 are finite and normal, but the inverse
            // x axis is 2^-127 and can flush, collapsing a full line to origin.
            [2.0_f32.powi(127), 0.0, 0.0, 2.0_f32.powi(-124)],
        ] {
            assert_capacity(
                qualify(
                    vello_encoding::Transform {
                        matrix,
                        translation: [0.0; 2],
                    },
                    (0.0, 0.0),
                    PhysicalSize::new(32, 16),
                )
                .unwrap_err(),
            );
        }
    }

    #[test]
    fn finite_setup_does_not_admit_overflowing_center_or_angle_consumer() {
        assert_capacity(
            qualify(
                vello_encoding::Transform {
                    matrix: [2.0_f32.powi(100), 0.0, 0.0, 2.0_f32.powi(-80)],
                    translation: [0.0; 2],
                },
                (2.0_f64.powi(100), 0.0),
                PhysicalSize::new(32, 16),
            )
            .unwrap_err(),
        );
        assert_capacity(
            qualify(
                vello_encoding::Transform {
                    matrix: [2.0_f32.powi(-110), 0.0, 0.0, 2.0_f32.powi(100)],
                    translation: [0.0; 2],
                },
                (0.0, 0.0),
                PhysicalSize::new(1 << 20, 16),
            )
            .unwrap_err(),
        );
    }

    #[test]
    fn padded_integer_sampling_domain_has_an_explicit_precision_boundary() {
        let account = Account::for_test();
        for (extent, maximum) in [
            (1, 15.0),
            (16, 15.0),
            (17, 31.0),
            (1 << 24, f64::from((1 << 24) - 1)),
        ] {
            let bounds = target_axis(extent, &account)
                .unwrap()
                .bounds(&account)
                .unwrap();
            assert_eq!((bounds.lower, bounds.upper), (0.0, maximum));
        }
        assert!(target_axis((1 << 24) + 1, &account).is_err());
        assert!(target_axis(u32::MAX, &account).is_err());
        assert!(target_axis(0, &account).is_err());
    }

    #[test]
    fn reassociated_zero_division_retains_uncertainty_without_losing_normal_field_rank() {
        let account = Account::for_test();
        let output = Value::ZERO.div(Value::ONE, &account).unwrap();
        assert!(!output.zero(&account).unwrap());
        let bounds = output.bounds(&account).unwrap();
        assert!(bounds.lower < 0.0 && bounds.upper > 0.0);
        qualify(
            vello_encoding::Transform::IDENTITY,
            (0.0, 0.0),
            PhysicalSize::new(32, 16),
        )
        .unwrap();
    }

    #[test]
    fn scalar_quota_precedes_numeric_failure_and_profiles_preserve_work_and_phase() {
        let mut expected_usage = None;
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 0,
                filter: RenderPhaseFilter::Phase(RenderPhase::GradientPrepare),
            },
        ] {
            let options = RenderAccountingOptions::new().with_profile(profile);
            let mut context = RenderAccountingContext::new(options);
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    crate::encode::qualify_native_sweep(
                        vello_encoding::Transform::IDENTITY,
                        sweep((16.0, 8.0)),
                        PhysicalSize::new(32, 16),
                        account,
                    )
                })
                .unwrap();
            let report = context.last_attempt().unwrap().attempt().unwrap();
            let usage = report.usage();
            assert!(usage.work(RenderWork::CoordinateVisits) > 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
            assert_eq!(
                report
                    .phase_usage(RenderPhase::GradientPrepare)
                    .work(RenderWork::CoordinateVisits),
                usage.work(RenderWork::CoordinateVisits)
            );
            if let Some(expected) = expected_usage {
                assert_eq!(usage, expected);
            } else {
                expected_usage = Some(usage);
            }
            let exact_limit = RenderLimits::new()
                .with_work_limit(
                    RenderWork::CoordinateVisits,
                    usage.work(RenderWork::CoordinateVisits),
                )
                .unwrap();
            let mut exact = RenderAccountingContext::new(options.with_limits(exact_limit));
            exact
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    crate::encode::qualify_native_sweep(
                        vello_encoding::Transform::IDENTITY,
                        sweep((16.0, 8.0)),
                        PhysicalSize::new(32, 16),
                        account,
                    )
                })
                .unwrap();
            assert_eq!(
                exact.last_attempt().unwrap().attempt().unwrap().usage(),
                usage
            );
            for cap in [0, usage.work(RenderWork::CoordinateVisits) - 1] {
                let limits = RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, cap)
                    .unwrap();
                let mut context = RenderAccountingContext::new(options.with_limits(limits));
                let error = context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        crate::encode::qualify_native_sweep(
                            vello_encoding::Transform::IDENTITY,
                            sweep((16.0, 8.0)),
                            PhysicalSize::new(32, 16),
                            account,
                        )
                    })
                    .unwrap_err();
                assert!(error.accounting_diagnostic().is_some());
                assert!(error.gradient_realization_diagnostic().is_none());
                let refused = context.last_attempt().unwrap().attempt().unwrap().usage();
                assert!(refused.work(RenderWork::CoordinateVisits) <= cap);
                assert_eq!(refused.live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(refused.live(RenderResource::RetainedCpuBytes), 0);
            }
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, 0)
                .unwrap();
            let mut context = RenderAccountingContext::new(options.with_limits(limits));
            let error = context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    crate::encode::qualify_native_sweep(
                        vello_encoding::Transform {
                            matrix: [f32::INFINITY; 4],
                            translation: [0.0; 2],
                        },
                        sweep((0.0, 0.0)),
                        PhysicalSize::new(32, 16),
                        account,
                    )
                })
                .unwrap_err();
            assert!(error.accounting_diagnostic().is_some());
            assert!(error.gradient_realization_diagnostic().is_none());
        }
    }

    #[test]
    fn receiving_proof_quota_and_real_stop_backing_refusals_release_custody_and_retry() {
        use crate::{Capabilities, Color, Gradient, GradientStop, Point, Rect, Scene};
        let gradient = Gradient::try_sweep(
            Point::new(16.0, 8.0),
            (0..8)
                .map(|index| GradientStop::try_new(f64::from(index) / 8.0, Color::BLACK).unwrap())
                .collect(),
        )
        .unwrap();
        let mut source = Scene::new();
        source.fill(Rect::new(0.0, 0.0, 32.0, 16.0), gradient);
        let commands = source
            .normalize(Capabilities::CURRENT, &Account::for_test())
            .unwrap();
        let empty = Scene::new()
            .normalize(Capabilities::CURRENT, &Account::for_test())
            .unwrap();
        let extent = PhysicalSize::new(32, 16);
        let encode = |account: &Account| {
            drop(crate::encode::encode_vello_scene(
                &commands, 1.0, extent, account,
            )?);
            Ok(())
        };
        let mut expected = None;
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 0,
                filter: RenderPhaseFilter::Phase(RenderPhase::GradientPrepare),
            },
        ] {
            let options = RenderAccountingOptions::new().with_profile(profile);
            let mut context = RenderAccountingContext::new(options);
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, encode)
                .unwrap();
            let attempt = context.last_attempt().unwrap().attempt().unwrap();
            let usage = attempt.usage();
            let gradient_work = attempt
                .phase_usage(RenderPhase::GradientPrepare)
                .work(RenderWork::CoordinateVisits);
            assert!(usage.peak(RenderResource::ScratchCpuBytes) > 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
            if let Some(expected) = expected {
                assert_eq!(usage, expected);
            } else {
                expected = Some(usage);
            }
            let count = usage.work(RenderWork::CoordinateVisits);
            let mut preparation = RenderAccountingContext::default();
            let crate::command::RenderCommand::Fill {
                paint: crate::command::RenderPaint::Gradient(gradient),
                ..
            } = &commands.commands[0]
            else {
                panic!("sweep fill fixture");
            };
            preparation
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                    drop(crate::encode::gradient_brush(
                        gradient,
                        kurbo::Affine::IDENTITY,
                        extent,
                        account,
                    )?);
                    Ok(())
                })
                .unwrap();
            let preparation_work = preparation
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits);
            let proof_cap = count - (gradient_work - preparation_work) / 2;
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, count)
                .unwrap();
            let mut exact = RenderAccountingContext::new(options.with_limits(limits));
            exact
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, encode)
                .unwrap();
            assert_eq!(
                exact.last_attempt().unwrap().attempt().unwrap().usage(),
                usage
            );
            for limits in [
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, proof_cap)
                    .unwrap(),
                RenderLimits::new().with_scratch_bytes(0),
            ] {
                let scalar_cap = limits.work_limit(RenderWork::CoordinateVisits);
                let mut refused = RenderAccountingContext::new(options.with_limits(limits));
                let error = refused
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, encode)
                    .unwrap_err();
                assert!(error.accounting_diagnostic().is_some());
                assert!(error.gradient_realization_diagnostic().is_none());
                let attempt = refused.last_attempt().unwrap().attempt().unwrap();
                let usage = attempt.usage();
                if scalar_cap == proof_cap {
                    // Refuse within the receiving proof after the native stop
                    // allocation, independently of later endpoint bookkeeping.
                    let accepted = attempt
                        .phase_usage(RenderPhase::GradientPrepare)
                        .work(RenderWork::CoordinateVisits);
                    assert!(accepted > preparation_work && accepted < gradient_work);
                    assert!(usage.peak(RenderResource::ScratchCpuBytes) > 0);
                }
                assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
                assert_eq!(usage.work(RenderWork::Submissions), 0);
                refused
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        drop(crate::encode::encode_vello_scene(
                            &empty, 1.0, extent, account,
                        )?);
                        Ok(())
                    })
                    .unwrap();
                let retry = refused.last_attempt().unwrap().attempt().unwrap().usage();
                assert_eq!(retry.live(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(retry.live(RenderResource::RetainedCpuBytes), 0);
            }
        }
    }
}
