//! Native zero-height radial color projection. No replacement axes are created.

use crate::{
    GradientPosition, GradientStopItem, PaintError, PaintOptions, PaintPhase, PaintReport,
    RadialGradientCase, RadialGradientGeometry, ResourceKind, WorkKind,
    accounting::{
        Meter, OperationKind, allocate, apply, bytes, charge, finish, release, setup_error,
    },
    gradient_stops::{
        ScalarPosition, StopWork, UsedGradientRecord, distribute_with, expand_with, fill_run,
        fixup_with, interior_omitted, validate_source,
    },
};
use std::mem::take;
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

/// Selects the ordinary last-color or repeating average semantic case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientRepeat {
    None,
    Repeat,
}

/// Native case or required color-measure representation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZeroHeightGradientError {
    ExpectedZeroHeight,
    UnrepresentableWeight {
        left_source_index: usize,
        left_position_index: usize,
        right_source_index: usize,
        right_position_index: usize,
    },
}
impl From<ZeroHeightGradientError> for PaintError {
    fn from(value: ZeroHeightGradientError) -> Self {
        Self::ZeroHeightGradient(value)
    }
}

/// One actual color occurrence and its finite nonnegative color-measure weight.
/// The payload is borrowed from the exact source; hints are never colors.
#[derive(Debug)]
pub struct WeightedGradientColor<'a, C> {
    pub source_index: usize,
    pub position_index: usize,
    pub payload: &'a C,
    pub weight: f64,
}

#[derive(Debug)]
enum Measure {
    Last(UsedGradientRecord),
    Average(Vec<UsedGradientRecord>),
}

/// Authenticated whole-image zero-height colors, borrowing their exact source.
/// Ordinary selection emits only the last occurrence at weight one. Repetition
/// emits every expanded color occurrence in order, including zero-weight ones.
/// This is a color measure, not a finite gradient-line position proof.
#[derive(Debug)]
pub struct ZeroHeightGradientColors<'a, C> {
    source: &'a [GradientStopItem<C>],
    measure: Measure,
}
impl<'a, C> ZeroHeightGradientColors<'a, C> {
    pub const fn source(&self) -> &'a [GradientStopItem<C>] {
        self.source
    }

    pub fn colors(&self) -> impl ExactSizeIterator<Item = WeightedGradientColor<'_, C>> {
        let records = match &self.measure {
            Measure::Last(record) => std::slice::from_ref(record),
            Measure::Average(records) => records.as_slice(),
        };
        records.iter().map(|record| {
            let GradientStopItem::Stop { payload, .. } = &self.source[record.source_index] else {
                unreachable!("authenticated color occurrence")
            };
            WeightedGradientColor {
                source_index: record.source_index,
                position_index: record.position_index,
                payload,
                weight: record.position,
            }
        })
    }
}

/// One complete borrowed color measure or typed refusal with accepted work.
#[derive(Debug)]
pub struct ZeroHeightGradientColorsOutcome<'a, C> {
    pub result: Result<ZeroHeightGradientColors<'a, C>, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves zero-height radial colors using the selected substituted-width
/// limit toward positive infinity. Geometry must be the authentic ZeroHeight
/// case. All raw grammar/components are validated before selecting the last
/// color or the specified stop-only repeating average. No payload is evaluated.
///
/// Hints affect ordering and omitted-color positioning, but have no independent
/// color weight or curve integral. The result cannot outlive its source.
pub fn resolve_zero_height_gradient_colors<'a, C>(
    geometry: &RadialGradientGeometry,
    source: &'a [GradientStopItem<C>],
    repeat: GradientRepeat,
    options: PaintOptions,
) -> ZeroHeightGradientColorsOutcome<'a, C> {
    let mut meter = match options.operation(OperationKind::ZeroHeightGradient) {
        Ok(meter) => meter,
        Err(error) => {
            return ZeroHeightGradientColorsOutcome {
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
            return ZeroHeightGradientColorsOutcome { result, report };
        }
    };
    let mut pairs = Vec::new();
    let mut normalized = Vec::new();
    let mut weights = Vec::new();
    let mut fixed_owner = false;
    let result = (|| {
        let validated = meter
            .with_phase(scopes[0], |p| {
                let mut work = StopWork::new(p, scopes[0]);
                if !work.validation(|| matches!(geometry.case(), RadialGradientCase::ZeroHeight))? {
                    return Err(ZeroHeightGradientError::ExpectedZeroHeight.into());
                }
                validate_source(&mut work, source)
            })
            .map_err(setup_error)??;
        let last = if repeat == GradientRepeat::None || validated.colors == 1 {
            Some(UsedGradientRecord {
                source_index: source.len() - 1,
                position_index: validated.last_ordinal,
                position: 1.0,
            })
        } else {
            meter
                .with_phase(scopes[1], |p| {
                    pairs = allocate(p, root, validated.records, ResourceKind::ScratchBytes)?;
                    expand_with(
                        &mut StopWork::new(p, scopes[1]),
                        source,
                        &mut pairs,
                        GradientPosition {
                            fraction: 0.0,
                            length: 0.0,
                        },
                        |_, _, _, position| Ok(position),
                    )
                })
                .map_err(setup_error)??;
            meter
                .with_phase(scopes[2], |p| {
                    fixup_with(
                        &mut StopWork::new(p, scopes[2]),
                        source,
                        &mut pairs,
                        GradientPosition {
                            fraction: 0.0,
                            length: 0.0,
                        },
                        GradientPosition {
                            fraction: 1.0,
                            length: 0.0,
                        },
                        lex_less,
                    )
                })
                .map_err(setup_error)??;
            meter
                .with_phase(scopes[3], |p| {
                    weights =
                        allocate(p, root, validated.colors, ResourceKind::RetainedResultBytes)?;
                    let mut work = StopWork::new(p, scopes[3]);
                    let first = work.record(&pairs, 0)?;
                    let last = work.record(&pairs, pairs.len() - 1)?;
                    let dimension =
                        if work.numeric(|| first.position.fraction != last.position.fraction)? {
                            Some(true)
                        } else if work.numeric(|| first.position.length != last.position.length)? {
                            Some(false)
                        } else {
                            None
                        };
                    if let Some(fraction) = dimension {
                        normalized = allocate(
                            work.meter(),
                            root,
                            validated.records,
                            ResourceKind::ScratchBytes,
                        )?;
                        project(&mut work, source, &pairs, &mut normalized, fraction)?;
                        distribute_with(&mut work, source, &mut normalized, fill_run)?;
                        weighted(&mut work, source, &normalized, &mut weights)
                    } else {
                        uniform(&mut work, source, &pairs, &mut weights, validated.colors)
                    }
                })
                .map_err(setup_error)??;
            None
        };
        meter
            .with_phase(scopes[4], |p| {
                apply(
                    p,
                    root,
                    Change::Acquire(
                        ResourceKind::RetainedResultBytes,
                        bytes::<ZeroHeightGradientColors<'_, C>>(1)?,
                    ),
                )?;
                fixed_owner = true;
                if last.is_some() {
                    charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                }
                charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                let measure = if let Some(record) = last {
                    Measure::Last(record)
                } else {
                    Measure::Average(take(&mut weights))
                };
                Ok(ZeroHeightGradientColors { source, measure })
            })
            .map_err(setup_error)?
    })();
    let cleanup = meter
        .with_phase(scopes[5], |p| {
            release(p, root, take(&mut normalized), ResourceKind::ScratchBytes)?;
            release(p, root, take(&mut pairs), ResourceKind::ScratchBytes)?;
            if result.is_err() {
                release(
                    p,
                    root,
                    take(&mut weights),
                    ResourceKind::RetainedResultBytes,
                )?;
                if fixed_owner {
                    apply(
                        p,
                        root,
                        Change::Release(
                            ResourceKind::RetainedResultBytes,
                            bytes::<ZeroHeightGradientColors<'_, C>>(1)?,
                        ),
                    )?;
                }
            }
            Ok(())
        })
        .map_err(setup_error)
        .and_then(|done| done);
    let result = result.and_then(|value| cleanup.map(|()| value));
    let (result, report) = finish(meter, result);
    ZeroHeightGradientColorsOutcome { result, report }
}

fn setup_scopes(meter: &mut Meter) -> Result<[Scope; 6], PaintError> {
    let mut scopes = [meter.root(); 6];
    for (slot, phase) in scopes.iter_mut().zip([
        PaintPhase::GradientValidation,
        PaintPhase::GradientStopResolution,
        PaintPhase::GradientStopFixup,
        PaintPhase::GradientStopDistribution,
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

fn lex_less(
    work: &mut StopWork<'_, '_, '_>,
    a: GradientPosition,
    b: GradientPosition,
) -> Result<bool, PaintError> {
    if work.numeric(|| a.fraction < b.fraction)? {
        Ok(true)
    } else if work.numeric(|| a.fraction == b.fraction)? {
        work.numeric(|| a.length < b.length)
    } else {
        Ok(false)
    }
}

#[derive(Clone, Copy)]
struct Normalized {
    value: f64,
    // Exact order in the chosen dimension, before floating normalization. For a
    // nonconstant omitted run, every adjacent occurrence is strictly separated.
    strict: bool,
}
impl ScalarPosition for Normalized {
    fn scalar(self) -> f64 {
        self.value
    }
    fn with_scalar(self, value: f64) -> Self {
        Self { value, ..self }
    }
}

fn selected(position: GradientPosition, fraction: bool) -> f64 {
    if fraction {
        position.fraction
    } else {
        position.length
    }
}

fn project<C>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    pairs: &[UsedGradientRecord<GradientPosition>],
    normalized: &mut Vec<UsedGradientRecord<Normalized>>,
    fraction: bool,
) -> Result<(), PaintError> {
    let first = work.record(pairs, 0)?;
    let last = work.record(pairs, pairs.len() - 1)?;
    let range = Range::new(
        work,
        selected(first.position, fraction),
        selected(last.position, fraction),
    )?;
    let mut previous: Option<(usize, f64)> = None;
    for index in 0..pairs.len() {
        let record = work.record(pairs, index)?;
        work.emit(
            normalized,
            UsedGradientRecord {
                source_index: record.source_index,
                position_index: record.position_index,
                position: Normalized {
                    value: 0.0,
                    strict: false,
                },
            },
        )?;
        if interior_omitted(work, source, record, index, pairs.len())? {
            continue;
        }
        let value = selected(record.position, fraction);
        if let Some((left, previous_value)) = previous {
            let strict = work.numeric(|| previous_value < value)?;
            for slot in left + 1..=index {
                work.record_mut(normalized, slot)?.position.strict = strict;
            }
        }
        previous = Some((index, value));
        let hint = matches!(
            work.source(source, record.source_index)?,
            GradientStopItem::Hint { .. }
        );
        let needed = if hint {
            let left = work.record(pairs, index - 1)?;
            if interior_omitted(work, source, left, index - 1, pairs.len())? {
                true
            } else {
                let right = work.record(pairs, index + 1)?;
                interior_omitted(work, source, right, index + 1, pairs.len())?
            }
        } else {
            true
        };
        if needed {
            let value = range.normalize(work, value)?;
            work.record_mut(normalized, index)?.position.value = value;
        }
    }
    Ok(())
}

fn loss<P, Q>(left: UsedGradientRecord<P>, right: UsedGradientRecord<Q>) -> PaintError {
    ZeroHeightGradientError::UnrepresentableWeight {
        left_source_index: left.source_index,
        left_position_index: left.position_index,
        right_source_index: right.source_index,
        right_position_index: right.position_index,
    }
    .into()
}

fn uniform<C>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    pairs: &[UsedGradientRecord<GradientPosition>],
    weights: &mut Vec<UsedGradientRecord>,
    count: usize,
) -> Result<(), PaintError> {
    let denominator = work.numeric(|| (count - 1) as f64)?;
    let full = work.numeric(|| 1.0 / denominator)?;
    let half = work.numeric(|| full * 0.5)?;
    let mut ordinal = 0;
    for index in 0..pairs.len() {
        let record = work.record(pairs, index)?;
        if matches!(
            work.source(source, record.source_index)?,
            GradientStopItem::Hint { .. }
        ) {
            continue;
        }
        let weight = if ordinal == 0 || ordinal == count - 1 {
            half
        } else {
            full
        };
        work.emit(
            weights,
            UsedGradientRecord {
                source_index: record.source_index,
                position_index: record.position_index,
                position: weight,
            },
        )?;
        ordinal += 1;
    }
    Ok(())
}

fn weighted<C>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    normalized: &[UsedGradientRecord<Normalized>],
    weights: &mut Vec<UsedGradientRecord>,
) -> Result<(), PaintError> {
    let mut previous: Option<UsedGradientRecord<Normalized>> = None;
    let mut penultimate = None;
    let mut strict = false;
    let mut incoming = 0.0;
    for index in 0..normalized.len() {
        let record = work.record(normalized, index)?;
        strict |= record.position.strict;
        if matches!(
            work.source(source, record.source_index)?,
            GradientStopItem::Hint { .. }
        ) {
            continue;
        }
        if let Some(left) = previous {
            let distance = work.numeric(|| record.position.value - left.position.value)?;
            if !work.numeric(|| distance.is_finite())? || work.numeric(|| distance < 0.0)? {
                return Err(loss(left, record));
            }
            if strict && work.numeric(|| distance == 0.0)? {
                return Err(loss(left, record));
            }
            let adjacent = work.numeric(|| incoming + distance)?;
            let position = work.numeric(|| adjacent * 0.5)?;
            if work.numeric(|| adjacent > 0.0)? && work.numeric(|| position == 0.0)? {
                return Err(loss(left, record));
            }
            work.emit(
                weights,
                UsedGradientRecord {
                    source_index: left.source_index,
                    position_index: left.position_index,
                    position,
                },
            )?;
            incoming = distance;
            penultimate = Some(left);
        }
        previous = Some(record);
        strict = false;
    }
    let last = previous.expect("validated nonempty color list");
    let position = work.numeric(|| incoming * 0.5)?;
    if work.numeric(|| incoming > 0.0)? && work.numeric(|| position == 0.0)? {
        return Err(loss(
            penultimate.expect("at least two expanded colors"),
            last,
        ));
    }
    work.emit(
        weights,
        UsedGradientRecord {
            source_index: last.source_index,
            position_index: last.position_index,
            position,
        },
    )
}

/// Positive distances retain an exponent, so neither raw span overflow nor tiny
/// raw distributed positions are prerequisites of the color measure.
#[derive(Clone, Copy)]
struct Distance {
    mantissa: f64,
    exponent: i32,
}
impl Distance {
    fn positive(work: &mut StopWork<'_, '_, '_>, value: f64) -> Result<Self, PaintError> {
        let bits = work.numeric(|| value.to_bits())?;
        let raw = work.numeric(|| (bits >> 52) & 0x7ff)?;
        if work.numeric(|| raw == 0)? {
            let scaled = work.numeric(|| value * 4_503_599_627_370_496.0)?;
            let mut result = Self::positive(work, scaled)?;
            result.exponent = work.numeric(|| result.exponent - 52)?;
            return Ok(result);
        }
        let fraction = work.numeric(|| bits & 0x000f_ffff_ffff_ffff)?;
        let normal_bits = work.numeric(|| fraction | 0x3ff0_0000_0000_0000)?;
        let mantissa = work.numeric(|| f64::from_bits(normal_bits))?;
        let raw = work.numeric(|| raw as i32)?;
        let exponent = work.numeric(|| raw - 1023)?;
        Ok(Self { mantissa, exponent })
    }
    fn between(work: &mut StopWork<'_, '_, '_>, a: f64, b: f64) -> Result<Self, PaintError> {
        let difference = work.numeric(|| b - a)?;
        if work.numeric(|| difference.is_finite())? {
            Self::positive(work, difference)
        } else {
            let a_half = work.numeric(|| a * 0.5)?;
            let b_half = work.numeric(|| b * 0.5)?;
            let half = work.numeric(|| b_half - a_half)?;
            let mut result = Self::positive(work, half)?;
            result.exponent = work.numeric(|| result.exponent + 1)?;
            Ok(result)
        }
    }
    fn ratio(self, work: &mut StopWork<'_, '_, '_>, denominator: Self) -> Result<f64, PaintError> {
        let quotient = work.numeric(|| self.mantissa / denominator.mantissa)?;
        let exponent = work.numeric(|| self.exponent - denominator.exponent)?;
        if work.numeric(|| exponent < -1075)? {
            return Ok(0.0);
        }
        // Split the final multiplication only in the subnormal output range.
        // Both factors before that final rounding are normal.
        if work.numeric(|| exponent < -1022)? {
            let first_exponent = work.numeric(|| exponent + 1022)?;
            let factor = power(work, first_exponent)?;
            let partial = work.numeric(|| quotient * factor)?;
            work.numeric(|| partial * f64::MIN_POSITIVE)
        } else {
            let factor = power(work, exponent)?;
            work.numeric(|| quotient * factor)
        }
    }
}
fn power(work: &mut StopWork<'_, '_, '_>, exponent: i32) -> Result<f64, PaintError> {
    let biased = work.numeric(|| exponent + 1023)?;
    let biased = work.numeric(|| biased as u64)?;
    let bits = work.numeric(|| biased << 52)?;
    work.numeric(|| f64::from_bits(bits))
}
struct Range {
    a: f64,
    b: f64,
    span: Distance,
}
impl Range {
    fn new(work: &mut StopWork<'_, '_, '_>, a: f64, b: f64) -> Result<Self, PaintError> {
        Ok(Self {
            a,
            b,
            span: Distance::between(work, a, b)?,
        })
    }
    fn normalize(&self, work: &mut StopWork<'_, '_, '_>, value: f64) -> Result<f64, PaintError> {
        if work.numeric(|| value == self.a)? {
            return Ok(0.0);
        }
        if work.numeric(|| value == self.b)? {
            return Ok(1.0);
        }
        Distance::between(work, self.a, value)?.ratio(work, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GradientStopPositions, PaintChange, PaintLimits, ProfileMode, RadialGradientSize,
        resolve_radial_gradient,
    };
    use surgeist_layout::{Point, Size};

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }
    fn geometry() -> RadialGradientGeometry {
        resolve_radial_gradient(
            Size::new(40.0, 20.0),
            Some(Point::new(0.0, 0.0)),
            RadialGradientSize::EllipseRadii(Size::new(
                GradientPosition {
                    fraction: 0.0,
                    length: 20.0,
                },
                GradientPosition {
                    fraction: 0.0,
                    length: 0.0,
                },
            )),
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap()
    }
    fn stop(value: f64, fraction: bool) -> GradientStopItem<()> {
        GradientStopItem::Stop {
            payload: (),
            positions: GradientStopPositions::One(if fraction {
                GradientPosition {
                    fraction: value,
                    length: 0.0,
                }
            } else {
                GradientPosition {
                    fraction: 0.0,
                    length: value,
                }
            }),
        }
    }
    fn normal_decode(limits: PaintLimits) -> (Result<Distance, PaintError>, PaintReport) {
        let mut meter = options(limits)
            .operation(OperationKind::ZeroHeightGradient)
            .unwrap();
        let scopes = setup_scopes(&mut meter).unwrap();
        let result = meter
            .with_phase(scopes[3], |p| {
                Distance::positive(&mut StopWork::new(p, scopes[3]), 1.5)
            })
            .unwrap();
        (result, meter.finish().unwrap().unwrap())
    }

    #[test]
    fn normal_distance_decode_admits_each_action_and_preserves_refusal_prefix() {
        let (value, report) = normal_decode(PaintLimits::unlimited());
        let value = value.unwrap();
        assert_eq!((value.mantissa, value.exponent), (1.5, 0));
        // to_bits, exponent-field extraction, normal/subnormal inspection,
        // fraction mask, normal-bit composition, from_bits, exponent conversion,
        // and bias subtraction: eight owning scalar actions, with no record walk.
        assert_eq!(report.work(WorkKind::GradientCoordinate), 8);
        assert_eq!(report.work(WorkKind::GradientStopVisit), 0);
        let (value, report) = normal_decode(
            PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 7),
        );
        let Err(PaintError::Budget(refusal)) = value else {
            panic!("eighth scalar must refuse")
        };
        assert_eq!(refusal.current, 7);
        assert_eq!(
            refusal.change,
            PaintChange::Work {
                kind: WorkKind::GradientCoordinate,
                amount: 1,
            }
        );
        assert_eq!(report.work(WorkKind::GradientCoordinate), 7);
        for &resource in ResourceKind::ALL {
            assert_eq!(report.live(resource), 0);
        }
        assert_eq!(
            normal_decode(PaintLimits::unlimited()).0.unwrap().exponent,
            0
        );
    }

    #[test]
    fn normalized_omissions_succeed_without_raw_tiny_or_overflowing_coordinates() {
        for (fraction, a, b) in [
            (true, 0.0, f64::from_bits(1)),
            (false, 0.0, f64::from_bits(1)),
            (false, -f64::MAX, f64::MAX),
        ] {
            let source = [
                stop(a, fraction),
                GradientStopItem::Stop {
                    payload: (),
                    positions: GradientStopPositions::Omitted,
                },
                GradientStopItem::Stop {
                    payload: (),
                    positions: GradientStopPositions::Omitted,
                },
                stop(b, fraction),
            ];
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(),
                &source,
                GradientRepeat::Repeat,
                options(PaintLimits::unlimited()),
            );
            let colors = outcome.result.unwrap();
            for (actual, expected) in
                colors
                    .colors()
                    .map(|c| c.weight)
                    .zip([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0])
            {
                assert!((actual - expected).abs() <= 2.0 * f64::EPSILON);
            }
            assert_eq!(outcome.report.unwrap().live(ResourceKind::ScratchBytes), 0);
        }
    }

    #[test]
    fn actual_result_capacity_and_late_publication_refusals_release_every_hold() {
        let source = [stop(0.0, false), stop(10.0, false), stop(40.0, false)];
        let baseline = resolve_zero_height_gradient_colors(
            &geometry(),
            &source,
            GradientRepeat::Repeat,
            options(PaintLimits::unlimited()),
        );
        let report = baseline.report.unwrap();
        let result = baseline.result.unwrap();
        let Measure::Average(records) = &result.measure else {
            panic!("three-color measure")
        };
        let expected = bytes::<UsedGradientRecord>(records.capacity()).unwrap()
            + bytes::<ZeroHeightGradientColors<'_, ()>>(1).unwrap();
        assert_eq!(report.live(ResourceKind::RetainedResultBytes), expected);
        assert!(report.peak(ResourceKind::ScratchBytes) > 0);
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
        let work = report.work(WorkKind::OutputRecord);
        for limits in [
            PaintLimits::unlimited()
                .with_resource_limit(ResourceKind::RetainedResultBytes, expected - 1),
            PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, work - 1),
        ] {
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(),
                &source,
                GradientRepeat::Repeat,
                options(limits),
            );
            assert!(matches!(outcome.result, Err(PaintError::Budget(_))));
            let report = outcome.report.unwrap();
            for &resource in ResourceKind::ALL {
                assert_eq!(report.live(resource), 0);
            }
        }
        let retry = resolve_zero_height_gradient_colors(
            &geometry(),
            &source,
            GradientRepeat::Repeat,
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap();
        assert_eq!(
            retry.colors().map(|c| c.weight).collect::<Vec<_>>(),
            [0.125, 0.5, 0.375]
        );
    }
}
