//! Accounted consumer of authentic native Paint geometry and used positions.

use crate::{
    Color, Gradient, GradientHint, GradientStop, Point, RenderAccountingContext, RenderResource,
    RepeatingGradient, Result,
    accounting::{
        Account, AccountedVec, BackedValue, Hold, RenderAttemptKind, RenderPhase, RenderWork,
    },
    color_reduction::{PremultipliedSum, premultiplied},
    paint::GradientGeometry,
};
use std::mem::size_of;
use surgeist_paint::{
    GradientStops, LinearGradientLine, RadialGradientCase, RadialGradientGeometry,
    UsedGradientItem, ZeroHeightGradientColors,
};

/// Native composition or representation refusal. Valid Paint values may exceed
/// the supported native consumer's geometry/representation capability.
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

/// Prepares an owned ordinary field from a positive native Paint line and its
/// actual used stops and transition hints. Zero length refuses; positions are neither clamped
/// nor repeated, and no authored CSS or color evaluation is performed.
pub fn prepare_linear_gradient(
    line: &LinearGradientLine,
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<Gradient> {
    prepare(
        stops,
        context,
        |account| linear_geometry(line, account),
        |geometry, (stops, _, _), fixed, account| {
            Gradient::from_accounted_paint(geometry, stops, fixed, account)
        },
    )
}

/// Prepares an owned ordinary field from native Paint elliptical or ZeroWidth
/// geometry and its used stops and hints. ZeroWidth uses physical distances at
/// basis zero. ZeroCircle and ZeroHeight refuse without replacement geometry.
pub fn prepare_radial_gradient(
    geometry: &RadialGradientGeometry,
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<Gradient> {
    prepare(
        stops,
        context,
        |account| radial_geometry(geometry, account),
        |geometry, (stops, _, _), fixed, account| {
            Gradient::from_accounted_paint(geometry, stops, fixed, account)
        },
    )
}

/// Prepares an owned repeating field from a positive native Paint line and its
/// actual used-stop result. Calling this function explicitly selects repetition.
/// Zero length refuses; no authored CSS or color evaluation is performed.
pub fn prepare_repeating_linear_gradient(
    line: &LinearGradientLine,
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<RepeatingGradient> {
    prepare(
        stops,
        context,
        |account| linear_geometry(line, account),
        |geometry, (stops, indices, positions), fixed, account| {
            RepeatingGradient::from_accounted_paint(
                geometry, stops, fixed, indices, positions, account,
            )
        },
    )
}

/// Prepares an owned repeating elliptical or ZeroWidth field from native Paint
/// geometry and its used stops. ZeroWidth uses physical distances at basis zero;
/// ZeroCircle and ZeroHeight remain unsupported without replacement geometry.
pub fn prepare_repeating_radial_gradient(
    geometry: &RadialGradientGeometry,
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<RepeatingGradient> {
    prepare(
        stops,
        context,
        |account| radial_geometry(geometry, account),
        |geometry, (stops, indices, positions), fixed, account| {
            RepeatingGradient::from_accounted_paint(
                geometry, stops, fixed, indices, positions, account,
            )
        },
    )
}

/// Reduces an authenticated native Paint zero-height color measure to an owned
/// numeric-sRGBA color. The producer has already selected Last or Average and
/// proved its source associations and weights; no field or hint program is built.
pub fn prepare_zero_height_gradient_color(
    colors: &ZeroHeightGradientColors<'_, Color>,
    context: &mut RenderAccountingContext,
) -> Result<Color> {
    context.run(
        RenderAttemptKind::PaintGradient,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                let coordinates = account.at(RenderPhase::Coordinates);
                let color = account.phase(RenderPhase::Coordinates, || {
                    coordinates.work(RenderWork::PaintVisits, 1)?;
                    let mut colors = colors.colors();
                    if colors.len() == 1 {
                        coordinates.work(RenderWork::GradientStopVisits, 1)?;
                        // Last's sole authenticated occurrence is a direct Color
                        // copy, including any straight RGB with zero alpha.
                        return Ok(*colors.next().expect("authenticated Last color").payload);
                    }
                    let mut total = PremultipliedSum::new();
                    while colors.len() != 0 {
                        coordinates.work(RenderWork::GradientStopVisits, 1)?;
                        let item = colors.next().expect("nonempty exact-size view");
                        let weight = scalar(&coordinates, || item.weight)?;
                        let color = premultiplied(*item.payload, &coordinates)?;
                        total.add_weighted(color, weight, &coordinates)?;
                    }
                    total.finish(&coordinates)
                })?;
                account.phase(RenderPhase::Output, || {
                    let output = account.at(RenderPhase::Output);
                    let fixed =
                        output.acquire(RenderResource::RetainedCpuBytes, size_of::<Color>())?;
                    output.work(RenderWork::PaintVisits, 1)?;
                    // This fixed Color is the complete owned result. There is no
                    // output vector, copied stop list, or borrowed storage to charge.
                    fixed.into_preparation()?.retain();
                    Ok(color)
                })
            })
        },
    )
}

/// Keep the required basis and its position projection in one closed state.
/// In particular, a zero-width basis has no divisor or unused ellipse geometry.
#[derive(Clone, Copy)]
enum AdapterGeometry {
    UnitBasis {
        geometry: GradientGeometry,
        basis: f64,
    },
    HorizontalDistance {
        origin_x: f64,
    },
}

impl AdapterGeometry {
    const fn expected_basis(self) -> f64 {
        match self {
            Self::UnitBasis { basis, .. } => basis,
            Self::HorizontalDistance { .. } => 0.0,
        }
    }

    const fn into_geometry(self) -> GradientGeometry {
        match self {
            Self::UnitBasis { geometry, .. } => geometry,
            Self::HorizontalDistance { origin_x } => {
                GradientGeometry::HorizontalDistance { origin_x }
            }
        }
    }
}

fn linear_geometry(line: &LinearGradientLine, account: &Account) -> Result<AdapterGeometry> {
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
    Ok(AdapterGeometry::UnitBasis {
        geometry: GradientGeometry::Linear { start, end },
        basis: length,
    })
}

fn radial_geometry(
    geometry: &RadialGradientGeometry,
    account: &Account,
) -> Result<AdapterGeometry> {
    account.work(RenderWork::PaintVisits, 1)?;
    let radii = match geometry.case() {
        RadialGradientCase::ZeroWidth => {
            let origin_x = scalar(account, || geometry.center().x)?;
            return Ok(AdapterGeometry::HorizontalDistance { origin_x });
        }
        RadialGradientCase::Ellipse { radii } => radii,
        case => return Err(PaintGradientError::UnsupportedRadialCase { case }.into()),
    };
    account.work(RenderWork::CoordinateVisits, 2)?;
    let center = geometry.center();
    let center = Point::new(center.x, center.y);
    let radius_x = scalar(account, || radii.width)?;
    let radius_y = scalar(account, || radii.height)?;
    Ok(AdapterGeometry::UnitBasis {
        geometry: GradientGeometry::Radial {
            center,
            radius_x,
            radius_y,
        },
        basis: radius_x,
    })
}

type ConvertedStops = (AccountedVec<GradientStop>, (usize, usize), (f64, f64));

fn prepare<T>(
    stops: &GradientStops<'_, Color>,
    context: &mut RenderAccountingContext,
    geometry: impl FnOnce(&Account) -> Result<AdapterGeometry>,
    construct: impl FnOnce(GradientGeometry, ConvertedStops, Hold, &Account) -> Result<BackedValue<T>>,
) -> Result<T> {
    context.run(
        RenderAttemptKind::PaintGradient,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                let coordinates = account.at(RenderPhase::Coordinates);
                let geometry = account.phase(RenderPhase::Coordinates, || {
                    // Geometry refusal precedes any stop-result inspection.
                    let geometry = geometry(&coordinates)?;
                    let expected = geometry.expected_basis();
                    coordinates.work(RenderWork::PaintVisits, 1)?;
                    let actual = scalar(&coordinates, || stops.line_length())?;
                    if !scalar(&coordinates, || actual == expected)? {
                        return Err(PaintGradientError::BasisMismatch { expected, actual }.into());
                    }
                    Ok(geometry)
                })?;
                let color_count = preflight(stops, account)?;
                let converted = account.phase(RenderPhase::Coordinates, || {
                    convert(stops, &geometry, color_count, &coordinates)
                })?;
                let output = account.at(RenderPhase::Output);
                account.phase(RenderPhase::Output, || {
                    let fixed = output.acquire(RenderResource::RetainedCpuBytes, size_of::<T>())?;
                    let guarded = construct(geometry.into_geometry(), converted, fixed, &output)?;
                    // Both actual allocation guards transfer through the enclosing
                    // standalone attempt only after all qualification succeeds.
                    guarded.require_preparation()?;
                    let (gradient, backing, import) = guarded.into_parts();
                    for hold in backing.into_iter().flatten() {
                        hold.into_preparation()?.retain();
                    }
                    drop(import);
                    Ok(gradient)
                })
            })
        },
    )
}

// PaintVisits: producer geometry, used-stop owner, and owned native gradient.
// Stop visits: each actual next() in preflight and conversion. Qualified output
// construction consumes the completed proof/backing without another record walk.
// Iterator length inspections are constant control metadata, not hidden walks.
fn preflight(stops: &GradientStops<'_, Color>, account: &Account) -> Result<usize> {
    let mut items = stops.items();
    let mut color_count = 0usize;
    while items.len() != 0 {
        account.work(RenderWork::GradientStopVisits, 1)?;
        if matches!(
            items.next().expect("nonempty exact-size view"),
            UsedGradientItem::Stop { .. }
        ) {
            color_count = color_count
                .checked_add(1)
                .ok_or_else(|| account.capacity_error(RenderResource::RetainedCpuBytes))?;
        }
    }
    // Native Paint already proves nonempty, nonadjacent, nonterminal hints.
    // Only color records require output capacity; hints live in those records.
    Ok(color_count)
}

// One CoordinateVisits unit per actual scalar inspection/calculation/conversion,
// admitted before action. Conversion uses 4 units per zero source item, 5 per
// nonzero source item, and 2 per adjacent pair (including hints) for positive
// unit-basis fields. Physical identity only captures each source position once.
// Geometry/basis uses 8 units for linear, 6 for positive radial, and 3 for
// horizontal distance. Repeating output alone subtracts/checks its span (2).
// No color revalidation or duplicate canonical quotient/ordering walk is needed:
// immutable Color and Paint geometry already own those intrinsic domains.
fn scalar<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

fn convert(
    stops: &GradientStops<'_, Color>,
    geometry: &AdapterGeometry,
    color_count: usize,
    account: &Account,
) -> Result<ConvertedStops> {
    let mut items = stops.items();
    let mut output = account.retained(color_count)?;
    let mut first = None;
    let mut last = None;
    let mut previous = None;
    // Pending color/hint fields are assembled once when the next color arrives.
    // No already-owned output record is patched or traversed again.
    let mut pending: Option<(f64, Color, Option<GradientHint>)> = None;
    while items.len() != 0 {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let item = items.next().expect("nonempty exact-size view");
        let (source_index, position_index) = match item {
            UsedGradientItem::Stop {
                source_index,
                position_index,
                ..
            } => (source_index, position_index),
            UsedGradientItem::Hint { source_index, .. } => (source_index, 0),
        };
        let failure = || PaintGradientError::UnrepresentablePosition {
            source_index,
            position_index,
        };
        let position = scalar(account, || match item {
            UsedGradientItem::Stop { position, .. } | UsedGradientItem::Hint { position, .. } => {
                position
            }
        })?;
        let field = match geometry {
            AdapterGeometry::UnitBasis { basis, .. } => {
                let field = scalar(account, || position / basis)?;
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
                previous = Some((position, field));
                field
            }
            AdapterGeometry::HorizontalDistance { .. } => position,
        };
        // Qualified division preserves source equality/order; physical identity
        // preserves the authentic proof exactly without redundant numeric checks.
        // Paint's item association proves inclusive hints and following colors.
        match item {
            UsedGradientItem::Stop { payload, .. } => {
                if let Some((position, color, hint_after)) =
                    pending.replace((field, *payload, None))
                {
                    output
                        .append_slot()?
                        .write(GradientStop::from_qualified_paint(
                            position, color, hint_after,
                        ));
                }
                first.get_or_insert((source_index, field));
                last = Some((source_index, field));
            }
            UsedGradientItem::Hint { .. } => {
                pending.as_mut().expect("Paint hint follows a color").2 =
                    Some(GradientHint::from_qualified_paint(field));
            }
        }
    }
    let (position, color, hint_after) = pending.expect("Paint list has a color");
    output
        .append_slot()?
        .write(GradientStop::from_qualified_paint(
            position, color, hint_after,
        ));
    let first = first.expect("Paint list has a stop");
    let last = last.expect("Paint list has a stop");
    Ok((output, (first.0, last.0), (first.1, last.1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderAccountingOptions, RenderLimits, RenderProfile, paint::GradientKind};
    use std::cell::Cell;
    use surgeist_paint::{
        GradientPosition, GradientStopItem, GradientStopPositions, PaintLimits, PaintOptions,
        ProfileMode, RadialGradientSize, resolve_gradient_stops, resolve_radial_gradient,
    };

    fn unit_basis(basis: f64) -> AdapterGeometry {
        AdapterGeometry::UnitBasis {
            geometry: GradientGeometry::Radial {
                center: Point::new(0.0, 0.0),
                radius_x: basis,
                radius_y: 1.0,
            },
            basis,
        }
    }

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

    fn zero_width_geometry(origin_x: f64, center_y: f64, height: f64) -> RadialGradientGeometry {
        resolve_radial_gradient(
            Default::default(),
            Some(surgeist_layout::Point::new(origin_x, center_y)),
            RadialGradientSize::EllipseRadii(surgeist_layout::Size::new(
                GradientPosition {
                    length: 0.0,
                    fraction: 0.0,
                },
                GradientPosition {
                    length: height,
                    fraction: 0.0,
                },
            )),
            options(),
        )
        .result
        .unwrap()
    }

    #[test]
    fn zero_width_identity_preserves_physical_two_hint_records_and_charges_only_required_actions() {
        let position = |length| GradientPosition {
            length,
            fraction: 0.0,
        };
        let source = [
            GradientStopItem::Stop {
                payload: Color::BLACK,
                positions: GradientStopPositions::Two(position(-4.0), position(0.0)),
            },
            GradientStopItem::Hint {
                position: position(2.0),
            },
            GradientStopItem::Stop {
                payload: Color::TRANSPARENT,
                positions: GradientStopPositions::Two(position(4.0), position(12.0)),
            },
        ];
        let used = resolve_gradient_stops(0.0, &source, options())
            .result
            .unwrap();
        let mut context = context(RenderLimits::new());
        let ordinary = prepare_radial_gradient(
            &zero_width_geometry(8.0, -f64::MAX, 8.0),
            &used,
            &mut context,
        )
        .unwrap();
        assert_eq!(ordinary.horizontal_distance_origin(), Some(8.0));
        assert_eq!(
            ordinary
                .stops()
                .iter()
                .map(|stop| stop.position())
                .collect::<Vec<_>>(),
            [-4.0, 0.0, 4.0, 12.0]
        );
        assert_eq!(
            ordinary
                .stops()
                .iter()
                .map(|stop| stop.hint_after().map(|hint| hint.position()))
                .collect::<Vec<_>>(),
            [None, Some(2.0), None, None]
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        // Origin capture, actual basis capture, and basis equality: 3. Each
        // of five source items is captured exactly once in physical identity.
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 8);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 10);
        assert_eq!(report.usage().work(RenderWork::PaintVisits), 3);
        let GradientKind::HorizontalDistance { stops, .. } = ordinary.kind() else {
            panic!("distance output")
        };
        let actual_bytes =
            (stops.capacity() * size_of::<GradientStop>() + size_of::<Gradient>()) as u64;
        assert_eq!(
            report.usage().peak(RenderResource::RetainedCpuBytes),
            actual_bytes
        );
        assert_eq!(
            report.usage().work(RenderWork::PreparedTransferredBytes),
            actual_bytes
        );
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        // Both-zero geometry has the same ZeroWidth interpretation; neither
        // center_y nor the unused original height is retained in the model.
        let same = prepare_radial_gradient(
            &zero_width_geometry(8.0, f64::MAX, 0.0),
            &used,
            &mut context,
        )
        .unwrap();
        assert_eq!(same, ordinary);
        let repeated = prepare_repeating_radial_gradient(
            &zero_width_geometry(8.0, 0.0, 8.0),
            &used,
            &mut context,
        )
        .unwrap();
        assert_eq!(repeated.horizontal_distance_origin(), Some(8.0));
        assert_eq!(repeated.stops(), ordinary.stops());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            10
        );
        context.set_options(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false })
                .with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::CoordinateVisits, 7)
                        .unwrap(),
                ),
        );
        let error =
            prepare_radial_gradient(&zero_width_geometry(8.0, 0.0, 8.0), &used, &mut context)
                .unwrap_err();
        let Some(crate::RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
        else {
            panic!("named scalar refusal")
        };
        assert_eq!(
            refusal.quantity(),
            crate::RenderQuantity::Work(RenderWork::CoordinateVisits)
        );
        assert_eq!(refusal.current(), 7);
        assert_eq!(refusal.amount(), 1);
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 7);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 10);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
        assert!(report.phase_count(RenderPhase::Cleanup) > 0);
        context.set_options(RenderAccountingOptions::new());
        assert_eq!(
            prepare_radial_gradient(&zero_width_geometry(8.0, 0.0, 8.0), &used, &mut context)
                .unwrap(),
            ordinary
        );
    }

    #[test]
    fn zero_width_basis_refuses_before_items_and_identity_has_no_quotient_loss() {
        let geometry = zero_width_geometry(0.0, 1.0, 8.0);
        let inputs = source(f64::from_bits(1), f64::MAX);
        let mut context = context(RenderLimits::new());
        let wrong = resolve_gradient_stops(1.0, &inputs, options())
            .result
            .unwrap();
        for error in [
            prepare_radial_gradient(&geometry, &wrong, &mut context).unwrap_err(),
            prepare_repeating_radial_gradient(&geometry, &wrong, &mut context).unwrap_err(),
        ] {
            assert_eq!(
                error.paint_gradient_diagnostic(),
                Some(&PaintGradientError::BasisMismatch {
                    expected: 0.0,
                    actual: 1.0
                })
            );
        }
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 3);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 0);
        let used = resolve_gradient_stops(0.0, &inputs, options())
            .result
            .unwrap();
        let ordinary = prepare_radial_gradient(&geometry, &used, &mut context).unwrap();
        assert_eq!(ordinary.stops()[0].position().to_bits(), 1);
        assert_eq!(ordinary.stops()[1].position(), f64::MAX);
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
        let source = source(-f64::MAX, f64::MAX);
        let used = resolve_gradient_stops(0.0, &source, options())
            .result
            .unwrap();
        assert!(prepare_radial_gradient(&geometry, &used, &mut context).is_ok());
        let error = prepare_repeating_radial_gradient(&geometry, &used, &mut context).unwrap_err();
        assert_eq!(
            error.paint_gradient_diagnostic(),
            Some(&PaintGradientError::UnrepresentableSpan {
                first_source_index: 0,
                last_source_index: 1
            })
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    fn zero_height_geometry() -> RadialGradientGeometry {
        resolve_radial_gradient(
            Default::default(),
            None,
            RadialGradientSize::EllipseRadii(surgeist_layout::Size::new(
                GradientPosition {
                    length: 20.0,
                    fraction: 0.0,
                },
                GradientPosition {
                    length: 0.0,
                    fraction: 0.0,
                },
            )),
            options(),
        )
        .result
        .unwrap()
    }

    #[test]
    fn zero_height_last_copies_color_without_channel_math_and_transfers_only_fixed_output() {
        use surgeist_paint::{GradientRepeat, resolve_zero_height_gradient_colors};
        let hidden = Color::try_rgba(1.0, 0.5, 0.25, 0.0).unwrap();
        let source = [GradientStopItem::Stop {
            payload: hidden,
            positions: GradientStopPositions::One(GradientPosition {
                length: 17.0,
                fraction: 0.0,
            }),
        }];
        let colors = resolve_zero_height_gradient_colors(
            &zero_height_geometry(),
            &source,
            GradientRepeat::None,
            options(),
        )
        .result
        .unwrap();
        let mut context = context(RenderLimits::new());
        assert_eq!(
            prepare_zero_height_gradient_color(&colors, &mut context).unwrap(),
            hidden
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::PaintVisits), 2);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 1);
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 0);
        assert_eq!(
            report.usage().peak(RenderResource::RetainedCpuBytes),
            size_of::<Color>() as u64
        );
        assert_eq!(
            report.usage().work(RenderWork::PreparedTransferredBytes),
            size_of::<Color>() as u64
        );
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
    }

    #[test]
    fn zero_height_average_counts_real_channel_actions_and_cleans_each_refused_prefix() {
        use surgeist_paint::{GradientRepeat, resolve_zero_height_gradient_colors};
        let source = [
            GradientStopItem::Stop {
                payload: Color::try_rgba(1.0, 0.0, 0.0, 0.0).unwrap(),
                positions: GradientStopPositions::One(GradientPosition {
                    length: 0.0,
                    fraction: 0.0,
                }),
            },
            GradientStopItem::Stop {
                payload: Color::try_rgba(0.0, 1.0, 0.0, 0.5).unwrap(),
                positions: GradientStopPositions::One(GradientPosition {
                    length: 10.0,
                    fraction: 0.0,
                }),
            },
            GradientStopItem::Stop {
                payload: Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
                positions: GradientStopPositions::One(GradientPosition {
                    length: 0.0,
                    fraction: 1.0,
                }),
            },
        ];
        let colors = resolve_zero_height_gradient_colors(
            &zero_height_geometry(),
            &source,
            GradientRepeat::Repeat,
            options(),
        )
        .result
        .unwrap();
        let mut context = context(RenderLimits::new());
        let expected = Color::try_rgba(0.0, 1.0 / 3.0, 2.0 / 3.0, 0.75).unwrap();
        assert_eq!(
            prepare_zero_height_gradient_color(&colors, &mut context).unwrap(),
            expected
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        // Each actual occurrence: weight capture1, premultiplication11,
        // weighted product/sum8. Final nonzero-alpha conversion12: 3*20+12.
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 72);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 3);
        assert_eq!(report.usage().work(RenderWork::PaintVisits), 2);
        assert_eq!(
            report.usage().peak(RenderResource::RetainedCpuBytes),
            size_of::<Color>() as u64
        );
        assert_eq!(
            report.usage().work(RenderWork::PreparedTransferredBytes),
            size_of::<Color>() as u64
        );
        assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
        for (limits, quantity, limit, current, amount, coordinates, visits, peak) in [
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 71)
                    .unwrap(),
                crate::RenderQuantity::Work(RenderWork::CoordinateVisits),
                71,
                71,
                1,
                71,
                3,
                0,
            ),
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::GradientStopVisits, 2)
                    .unwrap(),
                crate::RenderQuantity::Work(RenderWork::GradientStopVisits),
                2,
                2,
                1,
                40,
                2,
                0,
            ),
            (
                RenderLimits::new()
                    .with_work_limit(RenderWork::PaintVisits, 1)
                    .unwrap(),
                crate::RenderQuantity::Work(RenderWork::PaintVisits),
                1,
                1,
                1,
                72,
                3,
                size_of::<Color>() as u64,
            ),
            (
                RenderLimits::new().with_retained_bytes((size_of::<Color>() - 1) as u64),
                crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                (size_of::<Color>() - 1) as u64,
                0,
                size_of::<Color>() as u64,
                72,
                3,
                0,
            ),
        ] {
            // The output Paint visit refuses after a fixed Color lease exists.
            // Observe its ended cleanup phase explicitly. Other cases stay Off:
            // phase_count is observational and is always zero in that mode.
            let profile = if peak == 0 {
                RenderProfile::Off
            } else {
                RenderProfile::Summary { timing: false }
            };
            context.set_options(
                RenderAccountingOptions::new()
                    .with_limits(limits)
                    .with_profile(profile),
            );
            let error = prepare_zero_height_gradient_color(&colors, &mut context).unwrap_err();
            let Some(crate::RenderAccountingError::Refused(refusal)) =
                error.accounting_diagnostic()
            else {
                panic!("expected the named original admission refusal")
            };
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(refusal.quantity(), quantity);
            assert_eq!(refusal.current(), current);
            assert_eq!(refusal.amount(), amount);
            assert_eq!(
                refusal.reason(),
                crate::RenderRefusalReason::Limit { limit }
            );
            assert!(error.paint_gradient_diagnostic().is_none());
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.status(), crate::RenderAttemptStatus::Refused);
            assert_eq!(report.refusal(), Some(refusal));
            assert_eq!(
                report.usage().work(RenderWork::CoordinateVisits),
                coordinates
            );
            assert_eq!(report.usage().work(RenderWork::GradientStopVisits), visits);
            assert_eq!(report.usage().work(RenderWork::PaintVisits), 1);
            assert_eq!(report.usage().peak(RenderResource::RetainedCpuBytes), peak);
            assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
            assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
            assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
            if peak != 0 {
                assert!(report.phase_count(RenderPhase::Cleanup) > 0);
            }
        }
        context.set_options(RenderAccountingOptions::new());
        assert_eq!(
            prepare_zero_height_gradient_color(&colors, &mut context).unwrap(),
            expected
        );
    }

    #[test]
    fn scalar_counts_include_zero_short_circuit_without_duplicate_validation() {
        // Source geometry/basis: 6. Repeating span subtraction/domain: 2.
        // Conversion: 11 for [0,1], 12 for [1,2] (two source captures). This derives from scalar
        // actions, independently of the implementation's counter helpers.
        for (a, b, coordinates) in [(0.0, 1.0, 19), (1.0, 2.0, 20)] {
            let source = source(a, b);
            let stops = resolve_gradient_stops(1.0, &source, options())
                .result
                .unwrap();
            let mut context = context(RenderLimits::new());
            let result =
                prepare_repeating_radial_gradient(&geometry(), &stops, &mut context).unwrap();
            assert_eq!(result.stops()[0].position(), a);
            assert_eq!(result.stops()[1].position(), b);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report.usage().work(RenderWork::CoordinateVisits),
                coordinates
            );
            assert_eq!(report.usage().work(RenderWork::PaintVisits), 3);
            assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 4);
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
                0
            );
            assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        }
    }

    #[test]
    fn ordinary_adapter_consumes_the_proof_without_a_span_or_validation_walk() {
        for (a, b, coordinates) in [(0.0, 1.0, 17), (1.0, 2.0, 18)] {
            let source = source(a, b);
            let stops = resolve_gradient_stops(1.0, &source, options())
                .result
                .unwrap();
            let mut context = context(RenderLimits::new());
            let result = prepare_radial_gradient(&geometry(), &stops, &mut context).unwrap();
            assert_eq!(
                result.radial_geometry(),
                Some((Point::new(0.0, 0.0), 1.0, 1.0))
            );
            assert_eq!(
                result
                    .stops()
                    .iter()
                    .map(|s| s.position())
                    .collect::<Vec<_>>(),
                [a, b]
            );
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report.usage().work(RenderWork::CoordinateVisits),
                coordinates
            );
            assert_eq!(report.usage().work(RenderWork::PaintVisits), 3);
            assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 4);
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Output)
                    .work(RenderWork::CoordinateVisits),
                0
            );
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Output)
                    .work(RenderWork::GradientStopVisits),
                0
            );
            let GradientKind::Radial { stops, .. } = result.kind() else {
                panic!("radial adapter output")
            };
            let expected =
                (stops.capacity() * size_of::<GradientStop>() + size_of::<Gradient>()) as u64;
            assert_eq!(
                report.usage().peak(RenderResource::RetainedCpuBytes),
                expected
            );
            assert_eq!(
                report.usage().work(RenderWork::PreparedTransferredBytes),
                expected
            );
            assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        }
    }

    #[test]
    fn hinted_source_walks_scalar_captures_and_actual_color_capacity_are_accounted() {
        let [first, last] = source(0.0, 1.0);
        let source = [
            first,
            GradientStopItem::Hint {
                position: GradientPosition {
                    length: 0.25,
                    fraction: 0.0,
                },
            },
            last,
        ];
        let stops = resolve_gradient_stops(1.0, &source, options())
            .result
            .unwrap();
        let mut context = context(RenderLimits::new());
        let gradient = prepare_radial_gradient(&geometry(), &stops, &mut context).unwrap();
        assert_eq!(gradient.stops().len(), 2);
        assert_eq!(gradient.stops()[0].hint_after().unwrap().position(), 0.25);
        let report = context.last_attempt().unwrap().attempt().unwrap();
        // Geometry/basis 6. Three position captures/divides/finite/nonzero
        // checks = 12; two nonzero results need zero checks = 2; two source
        // adjacent equality/order checks = 4. No output validation walk.
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 24);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 6);
        assert_eq!(report.usage().work(RenderWork::PaintVisits), 3);
        let GradientKind::Radial { stops: backing, .. } = gradient.kind() else {
            panic!("radial adapter")
        };
        let actual_bytes =
            (backing.capacity() * size_of::<GradientStop>() + size_of::<Gradient>()) as u64;
        assert_eq!(
            report.usage().peak(RenderResource::RetainedCpuBytes),
            actual_bytes
        );
        assert_eq!(
            report.usage().work(RenderWork::PreparedTransferredBytes),
            actual_bytes
        );
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        // Refuse on the next scalar capture after both first color and hint
        // have been converted: 6 geometry + 4 color + 7 hint actions = 17.
        context.set_options(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 17)
                    .unwrap(),
            ),
        );
        assert!(
            prepare_radial_gradient(&geometry(), &stops, &mut context)
                .unwrap_err()
                .accounting_diagnostic()
                .is_some()
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 17);
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 6);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        context.set_options(RenderAccountingOptions::new());
        assert_eq!(
            prepare_radial_gradient(&geometry(), &stops, &mut context).unwrap(),
            gradient
        );
    }

    #[test]
    fn ordinary_adapter_accepts_overflowing_span_that_repeating_output_refuses() {
        let source = source(-f64::MAX, f64::MAX);
        let stops = resolve_gradient_stops(1.0, &source, options())
            .result
            .unwrap();
        let mut context = context(RenderLimits::new());
        let result = prepare_radial_gradient(&geometry(), &stops, &mut context).unwrap();
        assert_eq!(result.stops()[0].position(), -f64::MAX);
        assert_eq!(result.stops()[1].position(), f64::MAX);
        let error =
            prepare_repeating_radial_gradient(&geometry(), &stops, &mut context).unwrap_err();
        assert_eq!(
            error.paint_gradient_diagnostic(),
            Some(&PaintGradientError::UnrepresentableSpan {
                first_source_index: 0,
                last_source_index: 1,
            })
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
        assert!(report.phase_count(RenderPhase::Cleanup) > 0);
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
                    let (converted, indices, positions) =
                        convert(&stops, &unit_basis(1.0), 2, account)?;
                    let actual_bytes = converted.capacity() * size_of::<GradientStop>()
                        + size_of::<RepeatingGradient>();
                    let fixed = account.acquire(
                        RenderResource::RetainedCpuBytes,
                        size_of::<RepeatingGradient>(),
                    )?;
                    let guarded = RepeatingGradient::from_accounted_paint(
                        GradientGeometry::Radial {
                            center: Point::new(0.0, 0.0),
                            radius_x: 1.0,
                            radius_y: 1.0,
                        },
                        converted,
                        fixed,
                        indices,
                        positions,
                        account,
                    )?;
                    guarded.require_preparation()?;
                    let (gradient, backing, import) = guarded.into_parts();
                    for hold in backing.into_iter().flatten() {
                        hold.into_preparation()?.retain();
                    }
                    drop(import);
                    Ok((gradient, actual_bytes as u64))
                },
            )
            .unwrap();
        assert_eq!(gradient.stops().len(), 2);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
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
                let (converted, indices, positions) =
                    convert(&stops, &unit_basis(1.0), 2, account)?;
                actual_bytes.set(
                    (converted.capacity() * size_of::<GradientStop>()
                        + size_of::<RepeatingGradient>()) as u64,
                );
                let fixed = account.acquire(
                    RenderResource::RetainedCpuBytes,
                    size_of::<RepeatingGradient>(),
                )?;
                RepeatingGradient::from_accounted_paint(
                    GradientGeometry::Radial {
                        center: Point::new(0.0, 0.0),
                        radius_x: 1.0,
                        radius_y: 1.0,
                    },
                    converted,
                    fixed,
                    indices,
                    positions,
                    account,
                )
                .map(drop)
            },
        );
        assert!(result.unwrap_err().accounting_diagnostic().is_some());
        let report = context.last_attempt().unwrap().attempt().unwrap();
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
            .with_work_limit(RenderWork::CoordinateVisits, 5)
            .unwrap();
        let mut context = context(limits);
        let result = context.run(
            RenderAttemptKind::PaintGradient,
            RenderPhase::Coordinates,
            |account| convert(&stops, &unit_basis(1.0), 2, account).map(drop),
        );
        let error = result.unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(error.paint_gradient_diagnostic().is_none());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 5);
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
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            0
        );
    }
}
