//! Accounted consumer of authentic native Paint geometry and used positions.

use crate::{
    Color, Point, RenderAccountingContext, RenderResource, RepeatingGradient,
    RepeatingGradientStop, Result,
    accounting::{Account, AccountedVec, RenderAttemptKind, RenderPhase, RenderWork},
    paint::RepeatingGradientGeometry,
};
use std::mem::size_of;
use surgeist_paint::{
    GradientStops, LinearGradientLine, RadialGradientCase, RadialGradientGeometry, UsedGradientItem,
};

/// Native composition or representation refusal. Valid Paint values may exceed
/// the supported repeating consumer's geometry/representation capability.
#[derive(Debug, PartialEq)]
pub enum PaintGradientError {
    UnsupportedZeroLengthLinear,
    UnsupportedRadialCase {
        case: RadialGradientCase,
    },
    BasisMismatch {
        expected: f64,
        actual: f64,
    },
    UnsupportedHint {
        source_index: usize,
    },
    UnrepresentablePosition {
        source_index: usize,
        position_index: usize,
    },
    UnrepresentableSpan {
        first_source_index: usize,
        last_source_index: usize,
    },
    Allocation {
        resource: RenderResource,
    },
    CapacityOverflow {
        resource: RenderResource,
    },
}

/// Prepares an owned repeating field from a positive native Paint line and its
/// actual used-stop result. Calling this function explicitly selects repetition.
/// Hints and zero length refuse; no authored CSS or color evaluation is performed.
pub fn prepare_repeating_linear_gradient(
    line: &LinearGradientLine,
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<RepeatingGradient> {
    prepare(stops, context, |account| {
        account.work(RenderWork::PaintVisits, 1)?;
        let length = scalar(account, || line.length())?;
        if scalar(account, || length == 0.0)? {
            return Err(PaintGradientError::UnsupportedZeroLengthLinear.into());
        }
        // Each copied physical point contains two inspected scalar coordinates.
        account.work(RenderWork::CoordinateVisits, 2)?;
        let start = line.start();
        let start = Point::new(start.x, start.y);
        account.work(RenderWork::CoordinateVisits, 2)?;
        let end = line.end();
        let end = Point::new(end.x, end.y);
        Ok((RepeatingGradientGeometry::Linear { start, end }, length))
    })
}

/// Prepares an owned repeating elliptical field from positive native Paint
/// geometry and its used stops. Every radial zero case remains unsupported;
/// this operation never invents a degenerate used-stop basis or replacement radii.
pub fn prepare_repeating_radial_gradient(
    geometry: &RadialGradientGeometry,
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<RepeatingGradient> {
    prepare(stops, context, |account| {
        account.work(RenderWork::PaintVisits, 1)?;
        let radii = match geometry.case() {
            RadialGradientCase::Ellipse { radii } => radii,
            case => return Err(PaintGradientError::UnsupportedRadialCase { case }.into()),
        };
        account.work(RenderWork::CoordinateVisits, 2)?;
        let center = geometry.center();
        let center = Point::new(center.x, center.y);
        let radius_x = scalar(account, || radii.width)?;
        let radius_y = scalar(account, || radii.height)?;
        Ok((
            RepeatingGradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            },
            radius_x,
        ))
    })
}

fn prepare(
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
    geometry: impl FnOnce(&Account) -> Result<(RepeatingGradientGeometry, f64)>,
) -> Result<RepeatingGradient> {
    context.run(
        RenderAttemptKind::PaintGradient,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                let coordinates = account.at(RenderPhase::Coordinates);
                let (geometry, length) = account.phase(RenderPhase::Coordinates, || {
                    // Geometry refusal precedes any stop-result inspection.
                    let (geometry, length) = geometry(&coordinates)?;
                    coordinates.work(RenderWork::PaintVisits, 1)?;
                    let actual = scalar(&coordinates, || stops.line_length())?;
                    if !scalar(&coordinates, || actual == length)? {
                        return Err(PaintGradientError::BasisMismatch {
                            expected: length,
                            actual,
                        }
                        .into());
                    }
                    Ok((geometry, length))
                })?;
                preflight(stops, account)?;
                let (converted, first, last) = account.phase(RenderPhase::Coordinates, || {
                    convert(stops, length, &coordinates)
                })?;
                let output = account.at(RenderPhase::Output);
                account.phase(RenderPhase::Output, || {
                    let fixed = output.acquire(
                        RenderResource::RetainedCpuBytes,
                        size_of::<RepeatingGradient>(),
                    )?;
                    let guarded = RepeatingGradient::from_accounted_paint(
                        geometry, converted, fixed, first, last, &output,
                    )?;
                    // All validation/admission has succeeded. Both actual backing
                    // guards transfer through the enclosing standalone attempt.
                    let (gradient, backing, import) = guarded.into_parts();
                    for hold in backing.into_iter().flatten() {
                        hold.retain();
                    }
                    drop(import);
                    Ok(gradient)
                })
            })
        },
    )
}

// PaintVisits: one producer geometry, one used-stop owner, and one owned native
// gradient inspection in the canonical Output constructor. Stop visits: every
// actual next() in preflight/conversion, plus the canonical validation walk.
// Each iterator length inspection is constant control metadata, not a hidden walk.
fn preflight(stops: &GradientStops<'_, Color>, account: &Account) -> Result<()> {
    let mut items = stops.items();
    while items.len() != 0 {
        account.work(RenderWork::GradientStopVisits, 1)?;
        if let UsedGradientItem::Hint { source_index, .. } =
            items.next().expect("nonempty exact-size view")
        {
            return Err(PaintGradientError::UnsupportedHint { source_index }.into());
        }
    }
    Ok(())
}

// One CoordinateVisits unit per actual scalar inspection/calculation/conversion
// below, admitted before action. A scalar domain check is one inspection; the
// point-copy batches above each inspect exactly two already-finite coordinates.
// Canonical validation additionally charges its actual scalar checks. No source
// bytes, fake scratch, hidden constructor walk or independent counter is owned.
// Successful conversion uses 3 scalar units per zero input, 4 per nonzero input,
// and 2 per adjacent pair. Canonical validation uses 6*n+5 scalar units (four
// geometry domains, five per stop, n-1 ordering checks, span subtraction/domain).
// Geometry plus basis inspection uses 8 units for linear and 6 for radial.
fn scalar<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

fn convert(
    stops: &GradientStops<'_, Color>,
    length: f64,
    account: &Account,
) -> Result<(AccountedVec<RepeatingGradientStop>, usize, usize)> {
    let mut items = stops.items();
    let mut output = account.retained(items.len())?;
    let mut first = None;
    let mut last = None;
    let mut previous = None;
    while items.len() != 0 {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let UsedGradientItem::Stop {
            source_index,
            position_index,
            payload,
            position,
        } = items.next().expect("nonempty exact-size view")
        else {
            unreachable!("immutable source passed hint preflight");
        };
        let failure = || PaintGradientError::UnrepresentablePosition {
            source_index,
            position_index,
        };
        let field = scalar(account, || position / length)?;
        if !scalar(account, || field.is_finite())? {
            return Err(failure().into());
        }
        if scalar(account, || position != 0.0)? && scalar(account, || field == 0.0)? {
            return Err(failure().into());
        }
        if let Some((previous_position, previous_field)) = previous {
            let same = scalar(account, || position == previous_position)?;
            let preserves_order = scalar(account, || {
                if same {
                    field == previous_field
                } else {
                    field > previous_field
                }
            })?;
            if !preserves_order {
                return Err(failure().into());
            }
        }
        // Ordinary division rounding is allowed. Equality and strict order of
        // the actual adjacent inputs must survive that rounding, including Two
        // ordinals; no exact backmap or physical-span restriction is imposed.
        output.push_reserved(RepeatingGradientStop::from_qualified_paint(field, *payload));
        previous = Some((position, field));
        first.get_or_insert(source_index);
        last = Some(source_index);
    }
    Ok((
        output,
        first.expect("Paint list has a stop"),
        last.expect("Paint list has a stop"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderAccountingOptions, RenderLimits, RenderProfile};
    use std::cell::Cell;
    use surgeist_paint::{
        GradientPosition, GradientStopItem, GradientStopPositions, PaintLimits, PaintOptions,
        ProfileMode, RadialGradientSize, resolve_gradient_stops, resolve_radial_gradient,
    };

    fn options() -> PaintOptions {
        PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)
    }

    fn context(limits: RenderLimits) -> RenderAccountingContext {
        RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_limits(limits)
                .with_profile(RenderProfile::Summary { timing: false }),
        )
    }

    fn source(a: f64, b: f64) -> [GradientStopItem<Color>; 2] {
        [a, b].map(|length| GradientStopItem::Stop {
            payload: Color::BLACK,
            positions: GradientStopPositions::One(GradientPosition {
                length,
                fraction: 0.0,
            }),
        })
    }

    fn geometry() -> RadialGradientGeometry {
        resolve_radial_gradient(
            Default::default(),
            None,
            RadialGradientSize::CircleRadius(1.0),
            options(),
        )
        .result
        .unwrap()
    }

    #[test]
    fn scalar_counts_include_zero_short_circuit_and_all_three_stop_walks() {
        // Source geometry/basis: 6. Canonical domain/order/span checks: 17.
        // Conversion: 9 for [0,1], 10 for [1,2]. This derives from scalar
        // actions, independently of the implementation's counter helpers.
        for (a, b, coordinates) in [(0.0, 1.0, 32), (1.0, 2.0, 33)] {
            let source = source(a, b);
            let stops = resolve_gradient_stops(1.0, &source, options())
                .result
                .unwrap();
            let mut context = context(RenderLimits::new());
            let result =
                prepare_repeating_radial_gradient(&geometry(), &stops, &mut context).unwrap();
            assert_eq!(result.stops()[0].position(), a);
            assert_eq!(result.stops()[1].position(), b);
            let report = context.last_attempt().unwrap();
            assert_eq!(
                report.usage().work(RenderWork::CoordinateVisits),
                coordinates
            );
            assert_eq!(report.usage().work(RenderWork::PaintVisits), 3);
            assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 6);
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Coordinates)
                    .work(RenderWork::GradientStopVisits),
                2
            );
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Output)
                    .work(RenderWork::GradientStopVisits),
                2
            );
            assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        }
    }

    #[test]
    fn canonical_owner_keeps_actual_capacity_and_fixed_custody_until_transfer() {
        let source = source(1.0, 2.0);
        let stops = resolve_gradient_stops(1.0, &source, options())
            .result
            .unwrap();
        let mut context = context(RenderLimits::new());
        let (gradient, actual_bytes) = context
            .run(
                RenderAttemptKind::PaintGradient,
                RenderPhase::Output,
                |account| {
                    let (converted, first, last) = convert(&stops, 1.0, account)?;
                    let actual_bytes = converted.capacity() * size_of::<RepeatingGradientStop>()
                        + size_of::<RepeatingGradient>();
                    let fixed = account.acquire(
                        RenderResource::RetainedCpuBytes,
                        size_of::<RepeatingGradient>(),
                    )?;
                    let guarded = RepeatingGradient::from_accounted_paint(
                        RepeatingGradientGeometry::Radial {
                            center: Point::new(0.0, 0.0),
                            radius_x: 1.0,
                            radius_y: 1.0,
                        },
                        converted,
                        fixed,
                        first,
                        last,
                        account,
                    )?;
                    let (gradient, backing, import) = guarded.into_parts();
                    for hold in backing.into_iter().flatten() {
                        hold.retain();
                    }
                    drop(import);
                    Ok((gradient, actual_bytes as u64))
                },
            )
            .unwrap();
        assert_eq!(gradient.stops().len(), 2);
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), actual_bytes);
        assert_eq!(
            usage.work(RenderWork::PreparedTransferredBytes),
            actual_bytes
        );
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn canonical_refusal_disposes_both_armed_owners() {
        let source = source(1.0, 2.0);
        let stops = resolve_gradient_stops(1.0, &source, options())
            .result
            .unwrap();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::PaintVisits, 0)
            .unwrap();
        let mut context = context(limits);
        let actual_bytes = Cell::new(0);
        let result = context.run(
            RenderAttemptKind::PaintGradient,
            RenderPhase::Output,
            |account| {
                let (converted, first, last) = convert(&stops, 1.0, account)?;
                actual_bytes.set(
                    (converted.capacity() * size_of::<RepeatingGradientStop>()
                        + size_of::<RepeatingGradient>()) as u64,
                );
                let fixed = account.acquire(
                    RenderResource::RetainedCpuBytes,
                    size_of::<RepeatingGradient>(),
                )?;
                RepeatingGradient::from_accounted_paint(
                    RepeatingGradientGeometry::Radial {
                        center: Point::new(0.0, 0.0),
                        radius_x: 1.0,
                        radius_y: 1.0,
                    },
                    converted,
                    fixed,
                    first,
                    last,
                    account,
                )
                .map(drop)
            },
        );
        assert!(result.unwrap_err().accounting_diagnostic().is_some());
        let report = context.last_attempt().unwrap();
        // The configured cap belongs to Attempt; Output owns the attempted
        // charge, but Ledger identifies the ancestor that enforces its limit.
        assert_eq!(report.refusal().unwrap().phase(), RenderPhase::Attempt);
        assert_eq!(
            report.refusal().unwrap().quantity(),
            crate::RenderQuantity::Work(RenderWork::PaintVisits)
        );
        assert_eq!(
            report
                .phase_usage(RenderPhase::Output)
                .work(RenderWork::PaintVisits),
            0
        );
        assert_eq!(
            report.usage().peak(RenderResource::RetainedCpuBytes),
            actual_bytes.get()
        );
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
        assert!(report.phase_count(RenderPhase::Cleanup) > 0);
    }

    #[test]
    fn conversion_refusal_preserves_accepted_prefix_and_releases_backing() {
        let source = source(1.0, 2.0);
        let stops = resolve_gradient_stops(1.0, &source, options())
            .result
            .unwrap();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CoordinateVisits, 4)
            .unwrap();
        let mut context = context(limits);
        let result = context.run(
            RenderAttemptKind::PaintGradient,
            RenderPhase::Coordinates,
            |account| convert(&stops, 1.0, account).map(drop),
        );
        let error = result.unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(error.paint_gradient_diagnostic().is_none());
        let report = context.last_attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 4);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 2);
        assert!(report.usage().peak(RenderResource::RetainedCpuBytes) > 0);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    }

    #[test]
    fn allocation_and_capacity_errors_keep_the_paint_domain() {
        let mut context = context(RenderLimits::new());
        context
            .run(
                RenderAttemptKind::PaintGradient,
                RenderPhase::Coordinates,
                |account| {
                    let allocation = account.allocation_error(RenderResource::RetainedCpuBytes);
                    assert_eq!(
                        allocation.paint_gradient_diagnostic(),
                        Some(&PaintGradientError::Allocation {
                            resource: RenderResource::RetainedCpuBytes,
                        })
                    );
                    let error = match account.retained::<u64>(usize::MAX) {
                        Ok(_) => {
                            panic!("unrepresentable byte capacity must refuse before allocation")
                        }
                        Err(error) => error,
                    };
                    assert_eq!(
                        error.paint_gradient_diagnostic(),
                        Some(&PaintGradientError::CapacityOverflow {
                            resource: RenderResource::RetainedCpuBytes,
                        })
                    );
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            0
        );
    }
}
