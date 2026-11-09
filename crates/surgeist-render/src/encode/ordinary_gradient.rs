//! Contextual realization of ordinary finite scalar fields.
use super::{
    BrushTransform, FrameBrush, PreparedSampling,
    gradient_field::{self as field, scalar},
    gradient_with_stops, hint_program,
};
use crate::{
    Error, Gradient, GradientRealizationError as Refusal, PhysicalSize, Result,
    accounting::{Account, AccountedVec, RenderWork},
    paint::GradientKind,
    vello_engine::gradient_endpoints::{PadEndpoints, ProgramCoordinates},
};

pub(super) fn capacity_error(error: Error) -> Error {
    if error.accounting_diagnostic().is_some() || error.gradient_realization_diagnostic().is_some()
    {
        error
    } else {
        Refusal::InsufficientNativeCapacity.into()
    }
}
pub(super) fn geometry_error(error: Error) -> Error {
    if error.accounting_diagnostic().is_some() {
        error
    } else {
        Refusal::UnrepresentableGeometry.into()
    }
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
            const NORMALIZE_SCALE: f64 = f64::from_bits(1077_u64 << 52); //2^54.
            let normalized = scalar(account, || value * NORMALIZE_SCALE)?;
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
            const HIGH_SCALE: f64 = f64::from_bits(2046_u64 << 52); //2^1023.
            scalar(account, || high * HIGH_SCALE)?
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
        let normalized = if scalar(account, || value == self.first)? {
            0.0
        } else if scalar(account, || value == self.last)? {
            1.0
        } else {
            let difference = scalar(account, || value - self.first)?;
            let numerator = if scalar(account, || difference.is_finite())? {
                scalar(account, || difference / self.scale)?
            } else {
                let position = scalar(account, || value / self.scale)?;
                let first = scalar(account, || self.first / self.scale)?;
                scalar(account, || position - first)?
            };
            scalar(account, || numerator / self.delta)?
        };
        scalar(account, || normalized as f32)
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
            // Capture each authored coordinate once; all later arithmetic uses
            // these values rather than inspecting source points again.
            account.work(RenderWork::CoordinateVisits, 4)?;
            let (start_x, start_y, end_x, end_y) = (start.x(), start.y(), end.x(), end.y());
            if scalar(account, || start_x == end_x)? && scalar(account, || start_y == end_y)? {
                return Err(Refusal::DegenerateLinearField.into());
            }
            let difference = |end: f64, start: f64| -> Result<Scaled> {
                let end = Scaled::new(end, account)?;
                let negative_start = scalar(account, || -start)?;
                end.add(Scaled::new(negative_start, account)?, account)
            };
            let dx = difference(end_x, start_x)?;
            let dy = difference(end_y, start_y)?;
            let sx = Scaled::new(start_x, account)?;
            let sy = Scaled::new(start_y, account)?;
            let (u, v, ox, oy) = if let Some(range) = range {
                let scale = Scaled::new(range.scale, account)?
                    .mul(Scaled::new(range.delta, account)?, account)?;
                let first = Scaled::new(range.first, account)?;
                let origin = |d: Scaled, s: f64, base: Scaled| -> Result<Scaled> {
                    // FMA retains finite cancellation even when d*a overflows.
                    match d.value(account) {
                        Ok(d) => {
                            let exact = scalar(account, || d.mul_add(range.first, s))?;
                            if scalar(account, || exact.is_finite())? {
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
                    origin(dx, start_x, sx)?,
                    origin(dy, start_y, sy)?,
                )
            } else {
                (dx.shift(-shift), dy.shift(-shift), sx, sy)
            };
            let minus_v = Scaled {
                mantissa: scalar(account, || -v.mantissa)?,
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
            // Center coordinates and axes are captured once and reused.
            account.work(RenderWork::CoordinateVisits, 4)?;
            let (center_x, center_y, radius_x, radius_y) =
                (center.x(), center.y(), *radius_x, *radius_y);
            let scale = if let Some(range) = range {
                Scaled::new(range.scale, account)?
            } else {
                Scaled::new(1.0, account)?.shift(-shift)
            };
            let x = Scaled::new(radius_x, account)?.mul(scale, account)?;
            let y = Scaled::new(radius_y, account)?.mul(scale, account)?;
            let reference = if let Some(range) = range {
                field::RadialReference {
                    start: scalar(account, || range.first / range.scale)?,
                    end: scalar(account, || range.last / range.scale)?,
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
                    Scaled::new(center_x, account)?,
                    Scaled::new(center_y, account)?,
                ],
                Some(reference),
            )
        }
        GradientKind::HorizontalDistance { origin_x, .. } => {
            let origin_x = scalar(account, || *origin_x)?;
            let scale = if let Some(range) = range {
                Scaled::new(range.scale, account)?
            } else {
                Scaled::new(1.0, account)?.shift(-shift)
            };
            let one = Scaled::new(1.0, account)?;
            (
                [
                    scale,
                    zero,
                    zero,
                    one,
                    Scaled::new(origin_x, account)?,
                    zero,
                ],
                None,
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

fn narrow_radial_radii(reference: field::RadialReference, account: &Account) -> Result<(f32, f32)> {
    let start = scalar(account, || reference.start as f32)?;
    let end = scalar(account, || reference.end as f32)?;
    if !scalar(account, || start.is_finite())?
        || !scalar(account, || end.is_finite())?
        || scalar(account, || start >= end)?
    {
        return Err(Refusal::InsufficientNativeCapacity.into());
    }
    Ok((start, end))
}

fn qualify_stop_order(
    position: f64,
    offset: f32,
    previous: Option<(f64, f32)>,
    index: usize,
    account: &Account,
) -> Result<()> {
    if !scalar(account, || offset.is_finite())? {
        return Err(Refusal::UnrepresentableStop { stop_index: index }.into());
    }
    if let Some((p, n)) = previous
        && scalar(account, || position > p)?
        && scalar(account, || offset <= n)?
    {
        return Err(Refusal::UnrepresentableStop { stop_index: index }.into());
    }
    Ok(())
}

fn realize(
    gradient: &Gradient,
    range: Option<Range>,
    shift: i32,
    transform: kurbo::Affine,
    target: PhysicalSize,
    sampling: field::LinearSampling,
    account: &Account,
) -> Result<(peniko::Gradient, BrushTransform)> {
    let (local, device, radial) = geometry(gradient, range, shift, transform, account)?;
    let comparison_only = range.is_none();
    if let Some(reference) = radial {
        let radii = narrow_radial_radii(reference, account)?;
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
        field::qualify_linear_sampling(
            brush.reference_device,
            native,
            target,
            comparison_only,
            sampling,
            account,
        )
        .map_err(capacity_error)?;
        Ok((peniko::Gradient::new_linear((0., 0.), (1., 0.)), brush))
    }
}

fn realize_with_fallback(
    gradient: &Gradient,
    range: Option<Range>,
    transform: kurbo::Affine,
    target: PhysicalSize,
    sampling: field::LinearSampling,
    account: &Account,
) -> Result<(peniko::Gradient, BrushTransform, i32)> {
    let mut shift = 0;
    let (native, brush) =
        match realize(gradient, range, shift, transform, target, sampling, account) {
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
                realize(gradient, range, shift, transform, target, sampling, account)?
            }
            Err(error) => return Err(error),
        };
    Ok((native, brush, shift))
}

pub(super) fn prepare(
    gradient: &Gradient,
    transform: kurbo::Affine,
    target: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    if matches!(gradient.kind(), GradientKind::HorizontalDistance { .. }) {
        return prepare_horizontal_distance(gradient, transform, target, account);
    }
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
    if matches!(gradient.kind(), GradientKind::Radial { .. }) && scalar(account, || b <= 0.0)? {
        return Ok(FrameBrush::solid(last.color().into()));
    }
    let range = if scalar(account, || a == b)? {
        None
    } else {
        Some(Range::new(a, b, account)?)
    };
    let (native, brush, shift) = realize_with_fallback(
        gradient,
        range,
        transform,
        target,
        field::LinearSampling::Direct,
        account,
    )?;
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
    let mut hinted = false;
    for (index, stop) in supplied.iter().enumerate() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        if range.is_some() {
            hinted |= stop.hint_after().is_some();
        }
        let position = scalar(account, || stop.position())?;
        let offset = if let Some(range) = range {
            range.normalize(position, account)?
        } else {
            0.0
        };
        qualify_stop_order(position, offset, previous, index, account)?;
        previous = Some((position, offset));
        // Native Color copies the four already validated source channels.
        account.work(RenderWork::CoordinateVisits, 4)?;
        stops.append_slot()?.write(peniko::ColorStop::from((
            offset,
            peniko::Color::from(stop.color()),
        )));
    }
    let program = if let Some(range) = range.filter(|_| hinted) {
        Some(hint_program::prepare(
            supplied,
            &stops,
            |position, account| range.normalize(position, account),
            endpoints,
            crate::vello_engine::gradient_endpoints::ProgramCoordinates::Pad,
            account,
        )?)
    } else {
        None
    };
    let mut output = gradient_with_stops(native, stops, Some(brush))?;
    output.sampling = Some(match program {
        Some(program) => PreparedSampling::Program(program),
        None => PreparedSampling::PadRamp(endpoints),
    });
    Ok(output)
}

/// The dependency carrier owns only actual colours and source occurrence order.
/// Its uniform offsets cannot be confused with the semantic programme controls.
pub(super) fn horizontal_carrier_stops(
    supplied: &[crate::GradientStop],
    account: &Account,
) -> Result<AccountedVec<peniko::ColorStop>> {
    let mut carrier = account.temporary(supplied.len())?;
    let denominator = scalar(account, || (supplied.len() - 1) as f64)?;
    for (index, stop) in supplied.iter().enumerate() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let offset = if index == 0 {
            0.0
        } else if index == supplied.len() - 1 {
            1.0
        } else {
            let ordinal = scalar(account, || index as f64)?;
            let quotient = scalar(account, || ordinal / denominator)?;
            scalar(account, || quotient as f32)?
        };
        account.work(RenderWork::CoordinateVisits, 4)?;
        carrier.append_slot()?.write(peniko::ColorStop::from((
            offset,
            peniko::Color::from(stop.color()),
        )));
    }
    Ok(carrier)
}

/// No unused local coefficient is materialized before its final device product.
/// The transverse unit column completes the scalar coordinates; it is not an
/// authored radius, y coordinate or substituted ending shape.
pub(super) fn horizontal_device(
    origin_x: f64,
    basis: f64,
    transform: kurbo::Affine,
    account: &Account,
) -> Result<kurbo::Affine> {
    let zero = Scaled::new(0.0, account)?;
    let local = [
        Scaled::new(basis, account)?,
        zero,
        zero,
        Scaled::new(1.0, account)?,
        Scaled::new(origin_x, account)?,
        zero,
    ];
    affine(compose(transform, local, account)?, account)
}

pub(super) fn horizontal_brush(
    device: kurbo::Affine,
    transform: kurbo::Affine,
    target: PhysicalSize,
    sampling: field::LinearSampling,
    account: &Account,
) -> Result<BrushTransform> {
    // Repeat phase belongs after the sign-bit fold, never in this solved origin.
    let brush =
        field::linear_brush_transform(transform, device, false, account).map_err(geometry_error)?;
    let composed = field::affine_product(transform, brush.local, account)?;
    let native = field::narrow_transform(composed, account).map_err(geometry_error)?;
    if !field::check_native_linear(native, account).map_err(geometry_error)? {
        return Err(Refusal::InsufficientNativeCapacity.into());
    }
    field::qualify_linear_sampling(
        brush.reference_device,
        native,
        target,
        false,
        sampling,
        account,
    )
    .map_err(capacity_error)?;
    Ok(brush)
}

fn distance_position(value: f64, scale: f64, index: usize, account: &Account) -> Result<f32> {
    let quotient = scalar(account, || value / scale)?;
    let native = scalar(account, || quotient as f32)?;
    if !scalar(account, || native.is_finite())?
        || (scalar(account, || value != 0.0)? && scalar(account, || native == 0.0)?)
    {
        return Err(Refusal::UnrepresentableStop { stop_index: index }.into());
    }
    Ok(native)
}

fn prepare_horizontal_distance(
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
    if scalar(account, || b <= 0.0)? {
        return Ok(FrameBrush::solid(last.color().into()));
    }
    if scalar(account, || a == b)? {
        let (native, brush, shift) = realize_with_fallback(
            gradient,
            None,
            transform,
            target,
            field::LinearSampling::Direct,
            account,
        )?;
        let threshold = cutoff(a, shift, account)?;
        let endpoints = PadEndpoints::new(
            (threshold, first.color()),
            (threshold, last.color()),
            account,
        )?;
        let carrier = horizontal_carrier_stops(supplied, account)?;
        let mut output = gradient_with_stops(native, carrier, Some(brush))?;
        output.sampling = Some(PreparedSampling::HorizontalDistanceStep(endpoints));
        return Ok(output);
    }
    let low = scalar(account, || a.abs())?;
    let high = scalar(account, || b.abs())?;
    let largest = scalar(account, || low.max(high))?;
    let magnitude = Scaled::new(largest, account)?;
    let scale = Scaled {
        mantissa: 1.0,
        exponent: magnitude.exponent,
    }
    .value(account)?;
    let first_native = distance_position(a, scale, 0, account)?;
    let last_native = distance_position(b, scale, supplied.len() - 1, account)?;
    qualify_stop_order(
        b,
        last_native,
        Some((a, first_native)),
        supplied.len() - 1,
        account,
    )?;
    let first_wide = scalar(account, || f64::from(first_native))?;
    let last_wide = scalar(account, || f64::from(last_native))?;
    let span = scalar(account, || last_wide - first_wide)?;
    let range = Range {
        first: a,
        last: b,
        scale,
        delta: 1.0,
    };
    let (native, brush, _) = realize_with_fallback(
        gradient,
        Some(range),
        transform,
        target,
        field::LinearSampling::HorizontalDistancePad { span },
        account,
    )?;
    let endpoints = PadEndpoints::new(
        (first_native, first.color()),
        (last_native, last.color()),
        account,
    )?;
    let program = hint_program::prepare_projected(
        supplied,
        |index, position, account| distance_position(position, scale, index, account),
        |position, account| {
            let quotient = scalar(account, || position / scale)?;
            scalar(account, || quotient as f32)
        },
        endpoints,
        ProgramCoordinates::HorizontalDistancePad,
        account,
    )?;
    let carrier = horizontal_carrier_stops(supplied, account)?;
    let mut output = gradient_with_stops(native, carrier, Some(brush))?;
    output.sampling = Some(PreparedSampling::Program(program));
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
        assert!(brush.sampling.is_some());
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
                .attempt()
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
        let usage = limited.last_attempt().unwrap().attempt().unwrap().usage();
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
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
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
        .expect_err("the original native map must refuse fine overflow");
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
    #[test]
    fn negative_radial_solid_decision_accounts_its_executed_comparison() {
        use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
        use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
        let gradient = Gradient::try_radial(
            Point::new(0., 0.),
            1.,
            1.,
            vec![
                GradientStop::try_new(-2., Color::BLACK).unwrap(),
                GradientStop::try_new(-1., Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let mut context = RenderAccountingContext::default();
        context.run(RenderAttemptKind::Frame,RenderPhase::GradientPrepare,|account|{
            let brush=prepare(&gradient,kurbo::Affine::IDENTITY,PhysicalSize::new(8,8),account)?;
            assert!(matches!(brush.as_ref(),peniko::BrushRef::Solid(color) if color==peniko::Color::TRANSPARENT));
            Ok(())
        }).unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        // Two actual source position captures and the radial b<=0 comparison;
        // selecting a solid needs no stop-range equality or unused geometry.
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 3);
        assert_eq!(usage.work(RenderWork::GradientStopVisits), 2);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 2)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &gradient,
                        kurbo::Affine::IDENTITY,
                        PhysicalSize::new(8, 8),
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 2);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert!(
            prepare(
                &gradient,
                kurbo::Affine::IDENTITY,
                PhysicalSize::new(8, 8),
                &Account::for_test()
            )
            .is_ok()
        );
    }
    #[test]
    fn adjacent_stop_qualification_counts_only_executed_float_predicates() {
        use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
        use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
        for (position, offset, previous, actions) in [
            (0.0, 0.0, None, 1),
            (0.0, 0.0, Some((0.0, 0.0)), 2),
            (1.0, 1.0, Some((0.0, 0.0)), 3),
        ] {
            let mut context = RenderAccountingContext::default();
            context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::GradientPrepare,
                    |account| qualify_stop_order(position, offset, previous, 1, account),
                )
                .unwrap();
            // Finite offset; then source strict order if a predecessor exists;
            // then native interval separation only for a strict source pair.
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::CoordinateVisits),
                actions
            );
        }
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 2)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| qualify_stop_order(1.0, 1.0, Some((0.0, 0.0)), 1, account),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            2
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
        assert!(qualify_stop_order(1., 1., Some((0., 0.)), 1, &Account::for_test()).is_ok());
        let error =
            qualify_stop_order(1., 0., Some((0., 0.)), 7, &Account::for_test()).unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::UnrepresentableStop { stop_index: 7 })
        );
    }
    #[test]
    fn radial_radius_narrowing_admits_casts_and_each_executed_native_check() {
        use crate::accounting::{RenderAttemptKind, RenderPhase};
        use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
        let reference = field::RadialReference {
            start: 0.,
            end: 1.,
            delta: 1.,
        };
        let mut context = RenderAccountingContext::default();
        let radii = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| narrow_radial_radii(reference, account),
            )
            .unwrap();
        assert_eq!(radii, (0., 1.));
        // Two casts, two finiteness checks, one native radius-order comparison.
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            5
        );
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 4)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| narrow_radial_radii(reference, account),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            4
        );
        assert_eq!(
            narrow_radial_radii(reference, &Account::for_test()).unwrap(),
            (0., 1.)
        );
    }
    #[test]
    fn degenerate_line_decision_admits_range_equality_and_both_point_components() {
        use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
        use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
        let gradient = Gradient::try_linear(
            Point::new(2., 3.),
            Point::new(2., 3.),
            vec![
                GradientStop::try_new(0., Color::BLACK).unwrap(),
                GradientStop::try_new(1., Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let mut context = RenderAccountingContext::default();
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &gradient,
                        kurbo::Affine::IDENTITY,
                        PhysicalSize::new(8, 8),
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::DegenerateLinearField)
        );
        // Two stop-position captures, source equality, subtraction+finiteness
        // for the nonzero range, Scaled zero check, four point captures, and
        // both executed point-component comparisons.
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            12
        );
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 11)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    prepare(
                        &gradient,
                        kurbo::Affine::IDENTITY,
                        PhysicalSize::new(8, 8),
                        account,
                    )
                    .map(drop)
                },
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            11
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
    }
    #[test]
    fn distance_program_keeps_signed_controls_separate_from_its_uniform_carrier() {
        let gradient = Gradient::try_horizontal_distance(
            8.0,
            vec![
                GradientStop::try_new(-4.0, Color::BLACK).unwrap(),
                GradientStop::try_new(4.0, Color::TRANSPARENT).unwrap(),
                GradientStop::try_new(12.0, Color::BLACK).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(
            &gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(32, 8),
            &Account::for_test(),
        )
        .unwrap();
        let Some(PreparedSampling::Program(program)) = &brush.sampling else {
            panic!("nonzero distance uses its analytic programme");
        };
        assert!(matches!(
            program.coordinates,
            ProgramCoordinates::HorizontalDistancePad
        ));
        // S=8: semantic controls are p/S, not shifted to the first stop.
        assert_eq!(
            program
                .records
                .iter()
                .map(|r| r.position)
                .collect::<Vec<_>>(),
            [-0.5, 0.5, 1.5]
        );
        let peniko::Brush::Gradient(carrier) = &brush.brush else {
            panic!("linear carrier")
        };
        assert!(matches!(carrier.kind, peniko::GradientKind::Linear(_)));
        assert_eq!(
            carrier.stops.iter().map(|r| r.offset).collect::<Vec<_>>(),
            [0.0, 0.5, 1.0]
        );
        let inverse = brush.transform.unwrap().reference_device.inverse();
        for (x, expected) in [(0.0, -1.0), (8.0, 0.0), (16.0, 1.0)] {
            let point = inverse * kurbo::Point::new(x, 4.0);
            assert_eq!(point.x, expected);
        }
    }

    #[test]
    fn distance_overflowing_stop_span_uses_its_finite_final_device_basis() {
        let gradient = Gradient::try_horizontal_distance(
            0.0,
            vec![
                GradientStop::try_new(-f64::MAX, Color::BLACK).unwrap(),
                GradientStop::try_new(f64::MAX, Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(
            &gradient,
            kurbo::Affine::scale(f64::MIN_POSITIVE),
            PhysicalSize::new(8, 4),
            &Account::for_test(),
        )
        .unwrap();
        let Some(PreparedSampling::Program(program)) = &brush.sampling else {
            panic!("programme")
        };
        assert_eq!(program.records[0].position, -2.0);
        assert_eq!(program.records[1].position, 2.0);
        assert_eq!(
            brush.transform.unwrap().reference_device.as_coeffs(),
            [2.0, 0.0, -0.0, 2.0, 0.0, 0.0]
        );
    }

    #[test]
    fn distance_tiny_step_retains_an_unshifted_native_comparison_threshold() {
        let threshold = f64::from(f32::from_bits(1));
        let gradient = Gradient::try_horizontal_distance(
            0.0,
            vec![
                GradientStop::try_new(threshold, Color::BLACK).unwrap(),
                GradientStop::try_new(threshold, Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(
            &gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(8, 4),
            &Account::for_test(),
        )
        .unwrap();
        let Some(PreparedSampling::HorizontalDistanceStep(endpoints)) = &brush.sampling else {
            panic!("directed step descriptor");
        };
        assert_eq!(
            (
                endpoints.first_position_bits(),
                endpoints.last_position_bits()
            ),
            (1, 1)
        );
        assert!(brush.transform.unwrap().comparison_only);
        assert_eq!(
            brush.transform.unwrap().reference_device.as_coeffs(),
            [1.0, 0.0, -0.0, 1.0, 0.0, 0.0]
        );
    }
}
