//! Numeric box/control qualification of the resolved post-baseline operation.
use super::*;

fn input<S: LayoutScalar>(mode: WritingMode, amount: S) -> MixedInlineRunInputOf<S> {
    let axes = FlowAxes::new(mode, Direction::Ltr);
    let reference = crate::InlineBaselineContext::Named(crate::BaselineName::Central);
    let size = axes.physical_size(crate::geometry::LogicalSizeOf::new(
        S::from_f64(5.0),
        S::from_f64(10.0),
    ));
    MixedInlineRunInputOf {
        inline_baseline_context: reference,
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            Direction::Ltr,
        ),
        observe_tail: false,
        wrapping: None,
        available_inline_extent: AvailableOf::definite(S::from_f64(100.0)),
        flow_axes: axes,
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Start,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants: vec![
            MixedInlineParticipantOf::Atomic {
                item: AtomicInlineBoxParticipant {
                    baseline_reference: reference,
                    source_index: 0,
                    size,
                    content_size: size,
                    margin: Edges::ZERO,
                    padding: Edges::ZERO,
                    border: Edges::ZERO,
                    scrollbar_size: Size::ZERO,
                    first_baseline: Some(S::from_f64(6.0)),
                    alignment: InlineControlAlignment::Shift(
                        crate::BaselineOffsetOf::try_new(amount).unwrap(),
                    ),
                },
                participation: crate::AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    crate::InlineBreakOpportunity::Prohibited,
                ),
            },
            // The actual line-control contributes the unshifted8/2 strut;
            // the selected named child contributes raw6/4, without scaling.
            MixedInlineParticipantOf::ForcedLineBreak(ForcedLineBreakControlOf::new(
                1,
                InlineFlowOf::new(mode, Direction::Ltr, AvailableOf::MAX_CONTENT),
                InlineMetricsOf::try_new(reference, S::from_f64(8.0), S::from_f64(10.0)).unwrap(),
                InlineControlAlignment::Baseline,
                Clear::None,
            )),
        ],
    }
}

fn signed_envelope<S: LayoutScalar>() {
    let n = S::from_f64;
    for (amount, before, extent, child_start) in [(3.0, 9.0, 11.0, 0.0), (-3.0, 8.0, 15.0, 5.0)] {
        // +3: max(8,6+3)+max(2,4-3)=11.
        // -3: max(8,6-3)+max(2,4+3)=15.
        let report = layout_mixed_inline_run(input(WritingMode::HorizontalTb, n(amount))).unwrap();
        assert_eq!(report.baselines.numeric().first, Some(n(before)));
        assert_eq!(report.block_extent, n(extent));
        assert_eq!(
            crate::inline::expect_logical_position(report.atomics[0].position).block,
            n(child_start)
        );
        assert_eq!(report.atomics[0].item.first_baseline, Some(n(6.0)));
        assert_eq!(report.atomics[0].item.size, Size::new(n(5.0), n(10.0)));
        assert_eq!(
            report.atomics[0].item.content_size,
            Size::new(n(5.0), n(10.0))
        );
        assert!(report.baselines.numeric().first_named && report.baselines.numeric().last_named);
        assert_eq!(
            crate::inline::expect_logical_position(report.controls[0].position).block,
            n(before)
        );
    }

    // With only the child, displacement translates its interval relative to
    // the reference; no absent parent point enlarges the occupied ten units.
    let mut alone = input(WritingMode::HorizontalTb, n(20.0));
    alone.participants.pop();
    let report = layout_mixed_inline_run(alone).unwrap();
    assert_eq!(report.block_extent, n(10.0));
    assert_eq!(report.baselines.numeric().first, Some(n(26.0)));
    assert_eq!(
        crate::inline::expect_logical_position(report.atomics[0].position).block,
        S::ZERO
    );
}

#[test]
fn signed_envelope_preserves_raw_selected_baseline_and_absent_strut_both_scalars() {
    signed_envelope::<f32>();
    signed_envelope::<f64>();
}

#[test]
fn actual_forced_control_uses_the_same_post_match_operation_both_scalars() {
    fn lane<S: LayoutScalar>() {
        let n = S::from_f64;
        for (amount, common, extent, anchor, child) in
            [(3.0, 11.0, 15.0, 8.0, 5.0), (-3.0, 6.0, 11.0, 9.0, 0.0)]
        {
            let mut request = input(WritingMode::HorizontalTb, S::ZERO);
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut request.participants[0] else {
                unreachable!()
            };
            item.alignment = InlineControlAlignment::Baseline;
            let MixedInlineParticipantOf::ForcedLineBreak(control) = &mut request.participants[1]
            else {
                unreachable!()
            };
            *control = ForcedLineBreakControlOf::new(
                control.source_index(),
                control.flow(),
                control.metrics(),
                InlineControlAlignment::Shift(crate::BaselineOffsetOf::try_new(n(amount)).unwrap()),
                Clear::None,
            );
            let report = layout_mixed_inline_run(request).unwrap();
            // Shifted8/2 control +3 ->11/-1; -3 ->5/5. Union
            // with the actual unshifted6/4 atomic gives15 and11.
            assert_eq!(report.baselines.numeric().first, Some(n(common)));
            assert_eq!(report.block_extent, n(extent));
            assert_eq!(
                crate::inline::expect_logical_position(report.controls[0].position).block,
                n(anchor)
            );
            assert_eq!(
                crate::inline::expect_logical_position(report.atomics[0].position).block,
                n(child)
            );
            assert_eq!(report.atomics[0].item.first_baseline, Some(n(6.0)));
            assert_eq!(report.atomics[0].item.size.height, n(10.0));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

fn axes<S: LayoutScalar>() {
    let n = S::from_f64;
    for (mode, baseline, extent, dx, dy) in [
        (WritingMode::HorizontalTb, 9.0, 11.0, 0.0, -3.0),
        (WritingMode::VerticalRl, 9.0, 11.0, 3.0, 0.0),
        (WritingMode::VerticalLr, 8.0, 15.0, 3.0, 0.0),
        (WritingMode::SidewaysRl, 9.0, 11.0, 3.0, 0.0),
        (WritingMode::SidewaysLr, 9.0, 11.0, -3.0, 0.0),
    ] {
        let axes = FlowAxes::new(mode, Direction::Ltr);
        for available in [
            AvailableOf::definite(n(100.0)),
            AvailableOf::MIN_CONTENT,
            AvailableOf::MAX_CONTENT,
        ] {
            let mut request = input(mode, n(3.0));
            request.available_inline_extent = available;
            let report = layout_mixed_inline_run(request).unwrap();
            assert_eq!(report.baselines.numeric().first, Some(n(baseline)));
            assert_eq!(report.block_extent, n(extent));
            assert_eq!(report.inline_extent, n(5.0));
            let size = crate::geometry::LogicalSizeOf::new(n(5.0), n(10.0));
            let actual = axes.physical_point(
                crate::geometry::LogicalPointOf::new(
                    S::ZERO,
                    crate::inline::expect_logical_position(report.atomics[0].position).block,
                ),
                size,
                Size::new(n(100.0), n(100.0)),
            );
            // Compare in the same physical frame with the actual line's
            // matched baseline before the separately requested displacement.
            let matched = axes.physical_point(
                crate::geometry::LogicalPointOf::new(S::ZERO, n(baseline - 6.0)),
                size,
                Size::new(n(100.0), n(100.0)),
            );
            assert_eq!(actual.x - matched.x, n(dx));
            assert_eq!(actual.y - matched.y, n(dy));
            assert_eq!(report.atomics[0].item.first_baseline, Some(n(6.0)));
            assert_eq!(
                axes.logical_size(report.atomics[0].item.size).block,
                n(10.0)
            );
        }
    }
}

#[test]
fn positive_over_maps_all_five_axes_and_intrinsic_inputs_both_scalars() {
    axes::<f32>();
    axes::<f64>();
}

fn float_retry<S: LayoutScalar>() {
    let n = S::from_f64;
    let mut queries = Vec::new();
    let report = layout_mixed_inline_run_with_band_source(
        input(WritingMode::HorizontalTb, n(3.0)),
        |start, end| {
            queries.push((start, end));
            // A real five-unit child cannot fit the two-unit remaining band
            // beside an exclusion ending at block20. Retry below that edge.
            LogicalLineBandQueryResultOf {
                inline_start: S::ZERO,
                inline_end: if start < n(20.0) { n(2.0) } else { n(100.0) },
                next_transition: (start < n(20.0)).then_some(n(20.0)),
            }
        },
        |block, _| block,
    )
    .unwrap();
    assert!(queries.contains(&(S::ZERO, n(11.0))));
    assert!(queries.contains(&(n(20.0), n(31.0))));
    assert_eq!(report.line_bands.len(), 1);
    assert_eq!(report.line_bands[0].block_start, n(20.0));
    assert_eq!(report.line_bands[0].block_end, n(31.0));
    assert_eq!(report.baselines.numeric().first, Some(n(29.0)));
    assert_eq!(
        crate::inline::expect_logical_position(report.atomics[0].position).block,
        n(20.0)
    );
    assert_eq!(report.block_extent, n(31.0));
}

#[test]
fn actual_shifted_bounds_drive_exclusion_retry_both_scalars() {
    float_retry::<f32>();
    float_retry::<f64>();
}

#[test]
fn raw_fallback_centering_precedes_shift_and_envelope_normalization_both_scalars() {
    fn lane<S: LayoutScalar>() {
        let n = S::from_f64;
        for (amount, common, extent, start) in [
            (0.0, 4.0, 10.0, 2.0),
            (3.0, 5.0, 11.0, 0.0),
            (-3.0, 4.0, 11.0, 5.0),
        ] {
            let mut request = input(WritingMode::HorizontalTb, n(amount));
            request.inline_baseline_context = crate::InlineBaselineContext::Physical;
            request.available_inline_extent = AvailableOf::definite(n(110.0));
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut request.participants[0] else {
                unreachable!()
            };
            item.baseline_reference = crate::InlineBaselineContext::Physical;
            item.first_baseline = None;
            item.size.height = n(6.0);
            item.content_size.height = n(6.0);
            let MixedInlineParticipantOf::ForcedLineBreak(control) = &mut request.participants[1]
            else {
                unreachable!()
            };
            *control = ForcedLineBreakControlOf::new(
                1,
                control.flow(),
                InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, n(4.0), n(10.0))
                    .unwrap(),
                InlineControlAlignment::Baseline,
                Clear::None,
            );
            let mut queries = Vec::new();
            let report = layout_mixed_inline_run_with_band_source(
                request,
                |top, bottom| {
                    queries.push((top, bottom));
                    let intersects = top < n(20.0) && bottom > S::ZERO;
                    LogicalLineBandQueryResultOf {
                        inline_start: if intersects { n(10.0) } else { S::ZERO },
                        inline_end: n(110.0),
                        next_transition: intersects.then_some(n(20.0)),
                    }
                },
                |block, _| block,
            )
            .unwrap();
            // Raw4<6<10 selects the existing centered origin(10-6)/2=2.
            // +3 moves it to[-1,5], then normalizes the union with[0,10].
            // -3 moves it to[5,11]; zero retains the original centered band.
            assert_eq!(report.baselines.numeric().first, Some(n(common)));
            assert_eq!(report.block_extent, n(extent));
            assert_eq!(
                crate::inline::expect_logical_position(report.atomics[0].position).block,
                n(start)
            );
            assert_eq!(report.atomics[0].item.first_baseline, None);
            assert_eq!(report.atomics[0].item.size.height, n(6.0));
            assert_eq!(report.atomics[0].item.content_size.height, n(6.0));
            assert_eq!(
                crate::inline::expect_logical_position(report.controls[0].position).block,
                n(common)
            );
            assert_eq!(report.line_bands[0].block_end, n(extent));
            if amount != 0.0 {
                assert!(queries.contains(&(S::ZERO, n(extent))));
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn actual_boundary_shift_and_zero_share_the_common_float_reference_both_scalars() {
    fn lane<S: LayoutScalar>() {
        let n = S::from_f64;
        for (alignment, common, extent, anchor, child) in [
            (InlineControlAlignment::Baseline, 8.0, 12.0, 8.0, 2.0),
            (
                InlineControlAlignment::Shift(crate::BaselineOffsetOf::ZERO),
                8.0,
                12.0,
                8.0,
                2.0,
            ),
            (
                InlineControlAlignment::Shift(crate::BaselineOffsetOf::try_new(n(3.0)).unwrap()),
                11.0,
                15.0,
                8.0,
                5.0,
            ),
            (
                InlineControlAlignment::Shift(crate::BaselineOffsetOf::try_new(n(-3.0)).unwrap()),
                6.0,
                11.0,
                9.0,
                0.0,
            ),
        ] {
            let mut request = input(WritingMode::HorizontalTb, S::ZERO);
            request.available_inline_extent = AvailableOf::definite(n(110.0));
            request.participants.pop();
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut request.participants[0] else {
                unreachable!()
            };
            item.alignment = InlineControlAlignment::Baseline;
            item.source_index = 1;
            request.participants.insert(
                0,
                MixedInlineParticipantOf::Boundary(InlineBoundaryControlOf::new(
                    0,
                    InlineBoundaryKind::Start,
                    InlineFlowOf::new(
                        WritingMode::HorizontalTb,
                        Direction::Ltr,
                        AvailableOf::MAX_CONTENT,
                    ),
                    InlineMetricsOf::try_new(request.inline_baseline_context, n(8.0), n(10.0))
                        .unwrap(),
                    alignment,
                    InlineBoundaryEdgeOf::ZERO,
                )),
            );
            let report = layout_mixed_inline_run_with_band_source(
                request,
                |top, bottom| {
                    let intersects = top < n(20.0) && bottom > S::ZERO;
                    LogicalLineBandQueryResultOf {
                        inline_start: if intersects { n(10.0) } else { S::ZERO },
                        inline_end: n(110.0),
                        next_transition: intersects.then_some(n(20.0)),
                    }
                },
                |block, _| block,
            )
            .unwrap();
            assert_eq!(report.baselines.numeric().first, Some(n(common)));
            assert_eq!(report.block_extent, n(extent));
            assert_eq!(
                crate::inline::expect_logical_position(report.controls[0].position).block,
                n(anchor)
            );
            assert_eq!(
                crate::inline::expect_logical_position(report.atomics[0].position).block,
                n(child)
            );
            assert_eq!(report.atomics[0].item.first_baseline, Some(n(6.0)));
            assert!(
                report.baselines.numeric().first_named && report.baselines.numeric().last_named
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn absorbed_displacement_refuses_before_band_query_both_scalars() {
    fn lane<S: LayoutScalar>(amount: S) {
        let mut queried = false;
        let result = layout_mixed_inline_run_with_band_source(
            input(WritingMode::HorizontalTb, amount),
            |_, _| {
                queried = true;
                LogicalLineBandQueryResultOf {
                    inline_start: S::ZERO,
                    inline_end: S::from_f64(100.0),
                    next_transition: None,
                }
            },
            |block, _| block,
        );
        assert!(
            matches!(result, Err(NumericWrappingError::Group(error)) if error.reason() == crate::LayoutTextNumericError::NonZeroLost)
        );
        assert!(!queried);
    }
    lane::<f32>(f32::MAX);
    lane::<f64>(f64::MAX);
}

#[test]
fn large_real_clearance_anchor_cannot_absorb_the_shifted_line_both_scalars() {
    fn lane<S: LayoutScalar>(edge: S) {
        let mut request = input(WritingMode::HorizontalTb, S::from_f64(3.0));
        let MixedInlineParticipantOf::Atomic { item, .. } = &mut request.participants[0] else {
            unreachable!()
        };
        item.source_index = 1;
        let MixedInlineParticipantOf::ForcedLineBreak(control) = &mut request.participants[1]
        else {
            unreachable!()
        };
        *control = ForcedLineBreakControlOf::new(
            2,
            control.flow(),
            control.metrics(),
            control.alignment(),
            control.clear(),
        );
        request.participants.insert(
            0,
            MixedInlineParticipantOf::ForcedLineBreak(ForcedLineBreakControlOf::new(
                0,
                InlineFlowOf::new(
                    WritingMode::HorizontalTb,
                    Direction::Ltr,
                    AvailableOf::MAX_CONTENT,
                ),
                InlineMetricsOf::try_new(request.inline_baseline_context, S::ZERO, S::ZERO)
                    .unwrap(),
                InlineControlAlignment::Baseline,
                Clear::Both,
            )),
        );
        let mut queries = Vec::new();
        let result = layout_mixed_inline_run_with_band_source(
            request,
            |start, end| {
                queries.push((start, end));
                // A full-width finite float ends at edge. The initial
                // zero-height Forced line has no intersecting interval.
                let intersects = start < edge && end > S::ZERO && end > start;
                LogicalLineBandQueryResultOf {
                    inline_start: S::ZERO,
                    inline_end: if intersects {
                        S::ZERO
                    } else {
                        S::from_f64(100.0)
                    },
                    next_transition: intersects.then_some(edge),
                }
            },
            |block, clear| {
                if clear == PostLineClearIntent::Both {
                    edge
                } else {
                    block
                }
            },
        );
        assert!(
            matches!(result, Err(NumericWrappingError::Group(error)) if error.reason() == crate::LayoutTextNumericError::NonZeroLost)
        );
        assert!(queries.iter().all(|(start, _)| *start == S::ZERO));
    }
    lane::<f32>(1_073_741_824.0);
    lane::<f64>(1_152_921_504_606_846_976.0);
}
