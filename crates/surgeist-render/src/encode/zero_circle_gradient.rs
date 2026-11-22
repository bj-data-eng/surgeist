//! Native realization of the affine stop field of a zero-circle gradient.

use std::cmp::Ordering;

use super::{
    BrushTransform, FrameBrush, PreparedSampling,
    gradient_field::{self as field, ShaderScalar as Bound, scalar},
    hint_program::{self, Logarithms},
    ordinary_gradient::{capacity_error, geometry_error},
    repeating_gradient::radial_device_period,
};
use crate::{
    Result,
    accounting::{Account, AccountedVec, RenderAccountingError, RenderPhase, RenderWork},
    color_reduction::{PremultipliedSum, premultiplied},
    error::GradientRealizationError as Refusal,
    geometry::{PhysicalSize, Point},
    paint::{
        Color,
        zero_circle::{ZeroCircleGradient, ZeroCirclePosition, ZeroCircleStop},
    },
    vello_engine::gradient_endpoints::{CircleCoordinates, CircleRecord, CircleSegment},
};

const ZERO_SAMPLE: ZeroCirclePosition = ZeroCirclePosition {
    length: 0.0,
    fraction: 0.0,
};

pub(super) fn prepare(
    gradient: &ZeroCircleGradient,
    transform: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    let account = &account.at(RenderPhase::GradientPrepare);
    account.phase(RenderPhase::GradientPrepare, || {
        let stops = gradient.stops();
        let (first, last) = match stops {
            [first, .., last] => (first, last),
            [only] => {
                account.work(RenderWork::GradientStopVisits, 1)?;
                return solid(only.color, account);
            }
            [] => return Err(RenderAccountingError::Invariant.into()),
        };
        account.work(RenderWork::GradientStopVisits, 2)?;
        account.work(RenderWork::CoordinateVisits, 4)?;
        let first_position = first.position;
        let last_position = last.position;
        let comparison_only = scalar(account, || first_position.length == last_position.length)?;
        let repeat_origin = if gradient.is_repeating() {
            if comparison_only {
                let fraction =
                    Measure::new(first_position.fraction, last_position.fraction, account)?;
                let rule = if scalar(account, || fraction.span == 0.0)? {
                    Average::Uniform
                } else {
                    Average::Fraction(fraction)
                };
                return average(stops, rule, account);
            }
            let length = Measure::new(first_position.length, last_position.length, account)?;
            // The underresolved branch needs only the linear map and stops.
            // In particular, it never constructs a transformed center or hint program.
            field::check_finite(transform, account).map_err(geometry_error)?;
            let singular_value = radial_device_period(transform, account)?;
            let side = length.device_threshold(
                first_position.length,
                last_position.length,
                singular_value,
                account,
            )?;
            if side == Ordering::Less
                || (side == Ordering::Equal
                    && scalar(account, || last_position.fraction < first_position.fraction)?)
            {
                return average(stops, Average::Length(length), account);
            }
            Some(first_position)
        } else {
            if scalar(account, || last_position <= ZERO_SAMPLE)? {
                return solid(last.color, account);
            }
            None
        };

        let center = gradient.center();
        let (brush_transform, radius_upper, radius_scale) =
            prepare_carrier(center, transform, target_extent, comparison_only, account)?;
        let projection = Projection {
            radius_scale,
            origin: repeat_origin,
        };
        let records = prepare_records(stops, projection, account).map_err(capacity_error)?;
        let coordinates = match repeat_origin {
            Some(origin) => repeat_coordinates(origin, radius_scale, &records, account)?,
            None => CircleCoordinates::Pad,
        };
        qualify_sampling(&records, coordinates, radius_upper, account).map_err(capacity_error)?;

        // The two inline stops select Vello's radial draw. Colour comes from
        // the analytic records, which retain the actual stop occurrences.
        let brush = peniko::Gradient::new_two_point_radial((0.0, 0.0), 0.0, (0.0, 0.0), 1.0)
            .with_stops([
                (0.0, peniko::Color::TRANSPARENT),
                (1.0, peniko::Color::TRANSPARENT),
            ]);
        Ok(FrameBrush {
            brush: peniko::Brush::Gradient(brush),
            transform: Some(brush_transform),
            sampling: Some(PreparedSampling::Circle {
                records,
                coordinates,
            }),
            _hold: None,
            _payload: None,
        })
    })
}

fn solid(color: Color, account: &Account) -> Result<FrameBrush> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    Ok(FrameBrush::solid(color.to_peniko()))
}

/// A span stays usable for weighting even when direct endpoint subtraction
/// would overflow binary64.
#[derive(Clone, Copy)]
struct Measure {
    scale: f64,
    span: f64,
}

impl Measure {
    fn new(first: f64, last: f64, account: &Account) -> Result<Self> {
        let span = scalar(account, || last - first)?;
        if scalar(account, || span.is_finite())? {
            return Ok(Self { scale: 1.0, span });
        }
        let first = scalar(account, || first * 0.5)?;
        let last = scalar(account, || last * 0.5)?;
        let span = scalar(account, || last - first)?;
        Ok(Self { scale: 0.5, span })
    }

    fn half_segment(self, first: f64, last: f64, account: &Account) -> Result<f64> {
        let first = scalar(account, || first * self.scale)?;
        let last = scalar(account, || last * self.scale)?;
        let difference = scalar(account, || last - first)?;
        let normalized = scalar(account, || difference / self.span)?;
        scalar(account, || normalized * 0.5)
    }

    fn device_threshold(
        self,
        first: f64,
        last: f64,
        singular_value: f64,
        account: &Account,
    ) -> Result<Ordering> {
        let threshold = scalar(account, || 2.0 * self.scale)?;
        let period = scalar(account, || singular_value * self.span)?;
        // Only values within two binary64 steps of the power-of-two
        // threshold can cross it after restoring a half-ULP span residual.
        let lower = scalar(account, || threshold.next_down())?;
        let lower = scalar(account, || lower.next_down())?;
        if scalar(account, || period < lower)? {
            return Ok(Ordering::Less);
        }
        let upper = scalar(account, || threshold.next_up())?;
        let upper = scalar(account, || upper.next_up())?;
        if scalar(account, || period > upper)? {
            return Ok(Ordering::Greater);
        }
        // TwoSum recovers the low part of last - first. Its operands
        // match the exactly scaled endpoints used to form the high part.
        let a = scalar(account, || last * self.scale)?;
        let b = scalar(account, || -first * self.scale)?;
        let b_head = scalar(account, || self.span - a)?;
        let a_head = scalar(account, || self.span - b_head)?;
        let a_tail = scalar(account, || a - a_head)?;
        let b_tail = scalar(account, || b - b_head)?;
        let tail = scalar(account, || a_tail + b_tail)?;
        let high_residual = scalar(account, || self.span.mul_add(singular_value, -threshold))?;
        if scalar(account, || high_residual == 0.0)? {
            // Here the positive device scale cannot change the residual
            // sign, even when multiplying a subnormal tail flushes to zero.
            return scalar(account, || tail.partial_cmp(&0.0))?
                .ok_or_else(|| Refusal::UnrepresentableGeometry.into());
        }
        // Near cancellation high_residual is exactly representable: a
        // binary64 product near one or two has at most 106 significand bits.
        // The nonzero residual and a cancelling product cannot underflow.
        let residual = scalar(account, || tail.mul_add(singular_value, high_residual))?;
        scalar(account, || residual.partial_cmp(&0.0))?
            .ok_or_else(|| Refusal::UnrepresentableGeometry.into())
    }

    fn project(self, coordinate_scale: f64, refusal: Refusal, account: &Account) -> Result<f32> {
        let denominator = scalar(account, || self.scale * coordinate_scale)?;
        let span = scalar(account, || self.span / denominator)?;
        if scalar(account, || self.span != 0.0)? && scalar(account, || span == 0.0)? {
            return Err(refusal.into());
        }
        project(span, refusal, account)
    }
}

enum Average {
    Uniform,
    Fraction(Measure),
    Length(Measure),
}

fn average(stops: &[ZeroCircleStop], rule: Average, account: &Account) -> Result<FrameBrush> {
    let mut total = PremultipliedSum::new();
    match rule {
        Average::Uniform => {
            let intervals = scalar(account, || stops.len() - 1)?;
            let denominator = scalar(account, || intervals as f64)?;
            for (index, stop) in stops.iter().enumerate() {
                account.work(RenderWork::GradientStopVisits, 1)?;
                let color = premultiplied(stop.color, account)?;
                let numerator = if index == 0 || index == intervals {
                    0.5
                } else {
                    1.0
                };
                let weight = scalar(account, || numerator / denominator)?;
                total.add_weighted(color, weight, account)?;
            }
        }
        Average::Length(measure) | Average::Fraction(measure) => {
            let mut previous = None;
            for stop in stops {
                account.work(RenderWork::GradientStopVisits, 1)?;
                let color = premultiplied(stop.color, account)?;
                let position = scalar(account, || match rule {
                    Average::Length(_) => stop.position.length,
                    _ => stop.position.fraction,
                })?;
                if let Some((previous_position, previous_color)) = previous {
                    let weight = measure.half_segment(previous_position, position, account)?;
                    total.add_segment(previous_color, color, weight, account)?;
                }
                previous = Some((position, color));
            }
        }
    }
    solid(total.finish(account)?, account)
}

fn prepare_carrier(
    center: Point,
    transform: kurbo::Affine,
    target: PhysicalSize,
    comparison_only: bool,
    account: &Account,
) -> Result<(BrushTransform, f64, f64)> {
    account.work(RenderWork::CoordinateVisits, 2)?;
    let (cx, cy) = (center.x(), center.y());
    let reference = field::RadialReference {
        start: 0.0,
        end: 1.0,
        delta: 1.0,
    };
    let prepare = |scale| -> Result<(BrushTransform, f64, f64)> {
        let local = kurbo::Affine::new([scale, 0.0, 0.0, scale, cx, cy]);
        let device = field::affine_product(transform, local, account)?;
        let native = field::narrow_transform(device, account).map_err(geometry_error)?;
        let radius_upper =
            field::qualify_zero_circle(device, native, target, comparison_only, account)
                .map_err(capacity_error)?;
        Ok((
            BrushTransform {
                local,
                reference_device: device,
                radial: Some(reference),
                comparison_only,
            },
            radius_upper,
            scale,
        ))
    };
    match prepare(1.0) {
        Ok(carrier) => Ok(carrier),
        Err(error) if error.accounting_diagnostic().is_some() => Err(error),
        Err(error) => {
            // Choose a binary scale from the approximate reached local
            // radius, then independently certify the actual narrowed carrier.
            let local = kurbo::Affine::translate((cx, cy));
            let device = field::affine_product(transform, local, account)?;
            let scale = normalization_scale(device, target, account)?;
            if scalar(account, || scale == 1.0)? {
                return Err(error);
            }
            prepare(scale)
        }
    }
}

fn normalization_scale(
    device: kurbo::Affine,
    target: PhysicalSize,
    account: &Account,
) -> Result<f64> {
    // Scale selection is a heuristic only. The independently checked candidate
    // below owns the validation even when this estimate rounds.
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d, x, y] = device.as_coeffs();
    let ad = scalar(account, || a * d)?;
    let bc = scalar(account, || b * c)?;
    let determinant = scalar(account, || ad - bc)?;
    let inverse_determinant = scalar(account, || 1.0 / determinant)?;
    let width = scalar(account, || {
        f64::from(target.width().saturating_sub(1).max(1))
    })?;
    let height = scalar(account, || {
        f64::from(target.height().saturating_sub(1).max(1))
    })?;
    let mut radius = 0.0_f64;
    for (coordinate, extent) in [(a, height), (b, width), (c, height), (d, width)] {
        let inverse = scalar(account, || coordinate * inverse_determinant)?;
        let magnitude = scalar(account, || inverse.abs())?;
        let reached = scalar(account, || magnitude * extent)?;
        radius = scalar(account, || radius.max(reached))?;
    }
    let x = scalar(account, || x.abs())?;
    let y = scalar(account, || y.abs())?;
    let translated = scalar(account, || x.max(y))?;
    let near = scalar(account, || width.min(height))?;
    let factor = scalar(account, || translated / near)?;
    let factor = scalar(account, || factor.max(1.0))?;
    let radius = scalar(account, || radius * factor)?;
    if !scalar(account, || radius.is_finite())? || scalar(account, || radius <= 0.0)? {
        return Err(Refusal::UnrepresentableGeometry.into());
    }
    let bits = scalar(account, || radius.to_bits())?;
    let exponent_bits = scalar(account, || bits & (0x7ff_u64 << 52))?;
    let normal_bits = scalar(account, || exponent_bits.max(1_u64 << 52))?;
    scalar(account, || f64::from_bits(normal_bits))
}

fn project(value: f64, refusal: Refusal, account: &Account) -> Result<f32> {
    if scalar(account, || value == 0.0)? {
        return Ok(0.0);
    }
    let narrowed = scalar(account, || value as f32)?;
    if !scalar(account, || narrowed.is_normal())? {
        return Err(refusal.into());
    }
    Ok(narrowed)
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct NativePosition {
    length: f32,
    fraction: f32,
}

#[derive(Clone, Copy)]
struct Projection {
    radius_scale: f64,
    origin: Option<ZeroCirclePosition>,
}

impl Projection {
    fn position(
        self,
        position: ZeroCirclePosition,
        refusal: Refusal,
        account: &Account,
    ) -> Result<NativePosition> {
        if let Some(origin) = self.origin {
            let length = Measure::new(origin.length, position.length, account)?;
            let fraction = Measure::new(origin.fraction, position.fraction, account)?;
            return Ok(NativePosition {
                length: length.project(self.radius_scale, refusal, account)?,
                fraction: fraction.project(1.0, refusal, account)?,
            });
        }
        let length = scalar(account, || position.length / self.radius_scale)?;
        if scalar(account, || length == 0.0)? && scalar(account, || position.length != 0.0)? {
            return Err(refusal.into());
        }
        Ok(NativePosition {
            length: project(length, refusal, account)?,
            fraction: project(position.fraction, refusal, account)?,
        })
    }
}

#[derive(Clone, Copy)]
struct ProjectedStop<'a> {
    source: &'a ZeroCircleStop,
    authored: ZeroCirclePosition,
    position: NativePosition,
}

fn prepare_records(
    stops: &[ZeroCircleStop],
    projection: Projection,
    account: &Account,
) -> Result<AccountedVec<CircleRecord>> {
    let mut records = account.temporary(stops.len())?;
    let mut previous: Option<ProjectedStop<'_>> = None;
    let mut logs = None;
    for (index, current) in stops.iter().enumerate() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        account.work(RenderWork::CoordinateVisits, 2)?;
        let refusal = Refusal::UnrepresentableStop { stop_index: index };
        let authored = current.position;
        let position = projection.position(authored, refusal, account)?;
        let right = ProjectedStop {
            source: current,
            authored,
            position,
        };
        if let Some(left) = previous {
            let source_same_ring = scalar(account, || left.authored.length == authored.length)?;
            let native_same_ring = scalar(account, || left.position.length == position.length)?;
            let source_order = scalar(account, || left.authored.partial_cmp(&authored))?;
            let native_order = scalar(account, || left.position.partial_cmp(&position))?;
            if source_same_ring != native_same_ring || source_order != native_order {
                return Err(refusal.into());
            }
            let outgoing = segment(left, right, index - 1, projection, &mut logs, account)?;
            append_record(
                &mut records,
                left.source.color,
                left.position,
                outgoing,
                account,
            )?;
        }
        previous = Some(right);
    }
    if let Some(last) = previous {
        append_record(
            &mut records,
            last.source.color,
            last.position,
            CircleSegment::Linear,
            account,
        )?;
    }
    Ok(records)
}

fn segment(
    left: ProjectedStop<'_>,
    right: ProjectedStop<'_>,
    index: usize,
    projection: Projection,
    logs: &mut Option<Logarithms>,
    account: &Account,
) -> Result<CircleSegment> {
    let native_left = left.position;
    let native_right = right.position;
    if scalar(account, || native_left == native_right)? {
        return Ok(CircleSegment::Linear);
    }
    let same_ring = scalar(account, || native_left.length == native_right.length)?;
    account.work(RenderWork::CoordinateVisits, 1)?;
    let Some(hint) = left.source.hint_after else {
        let (a, b) = if same_ring {
            (native_left.fraction, native_right.fraction)
        } else {
            (native_left.length, native_right.length)
        };
        hint_program::linear(a, b, account)?;
        return Ok(CircleSegment::Linear);
    };
    account.work(RenderWork::CoordinateVisits, 2)?;
    if scalar(account, || hint == left.authored)? {
        return Ok(CircleSegment::HintAtStart);
    }
    if scalar(account, || hint == right.authored)? {
        return Ok(CircleSegment::HintAtEnd);
    }
    let refusal = Refusal::UnrepresentableHint { stop_index: index };
    let native_hint = projection.position(hint, refusal, account)?;
    if !scalar(account, || native_left < native_hint)?
        || !scalar(account, || native_hint < native_right)?
    {
        return Err(refusal.into());
    }
    if same_ring {
        return power(
            native_left.fraction,
            native_hint.fraction,
            native_right.fraction,
            index,
            logs,
            account,
        );
    }
    let left_ring = scalar(account, || hint.length == left.authored.length)?;
    let right_ring = scalar(account, || hint.length == right.authored.length)?;
    let native_left_ring = scalar(account, || native_hint.length == native_left.length)?;
    let native_right_ring = scalar(account, || native_hint.length == native_right.length)?;
    if left_ring != native_left_ring || right_ring != native_right_ring {
        return Err(refusal.into());
    }
    if left_ring {
        return Ok(CircleSegment::HintOnLeftRing);
    }
    if right_ring {
        hint_program::linear(native_hint.fraction, native_right.fraction, account)?;
        return Ok(CircleSegment::HintOnRightRing {
            fraction: native_hint.fraction,
        });
    }
    power(
        native_left.length,
        native_hint.length,
        native_right.length,
        index,
        logs,
        account,
    )
}

fn power(
    left: f32,
    hint: f32,
    right: f32,
    index: usize,
    logs: &mut Option<Logarithms>,
    account: &Account,
) -> Result<CircleSegment> {
    let logs = match logs {
        Some(logs) => logs,
        slot @ None => slot.insert(Logarithms::new(account)?),
    };
    let exponent = hint_program::power(left, hint, right, index, logs, account)?;
    Ok(CircleSegment::Power { exponent })
}

fn append_record(
    records: &mut AccountedVec<CircleRecord>,
    color: Color,
    position: NativePosition,
    outgoing: CircleSegment,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let (r, g, b, a) = (color.r(), color.g(), color.b(), color.a());
    let premultiplied = [
        scalar(account, || r * a)?,
        scalar(account, || g * a)?,
        scalar(account, || b * a)?,
        a,
    ];
    records.append_slot()?.write(CircleRecord {
        length: position.length,
        fraction: position.fraction,
        premultiplied,
        outgoing,
    });
    Ok(())
}

fn qualify_sampling(
    records: &[CircleRecord],
    coordinates: CircleCoordinates,
    radius_upper: f64,
    account: &Account,
) -> Result<()> {
    let fraction = match coordinates {
        CircleCoordinates::Repeat { .. } => qualify_repeat(coordinates, radius_upper, account)?,
        CircleCoordinates::Pad => Bound::ZERO,
    };
    // The right-ring hint can sample a negative secondary coordinate even
    // when the hint itself is positive. Prove all reached numerator and
    // quotient intermediates and the authored hint's divisor.
    let mut previous = CircleSegment::Linear;
    for record in records {
        account.work(RenderWork::GradientStopVisits, 1)?;
        if let CircleSegment::HintOnRightRing { fraction: hint } = previous {
            let right = scalar(account, || f64::from(record.fraction))?;
            let hint = scalar(account, || -f64::from(hint))?;
            let divisor = Bound::exact(right).add(Bound::exact(hint), account)?;
            let numerator = Bound::exact(right).add(fraction.neg(account)?, account)?;
            numerator.div(divisor, account)?;
        }
        previous = record.outgoing;
    }
    Ok(())
}

fn repeat_coordinates(
    origin: ZeroCirclePosition,
    radius_scale: f64,
    records: &[CircleRecord],
    account: &Account,
) -> Result<CircleCoordinates> {
    let projection = Projection {
        radius_scale,
        origin: None,
    };
    let origin = projection.position(
        origin,
        Refusal::UnrepresentableStop { stop_index: 0 },
        account,
    )?;
    let last = records.last().ok_or(RenderAccountingError::Invariant)?;
    account.work(RenderWork::GradientStopVisits, 1)?;
    account.work(RenderWork::CoordinateVisits, 2)?;
    // The first relative record is exactly (0,0). Use the actual last
    // relative record as the period so there is no competing rounding.
    Ok(CircleCoordinates::Repeat {
        origin_length: origin.length,
        origin_fraction: origin.fraction,
        span_length: last.length,
        span_fraction: last.fraction,
    })
}

fn qualify_repeat(
    coordinates: CircleCoordinates,
    radius_upper: f64,
    account: &Account,
) -> Result<Bound> {
    let CircleCoordinates::Repeat {
        origin_length,
        origin_fraction,
        span_length,
        span_fraction,
    } = coordinates
    else {
        return Err(RenderAccountingError::Invariant.into());
    };
    let raw = Bound::checked(0.0, radius_upper, account)?;
    let origin = Bound::exact(scalar(account, || -f64::from(origin_length))?);
    let length = Bound::exact(scalar(account, || f64::from(span_length))?);
    let fraction = Bound::exact(scalar(account, || -f64::from(span_fraction))?);
    let fraction_origin = Bound::exact(scalar(account, || -f64::from(origin_fraction))?);
    let exact_delta = raw.real_add(origin, account)?;
    let delta = raw.add(origin, account)?;
    let quotient = delta.div(length, account)?;
    let epsilon = Bound::exact(f64::from(f32::EPSILON));
    let min_normal = Bound::exact(f64::from(f32::MIN_POSITIVE));
    // The origin is zero or normal. Enclose raw input FTZ, subtraction
    // rounding, and a possible flush of the numerator before division.
    let delta_error = exact_delta
        .abs(account)?
        .real_add(min_normal, account)?
        .real_mul(epsilon, account)?
        .real_add(Bound::exact(3.0 * f32::MIN_POSITIVE as f64), account)?;
    let division_error = delta
        .abs(account)?
        .real_div(length, account)?
        .real_mul(Bound::exact(2.5 * f32::EPSILON as f64), account)?
        .real_add(min_normal, account)?;
    let quotient_error = delta_error
        .real_div(length, account)?
        .real_add(division_error, account)?;
    if scalar(account, || quotient_error.upper >= 1.0)? {
        return Err(Refusal::InsufficientNativeCapacity.into());
    }
    // Exact side tests correct the candidate by up to one. The secondary
    // tie at an exact leading seam may then decrement it once more.
    let floor_lower = scalar(account, || quotient.lower.floor())?;
    let floor_upper = scalar(account, || quotient.upper.floor())?;
    let lower = scalar(account, || floor_lower - 2.0)?;
    let upper = scalar(account, || floor_upper + 1.0)?;
    const EXACT_INTEGER_LIMIT: f64 = 16_777_216.0;
    if scalar(account, || lower < -EXACT_INTEGER_LIMIT)?
        || scalar(account, || upper > EXACT_INTEGER_LIMIT)?
    {
        return Err(Refusal::InsufficientNativeCapacity.into());
    }
    let cycle = Bound::checked(lower, upper, account)?;
    cycle.mul(length, account)?;
    if scalar(account, || span_fraction == 0.0)? {
        return Ok(fraction_origin);
    }
    fraction_origin.add(cycle.mul(fraction, account)?, account)
}

pub(super) fn native_glyph_transform(
    brush: BrushTransform,
    paint_to_device: kurbo::Affine,
    target_extent: PhysicalSize,
    records: &[CircleRecord],
    coordinates: CircleCoordinates,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    let local = field::narrow_transform(brush.local, account).map_err(geometry_error)?;
    let shape = field::narrow_transform(paint_to_device, account).map_err(geometry_error)?;
    let composed = field::native_product(shape, local, account).map_err(geometry_error)?;
    let radius_upper = field::qualify_zero_circle(
        brush.reference_device,
        composed,
        target_extent,
        brush.comparison_only,
        account,
    )
    .map_err(capacity_error)?;
    qualify_sampling(records, coordinates, radius_upper, account).map_err(capacity_error)?;
    Ok(local)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAD_PROJECTION: Projection = Projection {
        radius_scale: 1.0,
        origin: None,
    };

    const fn position(length: f64, fraction: f64) -> ZeroCirclePosition {
        ZeroCirclePosition { length, fraction }
    }

    const fn stop(
        position: ZeroCirclePosition,
        hint_after: Option<ZeroCirclePosition>,
    ) -> ZeroCircleStop {
        ZeroCircleStop {
            position,
            color: Color::BLACK,
            hint_after,
        }
    }

    #[test]
    fn hints_on_the_same_radius_as_an_endpoint_keep_their_source_role() {
        let account = Account::for_test();
        let left = position(4.0, -2.0);
        let right = position(8.0, 2.0);
        for (hint, expected) in [
            (left, CircleSegment::HintAtStart),
            (position(4.0, -1.0), CircleSegment::HintOnLeftRing),
            (position(6.0, 12.0), CircleSegment::Power { exponent: 1.0 }),
            (
                position(8.0, -2.0),
                CircleSegment::HintOnRightRing { fraction: -2.0 },
            ),
            (right, CircleSegment::HintAtEnd),
        ] {
            let stops = [stop(left, Some(hint)), stop(right, None)];
            let records = prepare_records(&stops, PAD_PROJECTION, &account).unwrap();
            match (records[0].outgoing, expected) {
                (CircleSegment::HintAtStart, CircleSegment::HintAtStart)
                | (CircleSegment::HintOnLeftRing, CircleSegment::HintOnLeftRing)
                | (CircleSegment::HintAtEnd, CircleSegment::HintAtEnd) => {}
                (
                    CircleSegment::Power { exponent: actual },
                    CircleSegment::Power { exponent: expected },
                )
                | (
                    CircleSegment::HintOnRightRing { fraction: actual },
                    CircleSegment::HintOnRightRing { fraction: expected },
                ) => assert_eq!(actual, expected),
                (actual, expected) => panic!("expected {expected:?}, got {actual:?}"),
            }
        }
    }

    #[test]
    fn distinct_leading_radii_cannot_narrow_to_the_same_ring() {
        let account = Account::for_test();
        let stops = [
            stop(position(4.0, 0.0), None),
            stop(position(4.0_f64.next_up(), 1.0), None),
        ];
        let error = prepare_records(&stops, PAD_PROJECTION, &account).unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::UnrepresentableStop { stop_index: 1 }),
        );
    }

    #[test]
    fn a_distinct_leading_hint_cannot_narrow_to_an_endpoint_ring() {
        let account = Account::for_test();
        let stops = [
            stop(position(4.0, 0.0), Some(position(4.0_f64.next_up(), 1.0))),
            stop(position(8.0, 0.0), None),
        ];
        let error = prepare_records(&stops, PAD_PROJECTION, &account).unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&Refusal::UnrepresentableHint { stop_index: 0 }),
        );
    }

    #[test]
    fn signed_repeat_cycles_reject_an_uncertain_floor_and_overflowing_fractions() {
        let account = Account::for_test();
        let coordinates = |origin_length, span_fraction| CircleCoordinates::Repeat {
            origin_length,
            origin_fraction: -0.5,
            span_length: 2.0,
            span_fraction,
        };
        assert!(qualify_repeat(coordinates(16.0, 1.0), 8.0, &account).is_ok());
        let Err(folded) = qualify_repeat(coordinates(16_000_000.0, 0.0), 8.0, &account)
            .map_err(capacity_error)
        else {
            panic!("the quotient can drift by a complete cycle");
        };
        assert_eq!(
            folded.gradient_realization_diagnostic(),
            Some(&Refusal::InsufficientNativeCapacity),
        );
        let Err(fraction) =
            qualify_repeat(coordinates(16.0, f32::MAX), 8.0, &account).map_err(capacity_error)
        else {
            panic!("the translated fraction must remain finite");
        };
        assert_eq!(
            fraction.gradient_realization_diagnostic(),
            Some(&Refusal::InsufficientNativeCapacity),
        );
    }

    #[test]
    fn nonbinary_repeat_period_is_the_relative_end_pair_with_an_independent_origin() {
        let account = Account::for_test();
        let first = position(-0.1, -0.3);
        let last = position(0.3, -0.5);
        let stops = [stop(first, None), stop(last, None)];
        let projection = Projection {
            radius_scale: 1.0,
            origin: Some(first),
        };
        let records = prepare_records(&stops, projection, &account).unwrap();
        let coordinates = repeat_coordinates(first, 1.0, &records, &account).unwrap();
        assert_eq!((records[0].length, records[0].fraction), (0.0, 0.0));
        assert_eq!((records[1].length, records[1].fraction), (0.4, -0.2));
        let CircleCoordinates::Repeat {
            origin_length,
            origin_fraction,
            span_length,
            span_fraction,
        } = coordinates
        else {
            panic!("expected repeating coordinates");
        };
        assert_eq!((origin_length, origin_fraction), (-0.1, -0.3));
        assert_eq!((span_length, span_fraction), (0.4, -0.2));
        let fraction = qualify_repeat(coordinates, 8.0, &account).unwrap();
        // At raw 0.7, cycle 2 owns the first leading seam and the actual
        // secondary sample is -(-0.3)-2*(-0.2) = 0.7.
        assert!(fraction.lower < 0.7 && fraction.upper > 0.7);
    }

    #[test]
    fn period_low_part_distinguishes_both_sides_of_a_rounded_two_pixel_tie() {
        let account = Account::for_test();
        let tiny = 2.0_f64.powi(-54);
        for (first, expected) in [
            (tiny, Ordering::Less),
            (0.0, Ordering::Equal),
            (-tiny, Ordering::Greater),
        ] {
            let length = Measure::new(first, 2.0, &account).unwrap();
            assert_eq!(length.span, 2.0);
            assert_eq!(
                length.device_threshold(first, 2.0, 1.0, &account).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn power_of_two_carrier_keeps_a_wide_zoom_at_a_finite_native_radius() {
        let account = Account::for_test();
        let center = Point::try_new(0.0, 0.0).unwrap();
        let paint_to_device = kurbo::Affine::scale(1.0e20);
        let target = PhysicalSize::new(32, 32);
        let (brush, extent, scale) =
            prepare_carrier(center, paint_to_device, target, false, &account).unwrap();
        assert!(scale < 1.0);
        assert_eq!(scale.to_bits() & ((1_u64 << 52) - 1), 0);
        assert_eq!(
            brush.reference_device.as_coeffs(),
            [1.0e20 * scale, 0.0, 0.0, 1.0e20 * scale, 0.0, 0.0],
        );
        assert!(extent >= 31.0_f64.hypot(31.0) / (1.0e20 * scale));
    }
}
