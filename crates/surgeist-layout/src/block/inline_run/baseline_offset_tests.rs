//! Resolved shifts retain actual named child facts before numeric placement.
use super::*;
use crate::{Display, InlineMetricsOf, WritingMode};

fn participant<S: LayoutScalar>(
    input: &crate::NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    name: crate::BaselineName,
    margin: Edges<S>,
) -> Result<AtomicInlineBoxParticipant<S>, LayoutErrorKindOf<S, ()>> {
    atomic_inline_box_participant(
        0,
        BlockChildProjection::from_node(input),
        output,
        margin,
        Edges::ZERO,
        Edges::ZERO,
        AtomicInlineContext {
            axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            baseline: crate::InlineBaselineContext::Named(name),
        },
    )
}

fn facts<S: LayoutScalar>() -> crate::NamedBaselinesOf<S> {
    let fact = |value| {
        crate::BaselineFactOf::Available(
            crate::BaselineCoordinateOf::try_new(PhysicalAxis::Vertical, S::from_f64(value))
                .unwrap(),
        )
    };
    crate::NamedBaselinesOf::unknown()
        .with(
            crate::BaselineName::Alphabetic,
            crate::BaselinePairOf::new(fact(2.0), fact(3.0)),
        )
        .with(
            crate::BaselineName::Central,
            crate::BaselinePairOf::new(fact(6.0), fact(7.0)),
        )
}

#[test]
fn shifts_match_actual_named_preference_before_displacing_both_scalars() {
    fn lane<S: LayoutScalar>() {
        let n = S::from_f64;
        let size = Size::new(n(5.0), n(10.0));
        let output = ComputeOutputOf::from_sizes(size, size).with_named_baselines(facts());
        for amount in [S::ZERO, n(3.0)] {
            for (display, source, alphabetic, central) in [
                (Display::InlineBlock, crate::BaselineSource::First, 2.0, 6.0),
                (Display::InlineBlock, crate::BaselineSource::Last, 3.0, 7.0),
                (Display::InlineBlock, crate::BaselineSource::Auto, 3.0, 7.0),
                (Display::InlineGrid, crate::BaselineSource::Auto, 2.0, 6.0),
            ] {
                let input = crate::NodeInputOf {
                    display,
                    baseline_source: source,
                    vertical_align: VerticalAlignOf::Shift(
                        crate::BaselineOffsetOf::try_new(amount).unwrap(),
                    ),
                    ..crate::NodeInputOf::default()
                };
                for (name, expected) in [
                    (crate::BaselineName::Alphabetic, alphabetic),
                    (crate::BaselineName::Central, central),
                ] {
                    let demand = named_atomic_demand(
                        &BlockChildProjection::from_node(&input),
                        crate::InlineBaselineContext::Named(name),
                    );
                    assert_eq!(
                        demand.selected_fact(facts()),
                        Some((
                            name,
                            crate::BaselineFactOf::Available(
                                crate::BaselineCoordinateOf::try_new(
                                    PhysicalAxis::Vertical,
                                    n(expected)
                                )
                                .unwrap()
                            )
                        ))
                    );
                    let item = participant(&input, output, name, Edges::ZERO).unwrap();
                    assert_eq!(item.first_baseline, Some(n(expected)));
                    assert_eq!(
                        item.baseline_reference,
                        crate::InlineBaselineContext::Named(name)
                    );
                    assert_eq!(item.alignment, input.vertical_align);
                    assert_eq!(item.size, size);
                    assert_eq!(item.content_size, size);
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn shifted_demand_keeps_unknown_axis_absence_margin_and_scroll_contracts_both_scalars() {
    fn lane<S: LayoutScalar>() {
        let n = S::from_f64;
        let size = Size::new(n(5.0), n(10.0));
        let plain = ComputeOutputOf::from_sizes(size, size);
        for amount in [S::ZERO, n(3.0)] {
            let input = crate::NodeInputOf {
                display: Display::InlineBlock,
                vertical_align: VerticalAlignOf::Shift(
                    crate::BaselineOffsetOf::try_new(amount).unwrap(),
                ),
                ..crate::NodeInputOf::default()
            };
            assert!(matches!(
                participant(&input, plain, crate::BaselineName::Central, Edges::ZERO),
                Err(LayoutErrorKindOf::MissingContext(
                    crate::LayoutMissingContext::NamedInlineBaseline
                ))
            ));
            let wrong = crate::BaselineFactOf::Available(
                crate::BaselineCoordinateOf::try_new(PhysicalAxis::Horizontal, n(6.0)).unwrap(),
            );
            let wrong = plain.with_named_baselines(crate::NamedBaselinesOf::unknown().with(
                crate::BaselineName::Central,
                crate::BaselinePairOf::new(wrong, wrong),
            ));
            assert!(matches!(
                participant(&input, wrong, crate::BaselineName::Central, Edges::ZERO),
                Err(LayoutErrorKindOf::InvalidInput(
                    LayoutInvalidInputOf::InlineBaseline(
                        crate::InlineBaselineError::AxisMismatch {
                            expected: PhysicalAxis::Vertical,
                            actual: PhysicalAxis::Horizontal
                        }
                    )
                ))
            ));

            let absent = plain.with_named_baselines(crate::NamedBaselinesOf::absent());
            let margin = Edges {
                top: n(2.0),
                bottom: n(3.0),
                left: S::ZERO,
                right: S::ZERO,
            };
            // Margin under edge13; central(-2+13)/2=5.5. These are raw
            // matched local coordinates; amount is consumed only downstream.
            assert_eq!(
                participant(&input, absent, crate::BaselineName::Alphabetic, margin)
                    .unwrap()
                    .first_baseline,
                Some(n(13.0))
            );
            assert_eq!(
                participant(&input, absent, crate::BaselineName::Central, margin)
                    .unwrap()
                    .first_baseline,
                Some(n(5.5))
            );
            let scrolling = crate::NodeInputOf {
                overflow: crate::ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden)
                    .unwrap(),
                ..input.clone()
            };
            assert_eq!(
                participant(&scrolling, absent, crate::BaselineName::Alphabetic, margin)
                    .unwrap()
                    .first_baseline,
                Some(n(10.0))
            );

            let mismatched = InlineMetricsOf::try_new(
                crate::InlineBaselineContext::Named(crate::BaselineName::Alphabetic),
                n(8.0),
                n(10.0),
            )
            .unwrap();
            let error = validate_metric_reference::<u32, S, ()>(
                1,
                crate::InlineBaselineContext::Named(crate::BaselineName::Central),
                mismatched,
                input.vertical_align,
            )
            .unwrap_err();
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineBaseline(
                    crate::InlineBaselineError::ReferenceMismatch { .. }
                ))
            ));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn projection_checks_one_retained_frame_and_local_coordinate_loss_both_scalars() {
    fn lane<S: LayoutScalar>(large: S) {
        let n = S::from_f64;
        let alignment = VerticalAlignOf::Shift(crate::BaselineOffsetOf::try_new(n(20.0)).unwrap());
        let horizontal = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        // The accepted no-strut line has common baseline26 and child start0.
        // Its pre-displacement matched start is20 in this same frame. An
        // independent unshifted solve would instead have baseline6/start0.
        assert_eq!(
            checked_shift_projection(alignment, horizontal, S::ZERO, |block| Point::new(
                S::ZERO,
                block
            ))
            .unwrap(),
            Point::ZERO
        );

        let one = VerticalAlignOf::Shift(crate::BaselineOffsetOf::try_new(S::ONE).unwrap());
        // Local coordinates remain small when only an ancestor is translated;
        // no ancestor-origin value participates in this parent-local frame.
        assert_eq!(
            checked_shift_projection(one, horizontal, S::ZERO, |block| Point::new(S::ZERO, block))
                .unwrap(),
            Point::ZERO
        );
        let inset_loss = checked_shift_projection(one, horizontal, S::ZERO, |block| {
            Point::new(S::ZERO, large + block)
        })
        .unwrap_err();
        assert_eq!(
            inset_loss.reason(),
            crate::LayoutTextNumericError::NonZeroLost
        );

        let vertical = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        let size = LogicalSizeOf::new(n(5.0), n(10.0));
        let projection_loss = checked_shift_projection(one, vertical, S::ZERO, |block| {
            vertical.physical_point(
                LogicalPointOf::new(S::ZERO, block),
                size,
                Size::new(large, n(100.0)),
            )
        })
        .unwrap_err();
        assert_eq!(
            projection_loss.reason(),
            crate::LayoutTextNumericError::NonZeroLost
        );
    }
    lane::<f32>(1_073_741_824.0);
    lane::<f64>(1_152_921_504_606_846_976.0);
}
