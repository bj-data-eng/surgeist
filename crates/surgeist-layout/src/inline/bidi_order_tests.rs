//! Numeric L2 slots retain line-left order through the actual flow projection.
//! Physical expectations follow pinned Writing Modes4 Table11, not the slot helper.

use super::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn atomic<S: LayoutScalar>(
    axes: FlowAxes,
    source: usize,
    inline: f64,
    level: u8,
    margin_start: f64,
    margin_end: f64,
) -> MixedInlineParticipantOf<S> {
    let size = axes.physical_size(LogicalSizeOf::new(n(inline), n(10.0)));
    MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            baseline_reference: crate::InlineBaselineContext::Physical,
            source_index: source,
            size,
            content_size: size,
            margin: axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                n(margin_start),
                n(margin_end),
                S::ZERO,
                S::ZERO,
            )),
            border: Edges::ZERO,
            padding: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: Some(n(8.0)),
            alignment: InlineControlAlignment::Baseline,
        },
        participation: AtomicInlineParticipation::new(
            BidiLevel::try_new(level).unwrap(),
            InlineBreakOpportunity::Prohibited,
        ),
    }
}

fn input<S: LayoutScalar>(
    axes: FlowAxes,
    capacity: f64,
    participants: Vec<MixedInlineParticipantOf<S>>,
) -> MixedInlineRunInputOf<S> {
    MixedInlineRunInputOf {
        inline_baseline_context: crate::InlineBaselineContext::Physical,
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        wrapping: None,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            axes.direction(),
        ),
        observe_tail: true,
        available_inline_extent: AvailableOf::definite(n(capacity)),
        flow_axes: axes,
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants,
    }
}

fn run<S: LayoutScalar>(
    input: MixedInlineRunInputOf<S>,
    band_start: f64,
    band_end: f64,
) -> MixedInlineRunReportOf<S> {
    layout_mixed_inline_run_with_band_source(
        input,
        |_, _| LogicalLineBandQueryResultOf {
            inline_start: n(band_start),
            inline_end: n(band_end),
            next_transition: None,
        },
        |block, intent| {
            assert_eq!(intent, PostLineClearIntent::None);
            block
        },
    )
    .unwrap()
}

fn physical_atomic<S: LayoutScalar>(
    axes: FlowAxes,
    row: &AtomicInlineSourceOf<S>,
    frame_inline: f64,
) -> crate::Point<S> {
    axes.physical_point(
        crate::inline::expect_logical_position(row.position),
        axes.logical_size(row.item.size),
        axes.physical_size(LogicalSizeOf::new(n(frame_inline), n(10.0))),
    )
}

fn physical_inline<S: LayoutScalar>(mode: WritingMode, value: f64) -> crate::Point<S> {
    match mode {
        WritingMode::HorizontalTb => crate::Point::new(n(value), S::ZERO),
        _ => crate::Point::new(S::ZERO, n(value)),
    }
}

fn table_lane<S: LayoutScalar>() {
    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let axes = FlowAxes::new(mode, direction);
            let level = u8::from(direction == Direction::Rtl);
            let a = atomic::<S>(axes, 17, 10.0, level, 0.0, 0.0);
            let b = atomic(axes, 41, 20.0, level, 0.0, 0.0);
            let report = run(input(axes, 30.0, vec![a, b]), 0.0, 30.0);
            // Table11: line-left is bottom only in SidewaysLr. An RTL
            // containing flow reverses line-to-flow coordinates even there,
            // where physical inline progression is increasing rather than decreasing.
            let expected = match (mode, direction) {
                (WritingMode::SidewaysLr, Direction::Ltr) => [20.0, 0.0],
                (WritingMode::SidewaysLr, Direction::Rtl) => [0.0, 10.0],
                (_, Direction::Ltr) => [0.0, 10.0],
                (_, Direction::Rtl) => [20.0, 0.0],
            };
            let visual = if direction == Direction::Ltr {
                [0, 1]
            } else {
                [1, 0]
            };
            assert_eq!(report.atomics.len(), 2);
            for (index, original) in [a, b].into_iter().enumerate() {
                let MixedInlineParticipantOf::Atomic { item, .. } = original else {
                    unreachable!()
                };
                let row = &report.atomics[index];
                assert_eq!(row.item, item);
                assert_eq!(row.visual_index, Some(visual[index]));
                assert_eq!(row.line_index, 0);
                assert_eq!(
                    physical_atomic(axes, row, 30.0),
                    physical_inline(mode, expected[index]),
                    "{mode:?} {direction:?} source{}",
                    row.item.source_index
                );
            }
            assert_eq!(report.block_extent, n(10.0));
            assert_eq!(report.baselines.numeric().first, Some(n(8.0)));
            assert_eq!(report.baselines.numeric().last, Some(n(8.0)));
            assert_eq!(report.line_bands.len(), 1);
        }
    }
}

#[test]
fn all_writing_mode_directions_preserve_literal_l2_physical_order_and_source_identity() {
    table_lane::<f32>();
    table_lane::<f64>();
}

fn mixed_lane<S: LayoutScalar>() {
    for (direction, levels, visual, positions) in [
        (Direction::Ltr, [0, 1, 1], [0, 2, 1], [0.0, 15.0, 10.0]),
        (Direction::Rtl, [1, 2, 2], [2, 0, 1], [25.0, 0.0, 20.0]),
    ] {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
        let participants = [10.0, 20.0, 5.0]
            .into_iter()
            .enumerate()
            .map(|(index, width)| atomic(axes, [17, 41, 73][index], width, levels[index], 0.0, 0.0))
            .collect();
        let report = run(input(axes, 35.0, participants), 0.0, 35.0);
        assert_eq!(report.atomics.len(), 3);
        // L2 gives A,C,B for0/1/1 and B,C,A for1/2/2. The receipt remains
        // in source order; its visual ordinal is not reflected a second time.
        for (index, row) in report.atomics.iter().enumerate() {
            assert_eq!(row.item.source_index, [17, 41, 73][index]);
            assert_eq!(row.visual_index, Some(visual[index]));
            assert_eq!(
                physical_atomic(axes, row, 35.0),
                crate::Point::new(n(positions[index]), S::ZERO)
            );
        }
    }
}

#[test]
fn mixed_resolved_levels_keep_original_visual_ordinals_when_coordinates_change_basis() {
    mixed_lane::<f32>();
    mixed_lane::<f64>();
}

fn decorated_lane<S: LayoutScalar>() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    let flow = InlineFlowOf::new(
        WritingMode::HorizontalTb,
        Direction::Rtl,
        AvailableOf::definite(n(60.0)),
    );
    let metrics =
        InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, n(8.0), n(10.0)).unwrap();
    let boundary = |source, kind, border, padding| {
        InlineBoundaryControlOf::new(
            source,
            kind,
            flow,
            metrics,
            InlineControlAlignment::Baseline,
            InlineBoundaryEdgeOf::try_new(S::ZERO, n(border), n(padding)).unwrap(),
            None,
        )
    };
    let start = boundary(17, InlineBoundaryKind::Start, 2.0, 1.0);
    let end = boundary(59, InlineBoundaryKind::End, 1.0, 1.0);
    let a = atomic(axes, 23, 10.0, 1, -2.0, 1.0);
    let b = atomic(axes, 41, 20.0, 1, 1.0, -3.0);
    let forced = ForcedLineBreakControlOf::new(
        77,
        flow,
        metrics,
        InlineControlAlignment::Baseline,
        Clear::None,
    );
    let mut query = input(
        axes,
        60.0,
        vec![
            MixedInlineParticipantOf::Boundary(start),
            a,
            b,
            MixedInlineParticipantOf::Boundary(end),
            MixedInlineParticipantOf::ForcedLineBreak(forced),
        ],
    );
    query.indent.amount = n(5.0);
    let report = run(query, 7.0, 47.0);
    // Signed advances are3,9,18,2 (total32), with selected flow start12.
    // RTL slots: Start12,A15,B24,End42; applying each atomic's own signed
    // start margin gives border origins A13/B25, physical x37/x15.
    assert_eq!(report.atomics.len(), 2);
    for (row, (source, x, visual)) in report.atomics.iter().zip([(23, 37.0, 2), (41, 15.0, 1)]) {
        assert_eq!(row.item.source_index, source);
        assert_eq!(row.visual_index, Some(visual));
        assert_eq!(
            physical_atomic(axes, row, 60.0),
            crate::Point::new(n(x), S::ZERO)
        );
    }
    for (row, original) in report.atomics.iter().zip([a, b]) {
        let MixedInlineParticipantOf::Atomic { item, .. } = original else {
            unreachable!()
        };
        assert_eq!(row.item, item);
    }
    assert_eq!(report.controls.len(), 3);
    for (row, (source, logical, extent, visual)) in report.controls.iter().zip([
        (17, 12.0, 3.0, Some(3)),
        (59, 42.0, 2.0, Some(0)),
        (77, 44.0, 0.0, None),
    ]) {
        assert_eq!(
            (row.source_index, row.visual_index, row.line_index),
            (source, visual, 0)
        );
        assert_eq!(
            crate::inline::expect_logical_position(row.position).inline,
            n(logical)
        );
        assert_eq!(row.inline_extent, n(extent));
        assert_eq!(
            crate::inline::expect_logical_position(row.position).block,
            n(8.0)
        );
    }
    assert_eq!(
        report.controls[0].endpoint,
        Some(InlineEndpointOf::from_control(start))
    );
    assert_eq!(
        report.controls[1].endpoint,
        Some(InlineEndpointOf::from_control(end))
    );
    assert!(report.controls[2].endpoint.is_none());
    assert_eq!(report.lines.len(), 1);
    assert_eq!(report.lines[0].end, surgeist_text::LineEnd::Forced);
    assert_eq!(report.block_extent, n(10.0));
    assert_eq!(report.line_bands.len(), 1);
    assert!(report.nonformatted_controls.is_empty());
}

#[test]
fn signed_margins_positive_endpoint_components_and_forced_completion_keep_real_receipts() {
    decorated_lane::<f32>();
    decorated_lane::<f64>();
}

fn opposed_lane<S: LayoutScalar>() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    for (widths, positions) in [([10.0, 20.0], [36.0, 16.0]), ([30.0, 20.0], [36.0, 16.0])] {
        let mut query = input(
            axes,
            60.0,
            vec![
                atomic(axes, 17, widths[0], 1, 0.0, 0.0),
                atomic(axes, 41, widths[1], 1, 0.0, 0.0),
            ],
        );
        // The actual typed Plaintext transition can retain a preceding strong
        // LTR line independently of this containing RTL flow. No opaque Text
        // source, endpoint, receipt or fabricated SelectedInlineLine is needed.
        let mut context = crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Plaintext,
            Direction::Rtl,
        );
        context.commit(context.resolve(Some(surgeist_text::ParagraphBaseDirection::LeftToRight)));
        query.line_box_context = context;
        query.indent.amount = n(3.0);
        let report = run(query, 7.0, 47.0);
        assert_eq!(report.line_box_context.weak_direction(), Direction::Ltr);
        assert_eq!(report.atomics.len(), 2);
        // The actual left-start alignment frame puts the short row at x16;
        // the unbreakable50px row overflows toward the same resolved Start.
        // Coordinates still convert by containing RTL, never by line LTR.
        for (index, row) in report.atomics.iter().enumerate() {
            assert_eq!(row.item.source_index, [17, 41][index]);
            assert_eq!(row.visual_index, Some([1, 0][index]));
            assert_eq!(
                physical_atomic(axes, row, 60.0),
                crate::Point::new(n(positions[index]), S::ZERO)
            );
        }
        assert_eq!(report.line_bands.len(), 1);
        assert_eq!(report.block_extent, n(10.0));
    }
}

#[test]
fn opposed_line_alignment_uses_containing_direction_for_short_and_overlong_offset_rows() {
    opposed_lane::<f32>();
    opposed_lane::<f64>();
}
