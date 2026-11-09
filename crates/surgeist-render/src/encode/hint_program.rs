//! Accounted analytic colour programmes and their whole-segment certificates.
use super::gradient_field::{ShaderScalar as Bound, scalar};
use crate::{
    Error, GradientRealizationError as Refusal, GradientStop, Result,
    accounting::{Account, AccountedVec, RenderWork},
    vello_engine::gradient_endpoints::{
        PadEndpoints, ProgramCoordinates, ProgramRecord, ProgramSegment,
    },
};

pub(super) struct PreparedProgram {
    pub(super) records: AccountedVec<ProgramRecord>,
    pub(super) endpoints: PadEndpoints,
    pub(super) coordinates: ProgramCoordinates,
}

const U: f64 = f32::EPSILON as f64;
const M: f64 = f32::MIN_POSITIVE as f64;
const V: f64 = 2.5 * U;
const BETA: f64 = (3.0 + 2.0 * U) * M;
const GAMMA: f64 = BETA + M;
const MAX_DIVISOR: f64 = f64::from_bits(1149_u64 << 52); //2^126.

fn capacity(error: Error) -> Error {
    if error.accounting_diagnostic().is_some() || error.gradient_realization_diagnostic().is_some()
    {
        error
    } else {
        Refusal::InsufficientNativeCapacity.into()
    }
}
fn require(account: &Account, condition: impl FnOnce() -> bool) -> Result<()> {
    if scalar(account, condition)? {
        Ok(())
    } else {
        Err(Refusal::InsufficientNativeCapacity.into())
    }
}
fn sub(left: Bound, right: Bound, account: &Account) -> Result<Bound> {
    left.real_add(right.neg(account)?, account)
}
// Intersect with a mathematical range already proved by the owning formula.
fn nonnegative(value: Bound, account: &Account) -> Result<Bound> {
    let lower = scalar(account, || value.lower.max(0.0))?;
    Bound::checked(lower, value.upper, account)
}
fn unit(value: Bound, account: &Account) -> Result<Bound> {
    let lower = scalar(account, || value.lower.max(0.0))?;
    let upper = scalar(account, || value.upper.min(1.0))?;
    Bound::checked(lower, upper, account)
}
fn quantum(account: &Account) -> Result<f64> {
    Ok(Bound::ONE.real_div(Bound::exact(511.0), account)?.lower)
}

/// The32-term atanh series with its positive geometric remainder. The
/// caller proves0<=z<1; no unproved platform transcendental is used.
fn log_series(z: Bound, account: &Account) -> Result<Bound> {
    require(account, || z.lower >= 0.0)?;
    require(account, || z.upper < 1.0)?;
    let squared = nonnegative(z.real_mul(z, account)?, account)?;
    let mut power = z;
    let mut sum = Bound::ZERO;
    for index in 0..32 {
        let doubled = scalar(account, || 2 * index)?;
        let odd = scalar(account, || doubled + 1)?;
        let denominator = scalar(account, || odd as f64)?;
        let term = power.real_div(Bound::exact(denominator), account)?;
        sum = sum.real_add(term, account)?;
        // The final iteration leaves z^65 for the explicit remainder.
        power = nonnegative(power.real_mul(squared, account)?, account)?;
    }
    let sum = sum.real_mul(Bound::exact(2.0), account)?;
    let denominator = sub(Bound::ONE, squared, account)?.real_mul(Bound::exact(65.0), account)?;
    let tail = power
        .real_mul(Bound::exact(2.0), account)?
        .real_div(denominator, account)?;
    let upper = Bound::exact(sum.upper)
        .real_add(Bound::exact(tail.upper), account)?
        .upper;
    nonnegative(Bound::checked(sum.lower, upper, account)?, account)
}

struct Logarithms {
    ln_two: Bound,
}
impl Logarithms {
    fn new(account: &Account) -> Result<Self> {
        let z = Bound::ONE.real_div(Bound::exact(3.0), account)?;
        Ok(Self {
            ln_two: log_series(z, account)?,
        })
    }
    fn endpoint(&self, mut value: f64, account: &Account) -> Result<Bound> {
        require(account, || value.is_finite())?;
        require(account, || value > 0.0)?;
        let mut bits = scalar(account, || value.to_bits())?;
        let shifted = scalar(account, || bits >> 52)?;
        let exponent_bits = scalar(account, || shifted & 0x7ff)?;
        let mut exponent = scalar(account, || exponent_bits as i32)?;
        let mut correction = 0;
        if scalar(account, || exponent == 0)? {
            const SCALE: f64 = f64::from_bits(1077_u64 << 52); //2^54.
            value = scalar(account, || value * SCALE)?;
            bits = scalar(account, || value.to_bits())?;
            let shifted = scalar(account, || bits >> 52)?;
            let exponent_bits = scalar(account, || shifted & 0x7ff)?;
            exponent = scalar(account, || exponent_bits as i32)?;
            correction = -54;
        }
        let fraction_bits = scalar(account, || bits & ((1_u64 << 52) - 1))?;
        let mantissa_bits = scalar(account, || fraction_bits | (1023_u64 << 52))?;
        let mantissa = scalar(account, || f64::from_bits(mantissa_bits))?;
        let mantissa = Bound::exact(mantissa);
        let z = nonnegative(
            sub(mantissa, Bound::ONE, account)?
                .real_div(mantissa.real_add(Bound::ONE, account)?, account)?,
            account,
        )?;
        let fraction = log_series(z, account)?;
        let unbiased = scalar(account, || exponent - 1023)?;
        let corrected = scalar(account, || unbiased + correction)?;
        let exponent = scalar(account, || corrected as f64)?;
        self.ln_two
            .real_mul(Bound::exact(exponent), account)?
            .real_add(fraction, account)
    }
    fn logarithm(&self, value: Bound, account: &Account) -> Result<Bound> {
        let lower = self.endpoint(value.lower, account)?;
        let upper = self.endpoint(value.upper, account)?;
        Bound::checked(lower.lower, upper.upper, account)
    }
    fn one_plus(&self, value: Bound, account: &Account) -> Result<Bound> {
        require(account, || value.lower > 0.0)?;
        if scalar(account, || value.upper <= 1.0)? {
            let z = value.real_div(Bound::exact(2.0).real_add(value, account)?, account)?;
            log_series(nonnegative(z, account)?, account)
        } else {
            // ln(1+r)=ln(r)+ln(1+1/r), including a narrow enclosure crossing1.
            let inverse = Bound::ONE.real_div(value, account)?;
            let z = inverse.real_div(Bound::exact(2.0).real_add(inverse, account)?, account)?;
            self.logarithm(value, account)?
                .real_add(log_series(nonnegative(z, account)?, account)?, account)
        }
    }
    fn power_tail(&self, base: Bound, exponent: Bound, account: &Account) -> Result<Bound> {
        let mut reduced = self.logarithm(base, account)?.real_mul(exponent, account)?;
        require(account, || reduced.upper <= 0.0)?;
        let mut squarings = 0;
        while scalar(account, || reduced.lower < -0.5)? {
            reduced = reduced.real_mul(Bound::exact(0.5), account)?;
            squarings += 1;
            if squarings > 2048 {
                return Err(Refusal::InsufficientNativeCapacity.into());
            }
        }
        let mut term = Bound::ONE;
        let mut sum = Bound::ONE;
        for index in 1..32 {
            let denominator = scalar(account, || index as f64)?;
            term = term
                .real_mul(reduced, account)?
                .real_div(Bound::exact(denominator), account)?;
            sum = sum.real_add(term, account)?;
        }
        let remainder = term
            .abs(account)?
            .real_mul(reduced.abs(account)?, account)?
            .real_div(Bound::exact(32.0), account)?
            .real_mul(Bound::exact(2.0), account)?;
        let lower = sub(
            Bound::exact(sum.lower),
            Bound::exact(remainder.upper),
            account,
        )?
        .lower;
        let upper = Bound::exact(sum.upper)
            .real_add(Bound::exact(remainder.upper), account)?
            .upper;
        let mut result = unit(Bound::checked(lower, upper, account)?, account)?;
        for _ in 0..squarings {
            result = unit(result.real_mul(result, account)?, account)?;
        }
        Ok(result)
    }
}

struct ParameterBounds {
    r: Bound,
    a: Bound,
}
fn parameter_bounds(left: f32, right: f32, account: &Account) -> Result<ParameterBounds> {
    let left = scalar(account, || f64::from(left))?;
    let right = scalar(account, || f64::from(right))?;
    let d = sub(Bound::exact(right), Bound::exact(left), account)?;
    let lower = sub(
        d.real_mul(Bound::exact(1.0 - U), account)?,
        Bound::exact(BETA),
        account,
    )?;
    let upper = d
        .real_mul(Bound::exact(1.0 + U), account)?
        .real_add(Bound::exact(BETA), account)?;
    require(account, || lower.lower >= M)?;
    require(account, || upper.upper <= MAX_DIVISOR)?;
    let delta = Bound::exact(U).real_add(Bound::exact(BETA).real_div(d, account)?, account)?;
    require(account, || delta.upper < 1.0)?;
    let denominator = sub(Bound::ONE, delta, account)?;
    let r = Bound::exact(U)
        .real_add(delta, account)?
        .real_add(
            Bound::exact(V).real_mul(Bound::exact(1.0 + U), account)?,
            account,
        )?
        .real_div(denominator, account)?;
    let a = Bound::exact(1.0 + V)
        .real_mul(Bound::exact(GAMMA), account)?
        .real_div(d.real_mul(denominator, account)?, account)?
        .real_add(Bound::exact(2.0 * M), account)?;
    Ok(ParameterBounds { r, a })
}

fn linear(left: f32, right: f32, account: &Account) -> Result<()> {
    let p = parameter_bounds(left, right, account)?;
    let error = p.r.real_add(p.a, account)?;
    let quantum = quantum(account)?;
    require(account, || error.upper <= quantum)
}

fn power(
    left: f32,
    hint: f32,
    right: f32,
    index: usize,
    logs: &Logarithms,
    account: &Account,
) -> Result<f32> {
    let left_bound = Bound::exact(scalar(account, || f64::from(left))?);
    let hint_bound = Bound::exact(scalar(account, || f64::from(hint))?);
    let right_bound = Bound::exact(scalar(account, || f64::from(right))?);
    let left_distance = sub(hint_bound, left_bound, account)?;
    let right_distance = sub(right_bound, hint_bound, account)?;
    let ratio = right_distance.real_div(left_distance, account)?;
    // Accepted canonical binary32 controls lie within0..1; their smallest
    // distinct spacing is2^-149. Thus this f64 ratio cannot overflow.
    let k = logs
        .ln_two
        .real_div(logs.one_plus(ratio, account)?, account)?;
    let lower_half = scalar(account, || k.lower * 0.5)?;
    let upper_half = scalar(account, || k.upper * 0.5)?;
    let middle = scalar(account, || lower_half + upper_half)?;
    let native = scalar(account, || middle as f32)?;
    if !scalar(account, || native.is_normal())? || !scalar(account, || native > 0.0)? {
        return Err(Refusal::UnrepresentableHint { stop_index: index }.into());
    }
    let native64 = scalar(account, || f64::from(native))?;
    let difference = sub(Bound::exact(native64), k, account)?.abs(account)?;
    let delta = Bound::exact(difference.upper);
    let mu = Bound::exact(scalar(account, || k.lower.min(native64))?);
    require(account, || mu.lower > 0.0)?;
    let k_native = Bound::exact(native64);
    let p = parameter_bounds(left, right, account)?;
    let p0_scaled = p.a.real_mul(Bound::exact(16_777_216.0), account)?;
    let p0 = Bound::exact(scalar(account, || p0_scaled.upper.max(64.0 * M))?);
    let rho = p.r.real_add(p.a.real_div(p0, account)?, account)?;
    let q0 = Bound::ONE
        .real_add(p.r, account)?
        .real_mul(p0, account)?
        .real_add(p.a, account)?;
    require(account, || p0.upper <= 0.5)?;
    require(account, || q0.upper < 1.0)?;
    require(account, || p.r.upper < 1.0)?;
    require(account, || rho.upper <= 1.0 / 16.0)?;
    let tail_domain = Bound::exact(M)
        .real_add(p.a, account)?
        .real_div(sub(Bound::ONE, p.r, account)?, account)?;
    require(account, || p0.lower >= tail_domain.upper)?;
    let tail = logs.power_tail(q0, mu, account)?;
    let z = k_native
        .real_mul(rho, account)?
        .real_div(sub(Bound::ONE, rho, account)?, account)?;
    require(account, || z.upper < 1.0 / 16.0)?;
    let input = z.real_div(sub(Bound::ONE, z, account)?, account)?;
    let coefficient = delta.real_div(mu.real_mul(Bound::exact(2.0), account)?, account)?;
    const W: f64 = 4.0 * U + 3.0 * U * U;
    let d0 = k_native
        .real_mul(Bound::exact(1.0 + U), account)?
        .real_mul(Bound::exact(M), account)?
        .real_add(Bound::exact(2.0 * M), account)?;
    let d0_domain = d0.real_add(Bound::exact(125.0 * W), account)?;
    require(account, || d0_domain.upper <= 1.0)?;
    let high = k_native
        .real_mul(
            Bound::exact((1.0 + U) * f64::from_bits(1002_u64 << 52) + U),
            account,
        )?
        .real_add(d0, account)?; //g=2^-21.
    require(account, || high.upper <= 1.0)?;
    let low = Bound::exact(W)
        .real_div(Bound::exact(2.0 * (1.0 - W)), account)?
        .real_add(d0, account)?;
    let log_product = Bound::exact(scalar(account, || high.upper.max(low.upper))?);
    let exp = Bound::exact(255.0 * U).real_add(Bound::exact(M), account)?;
    let middle = input.real_add(coefficient, account)?;
    let error = Bound::exact(scalar(account, || tail.upper.max(middle.upper))?)
        .real_add(log_product, account)?
        .real_add(exp, account)?
        .real_add(Bound::exact(2.0 * M), account)?;
    let quantum = quantum(account)?;
    require(account, || error.upper <= quantum)?;
    Ok(native)
}

pub(super) fn prepare(
    stops: &[GradientStop],
    native: &[peniko::ColorStop],
    normalize: impl Fn(f64, &Account) -> Result<f32>,
    endpoints: PadEndpoints,
    coordinates: ProgramCoordinates,
    account: &Account,
) -> Result<PreparedProgram> {
    prepare_projected(
        stops,
        |index, _, account| {
            account.work(RenderWork::GradientStopVisits, 1)?;
            scalar(account, || native[index].offset)
        },
        normalize,
        endpoints,
        coordinates,
        account,
    )
}

/// Semantic controls may differ from the uniform dependency carrier. This
/// provider owns each real control projection; no temporary colour list or
/// duplicated semantic control authority is needed.
pub(super) fn prepare_projected(
    stops: &[GradientStop],
    native: impl Fn(usize, f64, &Account) -> Result<f32>,
    normalize: impl Fn(f64, &Account) -> Result<f32>,
    endpoints: PadEndpoints,
    coordinates: ProgramCoordinates,
    account: &Account,
) -> Result<PreparedProgram> {
    let result = (|| {
        let mut records = account.temporary(stops.len())?;
        let mut logs = None;
        for (index, stop) in stops.iter().enumerate() {
            account.work(RenderWork::GradientStopVisits, 1)?;
            let position = scalar(account, || stop.position())?;
            let native_position = native(index, position, account)?;
            if !scalar(account, || native_position.is_finite())? {
                return Err(Refusal::UnrepresentableStop { stop_index: index }.into());
            }
            let outgoing = if let Some(next) = stops.get(index + 1) {
                account.work(RenderWork::GradientStopVisits, 1)?;
                let next_position = scalar(account, || next.position())?;
                let next_native = native(index + 1, next_position, account)?;
                let same = scalar(account, || position == next_position)?;
                let ordered = if same {
                    scalar(account, || native_position == next_native)?
                } else {
                    scalar(account, || native_position < next_native)?
                };
                if !ordered {
                    return Err(Refusal::UnrepresentableStop {
                        stop_index: index + 1,
                    }
                    .into());
                }
                if same {
                    ProgramSegment::Linear
                } else if let Some(hint) = stop.hint_after() {
                    let hint = scalar(account, || hint.position())?;
                    if scalar(account, || hint == position)? {
                        ProgramSegment::HintAtStart
                    } else if scalar(account, || hint == next_position)? {
                        ProgramSegment::HintAtEnd
                    } else {
                        let hint = normalize(hint, account)?;
                        if !scalar(account, || hint > native_position)?
                            || !scalar(account, || hint < next_native)?
                        {
                            return Err(Refusal::UnrepresentableHint { stop_index: index }.into());
                        }
                        if logs.is_none() {
                            logs = Some(Logarithms::new(account)?);
                        }
                        let exponent = power(
                            native_position,
                            hint,
                            next_native,
                            index,
                            logs.as_ref().expect("initialized logarithm bounds"),
                            account,
                        )?;
                        ProgramSegment::Power { exponent }
                    }
                } else {
                    linear(native_position, next_native, account)?;
                    ProgramSegment::Linear
                }
            } else {
                ProgramSegment::Linear
            };
            account.work(RenderWork::CoordinateVisits, 4)?;
            let color = stop.color();
            let (r, g, b, a) = (color.r(), color.g(), color.b(), color.a());
            let premultiplied = [
                scalar(account, || r * a)?,
                scalar(account, || g * a)?,
                scalar(account, || b * a)?,
                a,
            ];
            records.append_slot()?.write(ProgramRecord {
                position: native_position,
                premultiplied,
                outgoing,
            });
        }
        Ok(PreparedProgram {
            records,
            endpoints,
            coordinates,
        })
    })();
    result.map_err(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
    use crate::{
        Color, GradientHint, RenderAccountingContext, RenderAccountingOptions, RenderLimits,
    };

    #[test]
    fn practical_native_hints_have_certified_positive_power_coefficients() {
        let account = Account::for_test();
        let logs = Logarithms::new(&account).unwrap();
        // Independent analytic values ln(.5)/ln(H), rounded only in this
        // expected-value table. Accepted binary32 H slightly changes the curve.
        for (hint, expected) in [
            (0.001_f32, 0.10034333),
            (0.01, 0.150515),
            (0.1, std::f64::consts::LOG10_2),
            (0.25, 0.5),
            (0.5, 1.0),
            (0.75, 2.4094208),
            (0.9, 6.578813),
            (0.99, 68.96756),
        ] {
            let actual = power(0., hint, 1., 0, &logs, &account).unwrap();
            assert!(actual.is_normal() && actual > 0.0);
            assert!(
                (f64::from(actual) - expected).abs() <= expected * 1.0e-5,
                "hint{hint}: {actual} vs{expected}"
            );
        }
    }
    #[test]
    fn bounded_logarithms_enclose_independent_constants() {
        let account = Account::for_test();
        let logs = Logarithms::new(&account).unwrap();
        for (value, expected) in [
            (1., 0.),
            (2., std::f64::consts::LN_2),
            (0.5, -std::f64::consts::LN_2),
            (4., std::f64::consts::LN_2 * 2.),
        ] {
            let bounds = logs.endpoint(value, &account).unwrap();
            assert!(
                bounds.lower <= expected && bounds.upper >= expected,
                "{value}: {}..{} expected{expected}",
                bounds.lower,
                bounds.upper
            );
        }
        let tail = logs
            .power_tail(Bound::exact(0.25), Bound::exact(0.5), &account)
            .unwrap();
        assert!(tail.lower <= 0.5 && tail.upper >= 0.5);
    }
    #[test]
    fn coefficient_decomposition_refuses_before_its_first_bit_shift_and_can_retry() {
        let fixture = Account::for_test();
        let logs = Logarithms::new(&fixture).unwrap();
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 3)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| logs.endpoint(2.0, account).map(drop),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        // Finiteness, positive-domain check, and binary64 representation
        // capture are accepted; exponent extraction has not run.
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 3);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert!(logs.endpoint(2.0, &fixture).is_ok());
    }
    fn native(stops: &[GradientStop]) -> Vec<peniko::ColorStop> {
        stops
            .iter()
            .map(|stop| {
                peniko::ColorStop::from((stop.position() as f32, peniko::Color::from(stop.color())))
            })
            .collect()
    }
    #[test]
    fn limit_tags_and_zero_width_hints_do_not_require_power_arithmetic() {
        let account = Account::for_test();
        for (hint, kind) in [
            (0.0, ProgramSegment::HintAtStart),
            (1.0, ProgramSegment::HintAtEnd),
        ] {
            let stops = [
                GradientStop::try_new(0., Color::BLACK)
                    .unwrap()
                    .with_hint_after(GradientHint::try_new(hint).unwrap()),
                GradientStop::try_new(1., Color::TRANSPARENT).unwrap(),
            ];
            let native = native(&stops);
            let endpoints =
                PadEndpoints::new((0., Color::BLACK), (1., Color::TRANSPARENT), &account).unwrap();
            let program = prepare(
                &stops,
                &native,
                |_, _| panic!("endpoint hint must not be narrowed"),
                endpoints,
                ProgramCoordinates::Pad,
                &account,
            )
            .unwrap();
            assert!(matches!(
                (program.records[0].outgoing, kind),
                (ProgramSegment::HintAtStart, ProgramSegment::HintAtStart)
                    | (ProgramSegment::HintAtEnd, ProgramSegment::HintAtEnd)
            ));
        }
        let stops = [
            GradientStop::try_new(0., Color::BLACK)
                .unwrap()
                .with_hint_after(GradientHint::try_new(0.).unwrap()),
            GradientStop::try_new(0., Color::TRANSPARENT).unwrap(),
        ];
        let program = prepare(
            &stops,
            &native(&stops),
            |_, _| panic!("zero-width hint must not be narrowed"),
            PadEndpoints::new((0., Color::BLACK), (0., Color::TRANSPARENT), &account).unwrap(),
            ProgramCoordinates::Pad,
            &account,
        )
        .unwrap();
        assert!(matches!(
            program.records[0].outgoing,
            ProgramSegment::Linear
        ));
    }
    #[test]
    fn strict_hint_control_collapse_releases_programme_backing_and_preserves_ordinal() {
        let stops = [
            GradientStop::try_new(0., Color::BLACK)
                .unwrap()
                .with_hint_after(GradientHint::try_new(0.5).unwrap()),
            GradientStop::try_new(1., Color::TRANSPARENT).unwrap(),
        ];
        let native = native(&stops);
        let mut context = RenderAccountingContext::default();
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    let endpoints =
                        PadEndpoints::new((0., Color::BLACK), (1., Color::TRANSPARENT), account)?;
                    prepare(
                        &stops,
                        &native,
                        |_, _| Ok(0.0),
                        endpoints,
                        ProgramCoordinates::Pad,
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::UnrepresentableHint { stop_index: 0 })
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        let account = Account::for_test();
        assert!(
            prepare(
                &stops,
                &native,
                |p, account| scalar(account, || p as f32),
                PadEndpoints::new((0., Color::BLACK), (1., Color::TRANSPARENT), &account).unwrap(),
                ProgramCoordinates::Pad,
                &account
            )
            .is_ok()
        );
    }
    #[test]
    fn linear_intervals_refuse_division_outside_its_native_accuracy_domain() {
        let error = linear(0., f32::from_bits(1), &Account::for_test()).unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::InsufficientNativeCapacity)
        );
        assert!(linear(0., 1., &Account::for_test()).is_ok());
    }
}
