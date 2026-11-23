//! Affine stop fixup for the zero-circle radial limit.
//!
//! The substituted radius approaches zero from above. Positions retain both
//! terms of `length + fraction * radius`, ordered first by length and then by
//! fraction. The result preserves expanded color occurrences and hints for
//! consumers that evaluate the limit in their own color and sampling domains.

use crate::PaintError;
use crate::accounting::{
    Meter, OperationKind, PaintOptions, PaintReport, ResourceKind, WorkKind, allocate, apply,
    bytes, charge, finish, release, setup_error,
};
use crate::gradient_stops::{
    GradientPosition, GradientStopItem, ScalarInterpolation, StopWork, UsedGradientRecord,
    distribute_with, expand_with, fixup_with, setup_scopes, validate_source,
};
use crate::radial_gradient::{RadialGradientCase, RadialGradientGeometry};
use core::cmp::Ordering;
use core::error::Error;
use std::{fmt, mem::take};
use surgeist_layout::geometry::Point;
use surgeist_ledger::{Change, Scope};

/// Refusal to produce a qualified affine stop list for a zero circle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZeroCircleGradientError {
    ExpectedZeroCircle,
    UnrepresentableDistribution {
        left_source_index: usize,
        left_position_index: usize,
        right_source_index: usize,
        right_position_index: usize,
    },
}
impl fmt::Display for ZeroCircleGradientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ExpectedZeroCircle => f.write_str("gradient geometry must be a zero circle"),
            Self::UnrepresentableDistribution {
                left_source_index,
                left_position_index,
                right_source_index,
                right_position_index,
            } => write!(
                f,
                "the affine distribution between position {left_position_index} of gradient item {left_source_index} and position {right_position_index} of gradient item {right_source_index} cannot be represented"
            ),
        }
    }
}
impl Error for ZeroCircleGradientError {}
impl From<ZeroCircleGradientError> for PaintError {
    fn from(value: ZeroCircleGradientError) -> Self {
        Self::ZeroCircleGradient(value)
    }
}

/// A finite fixed-up position `length + fraction * radius` in the zero-circle limit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZeroCircleGradientPosition {
    length: f64,
    fraction: f64,
}
impl ZeroCircleGradientPosition {
    /// The constant term, in the same physical units as the geometry.
    pub const fn length(self) -> f64 {
        self.length
    }

    /// The coefficient of the substituted radius, where 1 means 100%.
    pub const fn fraction(self) -> f64 {
        self.fraction
    }
}

/// Meaning of one actual expanded record in the fixed-up source order.
#[derive(Debug)]
pub enum ZeroCircleGradientRecordKind<'a, C> {
    Stop { payload: &'a C },
    Hint,
}

#[derive(Debug)]
struct PreparedRecord<'a, C> {
    source_index: usize,
    position_index: usize,
    position: ZeroCircleGradientPosition,
    kind: ZeroCircleGradientRecordKind<'a, C>,
}

/// Borrows one prepared record. Field access projects stored values directly.
#[derive(Debug)]
pub struct ZeroCircleGradientRecord<'a, C> {
    record: &'a PreparedRecord<'a, C>,
}
impl<'a, C> ZeroCircleGradientRecord<'a, C> {
    /// The authored source item's index.
    pub const fn source_index(&self) -> usize {
        self.record.source_index
    }

    /// The zero-based ordinal within a double-position stop, or zero otherwise.
    pub const fn position_index(&self) -> usize {
        self.record.position_index
    }

    /// The record's qualified finite affine position.
    pub const fn position(&self) -> ZeroCircleGradientPosition {
        self.record.position
    }

    /// The color payload reference or independent hint identity of this record.
    pub fn kind(&self) -> ZeroCircleGradientRecordKind<'a, C> {
        match self.record.kind {
            ZeroCircleGradientRecordKind::Stop { payload } => {
                ZeroCircleGradientRecordKind::Stop { payload }
            }
            ZeroCircleGradientRecordKind::Hint => ZeroCircleGradientRecordKind::Hint,
        }
    }
}

/// Complete affine stop positions and their authentic zero-circle center.
/// Equal positions remain distinct, and payloads borrow the exact input slice.
///
/// ```compile_fail,E0451
/// use surgeist_paint::zero_circle_gradient::ZeroCircleGradientStops;
/// fn fabricate(authentic: ZeroCircleGradientStops<'_, ()>) {
///     let _ = ZeroCircleGradientStops { ..authentic };
/// }
/// ```
#[derive(Debug)]
pub struct ZeroCircleGradientStops<'a, C> {
    source: &'a [GradientStopItem<C>],
    center: Point<f64>,
    records: Vec<PreparedRecord<'a, C>>,
}
impl<'a, C> ZeroCircleGradientStops<'a, C> {
    /// The exact caller-owned source slice.
    pub const fn source(&self) -> &'a [GradientStopItem<C>] {
        self.source
    }

    /// The finite box-local center supplied by the resolved geometry.
    pub const fn center(&self) -> Point<f64> {
        self.center
    }

    /// Every expanded record in fixed-up order, including coincident occurrences.
    pub fn records(&self) -> impl ExactSizeIterator<Item = ZeroCircleGradientRecord<'_, C>> {
        self.records
            .iter()
            .map(|record| ZeroCircleGradientRecord { record })
    }
}

/// One complete affine stop list or typed refusal, with the operation's accepted work.
#[derive(Debug)]
pub struct ZeroCircleGradientStopsOutcome<'a, C> {
    pub result: Result<ZeroCircleGradientStops<'a, C>, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves native stops and hints for authentic zero-circle radial geometry.
///
/// Missing endpoints default to 0% and 100%. Positioned records clamp in
/// ascending affine order, including hints. Each remaining omitted run is
/// equally spaced between its positioned neighbors in both components.
/// Distinct length intervals and, at a common length, distinct fraction
/// intervals must survive distribution. Exact cancellation stays at zero,
/// and a nonzero result that would round to zero is refused. Payload
/// evaluation is left to callers.
pub fn resolve_zero_circle_gradient_stops<'a, C>(
    geometry: &RadialGradientGeometry,
    source: &'a [GradientStopItem<C>],
    options: PaintOptions,
) -> ZeroCircleGradientStopsOutcome<'a, C> {
    let mut meter = match options.operation(OperationKind::ZeroCircleGradient) {
        Ok(meter) => meter,
        Err(error) => {
            return ZeroCircleGradientStopsOutcome {
                result: Err(error),
                report: None,
            };
        }
    };
    let result = setup_scopes(&mut meter).and_then(|scopes| {
        let mut construction = Construction {
            staging: Vec::new(),
            prepared: Vec::new(),
            fixed_owner: false,
        };
        construction.run(&mut meter, scopes, geometry, source)
    });
    let (result, report) = finish(meter, result);
    ZeroCircleGradientStopsOutcome { result, report }
}

const FIRST: GradientPosition = GradientPosition {
    length: 0.0,
    fraction: 0.0,
};
const LAST: GradientPosition = GradientPosition {
    length: 0.0,
    fraction: 1.0,
};

type AffineRecord = UsedGradientRecord<GradientPosition>;

struct Construction<'a, C> {
    staging: Vec<AffineRecord>,
    prepared: Vec<PreparedRecord<'a, C>>,
    fixed_owner: bool,
}
impl<'a, C> Construction<'a, C> {
    fn run(
        &mut self,
        meter: &mut Meter,
        scopes: [Scope; 6],
        geometry: &RadialGradientGeometry,
        source: &'a [GradientStopItem<C>],
    ) -> Result<ZeroCircleGradientStops<'a, C>, PaintError> {
        let root = meter.root();
        let result = self.resolve(meter, scopes, geometry, source);
        let cleanup = meter
            .with_phase(scopes[5], |p| {
                release(p, root, take(&mut self.staging), ResourceKind::ScratchBytes)?;
                if result.is_err() {
                    release(
                        p,
                        root,
                        take(&mut self.prepared),
                        ResourceKind::RetainedResultBytes,
                    )?;
                    if self.fixed_owner {
                        apply(
                            p,
                            root,
                            Change::Release(
                                ResourceKind::RetainedResultBytes,
                                bytes::<ZeroCircleGradientStops<'_, C>>(1)?,
                            ),
                        )?;
                    }
                }
                Ok(())
            })
            .map_err(setup_error)
            .and_then(|done| done);
        result.and_then(|value| cleanup.map(|()| value))
    }

    fn resolve(
        &mut self,
        meter: &mut Meter,
        scopes: [Scope; 6],
        geometry: &RadialGradientGeometry,
        source: &'a [GradientStopItem<C>],
    ) -> Result<ZeroCircleGradientStops<'a, C>, PaintError> {
        let root = meter.root();
        let (source, record_count, _) = meter
            .with_phase(scopes[0], |p| {
                let mut work = StopWork::new(p, scopes[0]);
                if !work.validation(|| matches!(geometry.case(), RadialGradientCase::ZeroCircle))? {
                    return Err(ZeroCircleGradientError::ExpectedZeroCircle.into());
                }
                validate_source(&mut work, source)
            })
            .map_err(setup_error)??
            .into_parts();
        meter
            .with_phase(scopes[1], |p| {
                self.staging = allocate(p, root, record_count, ResourceKind::ScratchBytes)?;
                expand_with(
                    &mut StopWork::new(p, scopes[1]),
                    source,
                    &mut self.staging,
                    FIRST,
                    |_, _, _, position| Ok(position),
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[2], |p| {
                fixup_with(
                    &mut StopWork::new(p, scopes[2]),
                    source,
                    &mut self.staging,
                    FIRST,
                    LAST,
                    affine_less,
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[3], |p| {
                distribute_with(
                    &mut StopWork::new(p, scopes[3]),
                    source,
                    &mut self.staging,
                    fill_affine_run,
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[4], |p| {
                self.prepare(&mut StopWork::new(p, scopes[4]), root, source)?;
                apply(
                    p,
                    root,
                    Change::Acquire(
                        ResourceKind::RetainedResultBytes,
                        bytes::<ZeroCircleGradientStops<'_, C>>(1)?,
                    ),
                )?;
                self.fixed_owner = true;
                charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                Ok(ZeroCircleGradientStops {
                    source,
                    center: geometry.center(),
                    records: take(&mut self.prepared),
                })
            })
            .map_err(setup_error)?
    }

    fn prepare(
        &mut self,
        work: &mut StopWork<'_, '_, '_>,
        root: Scope,
        source: &'a [GradientStopItem<C>],
    ) -> Result<(), PaintError> {
        self.prepared = allocate(
            work.meter(),
            root,
            self.staging.len(),
            ResourceKind::RetainedResultBytes,
        )?;
        for index in 0..self.staging.len() {
            let record = work.record(&self.staging, index)?;
            let kind = match work.source(source, record.source_index)? {
                GradientStopItem::Stop { payload, .. } => {
                    ZeroCircleGradientRecordKind::Stop { payload }
                }
                GradientStopItem::Hint { .. } => ZeroCircleGradientRecordKind::Hint,
            };
            work.emit(
                &mut self.prepared,
                PreparedRecord {
                    source_index: record.source_index,
                    position_index: record.position_index,
                    position: ZeroCircleGradientPosition {
                        length: record.position.length,
                        fraction: record.position.fraction,
                    },
                    kind,
                },
            )?;
        }
        Ok(())
    }
}

fn affine_less(
    work: &mut StopWork<'_, '_, '_>,
    a: GradientPosition,
    b: GradientPosition,
) -> Result<bool, PaintError> {
    if work.numeric(|| a.length < b.length)? {
        return Ok(true);
    }
    if work.numeric(|| a.length == b.length)? {
        return work.numeric(|| a.fraction < b.fraction);
    }
    Ok(false)
}

#[derive(Clone, Copy)]
enum Component {
    Constant(f64),
    Varying(ScalarInterpolation),
    Exact(ExactWeights),
}
impl Component {
    fn between(work: &mut StopWork<'_, '_, '_>, a: f64, b: f64) -> Result<Self, PaintError> {
        if work.numeric(|| a == b)? {
            return Ok(Self::Constant(a));
        }
        let interpolation = ScalarInterpolation::varying(work, a, b)?;
        let opposite_signs = matches!(interpolation, ScalarInterpolation::Convex { .. });
        // Same-sign terms cannot cancel. With at most 64-bit weights, a
        // subnormal mean requires each endpoint to be within 2^64 times the
        // smallest normal, whose biased exponent field is 1.
        const SUBNORMAL_ENDPOINT_BOUND: f64 = f64::from_bits(65_u64 << 52);
        if !opposite_signs {
            let start = work.numeric(|| a.abs())?;
            let end = work.numeric(|| b.abs())?;
            if work.numeric(|| start >= SUBNORMAL_ENDPOINT_BOUND)?
                || work.numeric(|| end >= SUBNORMAL_ENDPOINT_BOUND)?
            {
                return Ok(Self::Varying(interpolation));
            }
        }
        Ok(Self::Exact(ExactWeights {
            start: Magnitude::decode(work, a)?,
            end: Magnitude::decode(work, b)?,
            opposite_signs,
            negative_start: work.numeric(|| a.is_sign_negative())?,
            interpolation,
        }))
    }

    fn at(
        self,
        work: &mut StopWork<'_, '_, '_>,
        step: DistributionStep,
    ) -> Result<Option<f64>, PaintError> {
        match self {
            Self::Constant(value) => Ok(Some(value)),
            Self::Varying(interpolation) => {
                // Distinct same-sign endpoints cannot have an interior zero.
                let value = interpolation.at(work, step.t)?;
                Ok(work.numeric(|| value != 0.0)?.then_some(value))
            }
            Self::Exact(exact) => exact.at(work, step),
        }
    }
}

#[derive(Clone, Copy)]
struct DistributionStep {
    start_weight: usize,
    end_weight: usize,
    denominator: usize,
    denominator_float: f64,
    t: f64,
}

#[derive(Clone, Copy)]
struct ExactWeights {
    start: Magnitude,
    end: Magnitude,
    opposite_signs: bool,
    negative_start: bool,
    interpolation: ScalarInterpolation,
}
impl ExactWeights {
    fn at(
        self,
        work: &mut StopWork<'_, '_, '_>,
        step: DistributionStep,
    ) -> Result<Option<f64>, PaintError> {
        // Combine integer-weighted dyadics before division. This preserves
        // cancellation and avoids rounding either weight before a tie.
        let field = work.numeric(|| self.start.field.min(self.end.field))?;
        let Some(start) = self.start.aligned(work, field, step.start_weight)? else {
            return self.interpolation.at(work, step.t).map(Some);
        };
        let Some(end) = self.end.aligned(work, field, step.end_weight)? else {
            return self.interpolation.at(work, step.t).map(Some);
        };
        let (magnitude, negative) = if self.opposite_signs {
            let difference = work.numeric(|| start.abs_diff(end))?;
            if work.numeric(|| difference == 0)? {
                return Ok(Some(0.0));
            }
            let start_dominates = work.numeric(|| start > end)?;
            (difference, start_dominates == self.negative_start)
        } else {
            // A sum larger than u128 has at least 2^128 min-subnormal units
            // before division by a 64-bit denominator. It cannot be near zero.
            let Some(sum) = work.numeric(|| start.checked_add(end))? else {
                return self.interpolation.at(work, step.t).map(Some);
            };
            (sum, self.negative_start)
        };
        let value = match round_subnormal(work, magnitude, field, step.denominator)? {
            Some(0) => return Ok(None),
            Some(bits) => work.numeric(|| f64::from_bits(bits))?,
            None => approximate_normal(work, magnitude, field, step.denominator_float)?,
        };
        if negative {
            return work.numeric(|| -value).map(Some);
        }
        Ok(Some(value))
    }
}

fn round_subnormal(
    work: &mut StopWork<'_, '_, '_>,
    magnitude: u128,
    field: u32,
    denominator: usize,
) -> Result<Option<u64>, PaintError> {
    let shift = work.numeric(|| field - 1)?;
    let headroom = work.numeric(|| magnitude.leading_zeros())?;
    if work.numeric(|| shift > headroom)? {
        // The result in smallest-subnormal units exceeds 2^128 / 2^64.
        return Ok(None);
    }
    let units = work.numeric(|| magnitude << shift)?;
    let divisor = work.numeric(|| denominator as u128)?;
    let whole = work.numeric(|| units / divisor)?;
    // Below the smallest normal, the bit pattern is the integer count of
    // smallest-subnormal units. Rounding may also reach the smallest normal.
    if work.numeric(|| whole >= 1_u128 << 52)? {
        return Ok(None);
    }
    let remainder = work.numeric(|| units % divisor)?;
    let twice = work.numeric(|| remainder * 2)?;
    let round_up = match work.numeric(|| twice.cmp(&divisor))? {
        Ordering::Less => false,
        Ordering::Equal => work.numeric(|| whole % 2 != 0)?,
        Ordering::Greater => true,
    };
    let whole = work.numeric(|| whole as u64)?;
    if round_up {
        return work.numeric(|| whole + 1).map(Some);
    }
    Ok(Some(whole))
}

fn approximate_normal(
    work: &mut StopWork<'_, '_, '_>,
    magnitude: u128,
    field: u32,
    denominator: f64,
) -> Result<f64, PaintError> {
    let magnitude = work.numeric(|| magnitude as f64)?;
    let quotient = work.numeric(|| magnitude / denominator)?;
    // field=1 encodes a 2^-1074 unit. Larger fields increase the unit
    // by one power of two per step, spanning subnormal and normal units.
    let bits = if work.numeric(|| field <= 52)? {
        let shift = work.numeric(|| field - 1)?;
        work.numeric(|| 1_u64 << shift)?
    } else {
        let biased = work.numeric(|| field - 52)?;
        let biased = work.numeric(|| u64::from(biased))?;
        work.numeric(|| biased << 52)?
    };
    let unit = work.numeric(|| f64::from_bits(bits))?;
    work.numeric(|| quotient * unit)
}

#[derive(Clone, Copy)]
struct Magnitude {
    significand: u64,
    field: u32,
}
impl Magnitude {
    fn decode(work: &mut StopWork<'_, '_, '_>, value: f64) -> Result<Self, PaintError> {
        let bits = work.numeric(|| value.to_bits())?;
        let exponent = work.numeric(|| (bits >> 52) & 0x7ff)?;
        let fraction = work.numeric(|| bits & 0x000f_ffff_ffff_ffff)?;
        if work.numeric(|| exponent == 0)? {
            return Ok(Self {
                significand: fraction,
                field: 1,
            });
        }
        Ok(Self {
            significand: work.numeric(|| fraction | (1_u64 << 52))?,
            field: work.numeric(|| exponent as u32)?,
        })
    }

    fn aligned(
        self,
        work: &mut StopWork<'_, '_, '_>,
        field: u32,
        weight: usize,
    ) -> Result<Option<u128>, PaintError> {
        if work.numeric(|| self.significand == 0)? {
            return Ok(Some(0));
        }
        let significand = work.numeric(|| u128::from(self.significand))?;
        let weight = work.numeric(|| weight as u128)?;
        // Each interior weight is nonzero and at most 64 bits. Its product with
        // a nonzero 53-bit significand fits 117 bits. If alignment exceeds the
        // available space, the shifted term outweighs the other by at least
        // eleven bits and the convex fallback avoids cancellation.
        let product = work.numeric(|| significand * weight)?;
        let shift = work.numeric(|| self.field - field)?;
        let headroom = work.numeric(|| product.leading_zeros())?;
        if work.numeric(|| shift > headroom)? {
            return Ok(None);
        }
        work.numeric(|| product << shift).map(Some)
    }
}

fn fill_affine_run(
    work: &mut StopWork<'_, '_, '_>,
    records: &mut [AffineRecord],
    start: usize,
    end: usize,
    left: AffineRecord,
    right: AffineRecord,
) -> Result<(), PaintError> {
    let length = Component::between(work, left.position.length, right.position.length)?;
    let fraction = Component::between(work, left.position.fraction, right.position.fraction)?;
    let varies_in_length = match (length, fraction) {
        (Component::Constant(_), Component::Constant(_)) => {
            for index in start..end {
                work.write(records, index, left.position)?;
            }
            return Ok(());
        }
        (Component::Constant(_), _) => false,
        _ => true,
    };
    let basis = |position: GradientPosition| {
        if varies_in_length {
            position.length
        } else {
            position.fraction
        }
    };
    let denominator = (end - start)
        .checked_add(1)
        .ok_or(PaintError::CapacityOverflow)?;
    let denominator_float = work.numeric(|| denominator as f64)?;
    let mut previous = left;
    for index in start..end {
        let record = work.record_mut(records, index)?;
        let ordinal = (index - start)
            .checked_add(1)
            .ok_or(PaintError::CapacityOverflow)?;
        let ordinal_float = work.numeric(|| ordinal as f64)?;
        let step = DistributionStep {
            start_weight: denominator - ordinal,
            end_weight: ordinal,
            denominator,
            denominator_float,
            t: work.numeric(|| ordinal_float / denominator_float)?,
        };
        let (Some(length), Some(fraction)) = (length.at(work, step)?, fraction.at(work, step)?)
        else {
            return Err(distribution_loss(previous, *record));
        };
        let position = GradientPosition { length, fraction };
        if !work.numeric(|| position.length.is_finite())?
            || !work.numeric(|| position.fraction.is_finite())?
            || !work.numeric(|| basis(previous.position) < basis(position))?
        {
            return Err(distribution_loss(previous, *record));
        }
        record.position = position;
        previous = *record;
    }
    if !work.numeric(|| basis(previous.position) < basis(right.position))? {
        return Err(distribution_loss(previous, right));
    }
    Ok(())
}

fn distribution_loss(left: AffineRecord, right: AffineRecord) -> PaintError {
    ZeroCircleGradientError::UnrepresentableDistribution {
        left_source_index: left.source_index,
        left_position_index: left.position_index,
        right_source_index: right.source_index,
        right_position_index: right.position_index,
    }
    .into()
}

#[cfg(test)]
mod tests;
