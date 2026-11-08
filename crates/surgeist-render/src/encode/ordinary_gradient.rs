//! Contextual realization of ordinary finite scalar fields.
use super::{BrushTransform, FrameBrush, gradient_field as field, gradient_with_stops};
use crate::{
    Error, Gradient, GradientRealizationError as Refusal, PhysicalSize, Result,
    accounting::{Account, RenderWork},
    paint::GradientKind,
    vello_engine::gradient_endpoints::PadEndpoints,
};

pub(super) fn capacity_error(error: Error) -> Error {
    if error.accounting_diagnostic().is_some() || error.gradient_realization_diagnostic().is_some()
    {
        error
    } else {
        Refusal::InsufficientNativeCapacity.into()
    }
}
fn geometry_error(error: Error) -> Error {
    if error.accounting_diagnostic().is_some() {
        error
    } else {
        Refusal::UnrepresentableGeometry.into()
    }
}

/// Admit each scalar comparison, arithmetic result, or representation conversion
/// before evaluating it. Integer exponent bookkeeping does not inspect coordinates.
fn scalar<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

/// A bounded binary factor. Products and affine sums stay scaled until the
/// coefficient actually required by the native interface is materialized.
#[derive(Clone, Copy)]
struct Scaled {
    mantissa: f64,
    exponent: i32,
}
impl Scaled {
    fn new(value: f64, account: &Account) -> Result<Self> {
        if scalar(account, || value == 0.0)? {
            return Ok(Self {
                mantissa: value,
                exponent: 0,
            });
        }
        let bits = scalar(account, || value.to_bits())?;
        let exponent = ((bits >> 52) & 0x7ff) as i32;
        if exponent == 0 {
            let normalized = scalar(account, || value * 2.0_f64.powi(54))?;
            let mut result = Self::new(normalized, account)?;
            result.exponent -= 54;
            Ok(result)
        } else {
            let mantissa = scalar(account, || {
                f64::from_bits((bits & ((1_u64 << 63) | ((1_u64 << 52) - 1))) | (1023_u64 << 52))
            })?;
            Ok(Self {
                mantissa,
                exponent: exponent - 1023,
            })
        }
    }
    fn mul(self, other: Self, account: &Account) -> Result<Self> {
        let product = scalar(account, || self.mantissa * other.mantissa)?;
        let mut result = Self::new(product, account)?;
        result.exponent += self.exponent + other.exponent;
        Ok(result)
    }
    fn add(self, other: Self, account: &Account) -> Result<Self> {
        let (high, low) = if self.exponent >= other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        if scalar(account, || high.mantissa == 0.0)? {
            return Ok(low);
        }
        if scalar(account, || low.mantissa == 0.0)? {
            return Ok(high);
        }
        let difference = low.exponent - high.exponent;
        let low_mantissa = if difference < -1074 {
            0.0
        } else {
            let scale = scalar(account, || 2.0_f64.powi(difference))?;
            scalar(account, || low.mantissa * scale)?
        };
        let sum = scalar(account, || high.mantissa + low_mantissa)?;
        let mut result = Self::new(sum, account)?;
        result.exponent += high.exponent;
        Ok(result)
    }
    fn shift(mut self, exponent: i32) -> Self {
        self.exponent += exponent;
        self
    }
    fn value(self, account: &Account) -> Result<f64> {
        let result = if scalar(account, || self.mantissa == 0.0)? {
            self.mantissa
        } else if self.exponent > 1023 {
            let scale = scalar(account, || 2.0_f64.powi(self.exponent - 1023))?;
            let high = scalar(account, || self.mantissa * scale)?;
            scalar(account, || high * 2.0_f64.powi(1023))?
        } else if self.exponent < -1022 {
            let scale = scalar(account, || 2.0_f64.powi(self.exponent + 1022))?;
            let high = scalar(account, || self.mantissa * scale)?;
            scalar(account, || high * f64::MIN_POSITIVE)?
        } else {
            let scale = scalar(account, || 2.0_f64.powi(self.exponent))?;
            scalar(account, || self.mantissa * scale)?
        };
        if scalar(account, || result.is_finite())? {
            Ok(result)
        } else {
            Err(Refusal::UnrepresentableGeometry.into())
        }
    }
}

fn affine(values: [Scaled; 6], account: &Account) -> Result<kurbo::Affine> {
    let mut output = [0.0; 6];
    for (output, value) in output.iter_mut().zip(values) {
        *output = value.value(account)?;
    }
    Ok(kurbo::Affine::new(output))
}
fn compose(transform: kurbo::Affine, local: [Scaled; 6], account: &Account) -> Result<[Scaled; 6]> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d, x, y] = transform.as_coeffs();
    let a = Scaled::new(a, account)?;
    let b = Scaled::new(b, account)?;
    let c = Scaled::new(c, account)?;
    let d = Scaled::new(d, account)?;
    let x = Scaled::new(x, account)?;
    let y = Scaled::new(y, account)?;
    let [u, v, w, z, ox, oy] = local;
    Ok([
        a.mul(u, account)?.add(c.mul(v, account)?, account)?,
        b.mul(u, account)?.add(d.mul(v, account)?, account)?,
        a.mul(w, account)?.add(c.mul(z, account)?, account)?,
        b.mul(w, account)?.add(d.mul(z, account)?, account)?,
        a.mul(ox, account)?
            .add(c.mul(oy, account)?, account)?
            .add(x, account)?,
        b.mul(ox, account)?
            .add(d.mul(oy, account)?, account)?
            .add(y, account)?,
    ])
}

#[derive(Clone, Copy)]
struct Range {
    first: f64,
    last: f64,
    scale: f64,
    delta: f64,
}
impl Range {
    fn new(first: f64, last: f64, account: &Account) -> Result<Self> {
        let span = scalar(account, || last - first)?;
        let (scale, delta) = if scalar(account, || span.is_finite())? {
            (span, 1.0)
        } else {
            let first_abs = scalar(account, || first.abs())?;
            let last_abs = scalar(account, || last.abs())?;
            let scale = scalar(account, || first_abs.max(last_abs))?;
            let low = scalar(account, || first / scale)?;
            let high = scalar(account, || last / scale)?;
            (scale, scalar(account, || high - low)?)
        };
        Ok(Self {
            first,
            last,
            scale,
            delta,
        })
    }
    fn normalize(self, value: f64, account: &Account) -> Result<f32> {
        // Source endpoint checks precede arithmetic; every executed division,
        // subtraction and narrowing is separately admitted.
        account.work(RenderWork::CoordinateVisits, 1)?;
        let normalized = if value == self.first {
            0.0
        } else {
            account.work(RenderWork::CoordinateVisits, 1)?;
            if value == self.last {
                1.0
            } else {
                account.work(RenderWork::CoordinateVisits, 1)?;
                let difference = value - self.first;
                account.work(RenderWork::CoordinateVisits, 1)?;
                let numerator = if difference.is_finite() {
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    difference / self.scale
                } else {
                    account.work(RenderWork::CoordinateVisits, 3)?;
                    value / self.scale - self.first / self.scale
                };
                account.work(RenderWork::CoordinateVisits, 1)?;
                numerator / self.delta
            }
        };
        account.work(RenderWork::CoordinateVisits, 1)?;
        Ok(normalized as f32)
    }
}

/// Directed binary32 cutoff of the exact scaled binary64 value. Scaling changes
/// only its exponent, so subnormal/overflow cases need no lossy f64 product.
pub(super) fn cutoff(value: f64, shift: i32, account: &Account) -> Result<f32> {
    let scaled = Scaled::new(value, account)?.shift(shift);
    if scalar(account, || value == 0.0)? {
        return Ok(0.0);
    }
    if scaled.exponent > 127 {
        return Ok(if scalar(account, || value > 0.0)? {
            f32::INFINITY
        } else {
            -f32::MAX
        });
    }
    if scaled.exponent < -149 {
        return Ok(if scalar(account, || value > 0.0)? {
            f32::from_bits(1)
        } else {
            0.0
        });
    }
    // This exponent range is safely normal in binary64 and exactly scalable.
    let exact = scaled.value(account)?;
    let rounded = scalar(account, || exact as f32)?;
    let widened = scalar(account, || f64::from(rounded))?;
    if scalar(account, || widened < exact)? {
        scalar(account, || rounded.next_up())
    } else {
        Ok(rounded)
    }
}

fn scaled_geometry(
    gradient: &Gradient,
    range: Option<Range>,
    shift: i32,
    account: &Account,
) -> Result<([Scaled; 6], Option<field::RadialReference>)> {
    let zero = Scaled::new(0.0, account)?;
    let (local, radial) = match gradient.kind() {
        GradientKind::Linear { start, end, .. } => {
            account.work(RenderWork::CoordinateVisits, 4)?;
            if start == end {
                return Err(Refusal::DegenerateLinearField.into());
            }
            let difference = |end: f64, start: f64| {
                Scaled::new(end, account)?.add(Scaled::new(-start, account)?, account)
            };
            let dx = difference(end.x(), start.x())?;
            let dy = difference(end.y(), start.y())?;
            let sx = Scaled::new(start.x(), account)?;
            let sy = Scaled::new(start.y(), account)?;
            let (u, v, ox, oy) = if let Some(range) = range {
                let scale = Scaled::new(range.scale, account)?
                    .mul(Scaled::new(range.delta, account)?, account)?;
                let first = Scaled::new(range.first, account)?;
                let origin = |d: Scaled, s: f64, base: Scaled| -> Result<Scaled> {
                    // FMA retains finite cancellation even when d*a overflows.
                    match d.value(account) {
                        Ok(d) => {
                            account.work(RenderWork::CoordinateVisits, 1)?;
                            let exact = d.mul_add(range.first, s);
                            account.work(RenderWork::CoordinateVisits, 1)?;
                            if exact.is_finite() {
                                return Scaled::new(exact, account);
                            }
                        }
                        Err(error) if error.accounting_diagnostic().is_some() => return Err(error),
                        Err(_) => {}
                    }
                    d.mul(first, account)?.add(base, account)
                };
                (
                    dx.mul(scale, account)?,
                    dy.mul(scale, account)?,
                    origin(dx, start.x(), sx)?,
                    origin(dy, start.y(), sy)?,
                )
            } else {
                (dx.shift(-shift), dy.shift(-shift), sx, sy)
            };
            let minus_v = Scaled {
                mantissa: -v.mantissa,
                ..v
            };
            ([u, v, minus_v, u, ox, oy], None)
        }
        GradientKind::Radial {
            center,
            radius_x,
            radius_y,
            ..
        } => {
            account.work(RenderWork::CoordinateVisits, 4)?;
            let scale = if let Some(range) = range {
                Scaled::new(range.scale, account)?
            } else {
                Scaled::new(1.0, account)?.shift(-shift)
            };
            let x = Scaled::new(*radius_x, account)?.mul(scale, account)?;
            let y = Scaled::new(*radius_y, account)?.mul(scale, account)?;
            let reference = if let Some(range) = range {
                account.work(RenderWork::CoordinateVisits, 2)?;
                field::RadialReference {
                    start: range.first / range.scale,
                    end: range.last / range.scale,
                    delta: range.delta,
                }
            } else {
                field::RadialReference {
                    start: 0.0,
                    end: 1.0,
                    delta: 1.0,
                }
            };
            (
                [
                    x,
                    zero,
                    zero,
                    y,
                    Scaled::new(center.x(), account)?,
                    Scaled::new(center.y(), account)?,
                ],
                Some(reference),
            )
        }
        GradientKind::Sweep { .. } => unreachable!("sweep uses normalized native path"),
    };
    Ok((local, radial))
}

fn geometry(
    gradient: &Gradient,
    range: Option<Range>,
    shift: i32,
    transform: kurbo::Affine,
    account: &Account,
) -> Result<(kurbo::Affine, kurbo::Affine, Option<field::RadialReference>)> {
    let (local, radial) = scaled_geometry(gradient, range, shift, account)?;
    let device = affine(compose(transform, local, account)?, account)?;
    // Linear preparation solves its local brush map from the device field.
    // Its unused unsolved local coefficient may overflow despite finite device geometry.
    let local = if radial.is_some() {
        affine(local, account)?
    } else {
        kurbo::Affine::IDENTITY
    };
    Ok((local, device, radial))
}

fn realize(
    gradient: &Gradient,
    range: Option<Range>,
    shift: i32,
    transform: kurbo::Affine,
    target: PhysicalSize,
    account: &Account,
) -> Result<(peniko::Gradient, BrushTransform)> {
    let (local, device, radial) = geometry(gradient, range, shift, transform, account)?;
    let comparison_only = range.is_none();
    if let Some(reference) = radial {
        account.work(RenderWork::CoordinateVisits, 3)?;
        let radii = (reference.start as f32, reference.end as f32);
        if !radii.0.is_finite() || !radii.1.is_finite() || radii.0 >= radii.1 {
            return Err(Refusal::InsufficientNativeCapacity.into());
        }
        let native = field::narrow_transform(device, account).map_err(geometry_error)?;
        field::qualify_radial(
            device,
            native,
            radii,
            reference,
            target,
            comparison_only,
            account,
        )
        .map_err(capacity_error)?;
        Ok((
            peniko::Gradient::new_two_point_radial((0., 0.), radii.0, (0., 0.), radii.1),
            BrushTransform {
                local,
                reference_device: device,
                radial: Some(reference),
                comparison_only,
            },
        ))
    } else {
        let mut brush = field::linear_brush_transform(transform, device, false, account)
            .map_err(geometry_error)?;
        brush.comparison_only = comparison_only;
        let composed = field::affine_product(transform, brush.local, account)?;
        let native = field::narrow_transform(composed, account).map_err(geometry_error)?;
        if !field::check_native_linear(native, account).map_err(geometry_error)? {
            return Err(Refusal::InsufficientNativeCapacity.into());
        }
        field::qualify_linear(
            brush.reference_device,
            native,
            target,
            comparison_only,
            account,
        )
        .map_err(capacity_error)?;
        Ok((peniko::Gradient::new_linear((0., 0.), (1., 0.)), brush))
    }
}

pub(super) fn prepare(
    gradient: &Gradient,
    transform: kurbo::Affine,
    target: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    let supplied = gradient.stops();
    account.work(RenderWork::GradientStopVisits, 1)?;
    let first = supplied[0];
    if supplied.len() == 1 {
        return Ok(FrameBrush::solid(first.color().into()));
    }
    account.work(RenderWork::GradientStopVisits, 1)?;
    let last = supplied[supplied.len() - 1];
    account.work(RenderWork::CoordinateVisits, 2)?;
    let (a, b) = (first.position(), last.position());
    if matches!(gradient.kind(), GradientKind::Radial { .. }) && b <= 0.0 {
        return Ok(FrameBrush::solid(last.color().into()));
    }
    let range = if a == b {
        None
    } else {
        Some(Range::new(a, b, account)?)
    };
    let mut shift = 0;
    let (native, brush) = match realize(gradient, range, shift, transform, target, account) {
        Ok(result) => result,
        Err(error)
            if range.is_none()
                && error.accounting_diagnostic().is_none()
                && error.gradient_realization_diagnostic()
                    != Some(&Refusal::DegenerateLinearField) =>
        {
            // A power-two basis is selected only when the original native basis
            // cannot qualify. No translation is introduced at the threshold.
            let (local, _) = scaled_geometry(gradient, None, 0, account)?;
            let device = compose(transform, local, account)?;
            let mut exponent = None;
            for value in &device[..4] {
                if !scalar(account, || value.mantissa == 0.0)? {
                    exponent =
                        Some(exponent.map_or(value.exponent, |e: i32| e.max(value.exponent)));
                }
            }
            shift = exponent.ok_or(error)?;
            realize(gradient, range, shift, transform, target, account)?
        }
        Err(error) => return Err(error),
    };
    let boundary = if range.is_some() {
        (0.0, 1.0)
    } else {
        let value = cutoff(a, shift, account)?;
        (value, value)
    };
    let endpoints = PadEndpoints::new(
        (boundary.0, first.color()),
        (boundary.1, last.color()),
        account,
    )?;
    let mut stops = account.temporary(supplied.len())?;
    let mut previous = None;
    for (index, stop) in supplied.iter().enumerate() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let position = stop.position();
        let offset = if let Some(range) = range {
            range.normalize(position, account)?
        } else {
            0.0
        };
        account.work(RenderWork::CoordinateVisits, 2)?;
        if !offset.is_finite() || previous.is_some_and(|(p, n)| position > p && offset <= n) {
            return Err(Refusal::UnrepresentableStop { stop_index: index }.into());
        }
        previous = Some((position, offset));
        account.work(RenderWork::CoordinateVisits, 4)?;
        stops.push_reserved(peniko::ColorStop::from((
            offset,
            peniko::Color::from(stop.color()),
        )));
    }
    let mut output = gradient_with_stops(native, stops, Some(brush))?;
    output.endpoints = Some(endpoints);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, GradientStop, Point};
    #[test]
    fn directed_cutoff_preserves_both_sides_of_binary32_coordinates() {
        let account = Account::for_test();
        let cases = [
            (0.0, 0, 0),
            (-0.0, 0, 0),
            (f64::from_bits(1), 0, 1),
            (-f64::from_bits(1), 0, 0),
            (f64::from(f32::from_bits(1)), 0, 1),
            (f64::from(f32::from_bits(1)).next_up(), 0, 2),
            (f64::from(f32::from_bits(1)).next_down(), 0, 1),
            (
                f64::from(0.25_f32).next_up(),
                0,
                0.25_f32.next_up().to_bits(),
            ),
            (f64::from(0.25_f32).next_down(), 0, 0.25_f32.to_bits()),
            (f64::MAX, 1024, f32::INFINITY.to_bits()),
            (-f64::MAX, 1024, (-f32::MAX).to_bits()),
            (f64::from_bits(1), -1024, 1),
            (-f64::from_bits(1), -1024, 0),
        ];
        for (value, shift, bits) in cases {
            assert_eq!(
                cutoff(value, shift, &account).unwrap().to_bits(),
                bits,
                "{value} *2^{shift}"
            );
        }
    }
    #[test]
    fn overflowing_source_span_has_finite_linear_and_radial_native_fields() {
        let account = Account::for_test();
        let tiny = f64::MIN_POSITIVE;
        let stops = vec![
            GradientStop::try_new(-f64::MAX, Color::BLACK).unwrap(),
            GradientStop::try_new(f64::MAX, Color::TRANSPARENT).unwrap(),
        ];
        let line =
            Gradient::try_linear(Point::new(0., 0.), Point::new(tiny, 0.), stops.clone()).unwrap();
        let ellipse = Gradient::try_radial(Point::new(0., 0.), tiny, tiny, stops).unwrap();
        for gradient in [line, ellipse] {
            let brush = prepare(
                &gradient,
                kurbo::Affine::IDENTITY,
                PhysicalSize::new(8, 8),
                &account,
            )
            .unwrap();
            let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
                panic!("native field");
            };
            assert_eq!(native.stops[0].offset, 0.0);
            assert_eq!(native.stops[1].offset, 1.0);
            if let peniko::GradientKind::Radial(radial) = native.kind {
                assert_eq!((radial.start_radius, radial.end_radius), (-1., 1.));
            }
        }
    }
    #[test]
    fn source_distinct_positions_that_collapse_after_normalization_refuse_the_later_stop() {
        let stops = vec![
            GradientStop::try_new(0., Color::BLACK).unwrap(),
            GradientStop::try_new(0.5, Color::BLACK).unwrap(),
            GradientStop::try_new(0.5_f64.next_up(), Color::BLACK).unwrap(),
            GradientStop::try_new(1., Color::BLACK).unwrap(),
        ];
        let gradient = Gradient::try_linear(Point::new(0., 0.), Point::new(8., 0.), stops).unwrap();
        let error = prepare(
            &gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(8, 8),
            &Account::for_test(),
        )
        .err()
        .unwrap();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::UnrepresentableStop { stop_index: 2 })
        );
    }
    #[test]
    fn negative_radial_stop_contribution_uses_unswapped_arbitrary_native_radii() {
        let gradient = Gradient::try_radial(
            Point::new(0., 0.),
            8.,
            4.,
            vec![
                GradientStop::try_new(-0.5, Color::BLACK).unwrap(),
                GradientStop::try_new(1., Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(
            &gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(8, 8),
            &Account::for_test(),
        )
        .unwrap();
        let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
            panic!("native field");
        };
        let peniko::GradientKind::Radial(radial) = native.kind else {
            panic!("radial field");
        };
        assert_eq!(radial.start_radius, -1.0 / 3.0);
        assert_eq!(radial.end_radius, 2.0 / 3.0);
        assert!(brush.endpoints.is_some());
    }
    #[test]
    fn fma_origin_retains_finite_cancellation_of_an_overflowing_product() {
        let account = Account::for_test();
        let gradient = Gradient::try_linear(
            Point::new(-f64::MAX, 0.),
            Point::new(0., 0.),
            vec![
                GradientStop::try_new(2., Color::BLACK).unwrap(),
                GradientStop::try_new(3., Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        // The logical origin MAX*2-MAX is finite; device shrink keeps its
        // actual native projection finite without materializing unused axes.
        let transform = kurbo::Affine::scale(f64::MIN_POSITIVE);
        let result = prepare(&gradient, transform, PhysicalSize::new(8, 8), &account);
        assert!(result.is_ok());
    }
    #[test]
    fn directed_cutoff_admits_exact_executed_actions_and_preserves_a_refused_prefix() {
        use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
        use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
        let value = f64::from(f32::from_bits(1));
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| cutoff(value, 0, account),
            )
            .unwrap();
        // Three normalized-source actions (zero test, bits, mantissa), one
        // threshold zero test, four materialization actions (zero test, scale,
        // product, finite test), and narrowing, widening, directed comparison.
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            11
        );
        let mut limited = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 10)
                    .unwrap(),
            ),
        );
        let error = limited
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| cutoff(value, 0, account),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = limited.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 10);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(cutoff(value, 0, &Account::for_test()).unwrap().to_bits(), 1);
    }
    #[test]
    fn scaled_affine_refusal_preserves_accounting_during_an_accepted_composition_prefix() {
        use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
        use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
        let fixture = Account::for_test();
        let z = Scaled::new(0., &fixture).unwrap();
        let one = Scaled::new(1., &fixture).unwrap();
        let local = [one, z, z, one, z, z];
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 20)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| compose(kurbo::Affine::IDENTITY, local, account),
            )
            .err()
            .unwrap();
        assert!(error.accounting_diagnostic().is_some());
        // Six coefficient captures, ten actual scalar decomposition actions,
        // then the first product and its three normalization actions. Admission
        // rejects the next product before evaluating it.
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 20);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        let result = affine(
            compose(kurbo::Affine::IDENTITY, local, &fixture).unwrap(),
            &fixture,
        )
        .unwrap();
        assert_eq!(result, kurbo::Affine::IDENTITY);
    }
    #[test]
    fn step_basis_uses_final_device_scale_when_the_original_native_basis_cannot_qualify() {
        let gradient = Gradient::try_linear(
            Point::new(0., 0.),
            Point::new(1., 0.),
            vec![
                GradientStop::try_new(0.5, Color::BLACK).unwrap(),
                GradientStop::try_new(0.5, Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(
            &gradient,
            kurbo::Affine::scale(2.0_f64.powi(500)),
            PhysicalSize::new(8, 8),
            &Account::for_test(),
        )
        .unwrap();
        let field = brush.transform.unwrap();
        assert!(field.comparison_only);
        assert_eq!(field.reference_device.as_coeffs(), [1., 0., 0., 1., 0., 0.]);
        let original = prepare(
            &gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(8, 8),
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(original.transform.unwrap().local, kurbo::Affine::IDENTITY);
    }
    #[test]
    fn radial_step_requires_finite_native_fine_arithmetic_and_can_rescale_its_basis() {
        let account = Account::for_test();
        let original = kurbo::Affine::scale(1.0e-18);
        let native = field::narrow_transform(original, &account).unwrap();
        let reference = field::RadialReference {
            start: 0.,
            end: 1.,
            delta: 1.,
        };
        let target = PhysicalSize::new(32, 32);
        // At device(31,0), each original rotated component is about2.192e19;
        // its square exceeds binary32 MAX while q remains below1e30.
        let error = field::qualify_radial(
            original,
            native,
            (0., 1.),
            reference,
            target,
            true,
            &account,
        )
        .map_err(capacity_error)
        .err()
        .expect("the original native map must refuse fine overflow");
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::InsufficientNativeCapacity)
        );

        let gradient = Gradient::try_radial(
            Point::new(0., 0.),
            1.0e-18,
            1.0e-18,
            vec![
                GradientStop::try_new(1.0e30, Color::BLACK).unwrap(),
                GradientStop::try_new(1.0e30, Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(&gradient, kurbo::Affine::IDENTITY, target, &account).unwrap();
        let solved = brush.transform.unwrap();
        let [x, b, c, y, tx, ty] = solved.reference_device.as_coeffs();
        assert_eq!([b, c, tx, ty], [0.; 4]);
        assert!((1.0..2.0).contains(&x) && (1.0..2.0).contains(&y));
        // Independent real bound for the selected concentric native setup:
        // the nudged equal-center map has maximal norm multiplier r²/(r²-1)
        // and center displacement sqrt(2)/4096. Unit-sized axes keep both
        // affine components, squares, sum, root and focal addition far below
        // binary32 MAX. The qualifier separately encloses permitted rounding.
        let epsilon = 1.0 / 4096.0;
        let r = 1.0 / (2.0_f64.sqrt() * epsilon);
        let multiplier = (r * r) / (r * r - 1.0);
        let norm_bound = (31.0_f64.hypot(31.0) / x.min(y) + 2.0_f64.sqrt() * epsilon) * multiplier;
        assert!(norm_bound * norm_bound < f64::from(f32::MAX));
        let scaled_threshold = 1.0e30 * (1.0e-18 / x);
        assert!(norm_bound < scaled_threshold);
        field::qualify_radial(
            solved.reference_device,
            field::narrow_transform(solved.reference_device, &account).unwrap(),
            (0., 1.),
            solved.radial.unwrap(),
            target,
            true,
            &account,
        )
        .unwrap();
    }
}
