use crate::Available;
use crate::inline::{
    AtomicInlineBoxParticipant, InlineControlAlignment, InlineParticipant,
    InlineParticipantLayoutKind, InlineRunInput, LogicalLineBandQueryResultOf,
    MixedInlineParticipantOf, MixedInlineRunInputOf, PostLineClearIntent,
    inline_candidate_scan_visits, inline_run_max_content_width, inline_run_min_content_width,
    layout_inline_run, layout_mixed_inline_run, layout_mixed_inline_run_with_band_source,
    mapped_post_line_clear_intent, reset_inline_candidate_scan_visits,
};
use crate::*;

fn mr02_atomic<S: LayoutScalar>(
    source_index: usize,
    extent: f64,
    block_extent: f64,
    baseline: f64,
    bidi_level: u8,
    following_break: InlineBreakOpportunity,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            source_index,
            size: Size::new(S::from_f64(extent), S::from_f64(block_extent)),
            content_size: Size::new(S::from_f64(extent), S::from_f64(block_extent)),
            margin: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: Some(S::from_f64(baseline)),
            alignment: InlineControlAlignment::Baseline,
        },
        participation: AtomicInlineParticipation::new(
            BidiLevel::try_new(bidi_level).unwrap(),
            following_break,
        ),
    }
}

fn mr02_input<S: LayoutScalar>(
    available_inline_extent: AvailableOf<S>,
    participants: Vec<MixedInlineParticipantOf<S>>,
) -> MixedInlineRunInputOf<S> {
    MixedInlineRunInputOf {
        available_inline_extent,
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        participants,
    }
}

#[test]
fn authored_atomic_shape_band_break_target_keeps_34_38_then_42_46_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let run = |break_after| {
            let mut participants = vec![mixed_boundary(
                100,
                flow,
                InlineMetricsOf::from_ascent_descent(S::from_f64(16.0), S::from_f64(4.0)).unwrap(),
            )];
            // Every extent is an authored box allocation. The first 48px box
            // does not represent the historical fixture's measured text.
            for (index, width) in [48.0, 34.0, 38.0, 42.0, 46.0].into_iter().enumerate() {
                participants.push(mr02_atomic(
                    index,
                    width,
                    16.0,
                    16.0,
                    0,
                    if index == break_after {
                        InlineBreakOpportunity::Allowed
                    } else {
                        InlineBreakOpportunity::Prohibited
                    },
                ));
            }
            layout_mixed_inline_run_with_band_source(
                mr02_input(AvailableOf::definite(S::from_f64(180.0)), participants),
                |start, _| LogicalLineBandQueryResultOf {
                    inline_start: if start < S::from_f64(40.0) {
                        S::from_f64(44.0)
                    } else {
                        S::ZERO
                    },
                    inline_end: S::from_f64(180.0),
                    next_transition: if start < S::from_f64(40.0) {
                        Some(S::from_f64(40.0))
                    } else {
                        None
                    },
                },
                |block, _| block,
            )
        };
        // The first band is 136px: 48+34+38=120 fits, adding 42 does not.
        // The actual allowed endpoint after 38 commits 42+46 on the next band.
        let actual = run(2);
        assert_eq!(
            actual
                .atomics
                .iter()
                .map(|row| (row.item.source_index, row.line_index, row.inline_start))
                .collect::<Vec<_>>(),
            vec![
                (0, 0, S::from_f64(44.0)),
                (1, 0, S::from_f64(92.0)),
                (2, 0, S::from_f64(126.0)),
                (3, 1, S::from_f64(44.0)),
                (4, 1, S::from_f64(86.0))
            ]
        );
        assert_eq!(actual.line_bands.len(), 2);
        assert_eq!(actual.line_bands[1].block_start, S::from_f64(20.0));
        assert_eq!(actual.block_extent, S::from_f64(40.0));
        let wrong = run(3);
        assert_eq!(
            wrong
                .atomics
                .iter()
                .map(|row| (row.item.source_index, row.line_index))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 1)]
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_inline_linear_empty_strut_and_forced_clear_preserve_metrics_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(9.0), S::from_f64(3.0)).unwrap();
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::MAX_CONTENT,
            vec![mixed_forced_line_break(7, flow_axes, metrics, Clear::Both)],
        ));

        assert_eq!(report.inline_extent, S::ZERO);
        assert_eq!(report.block_extent, S::from_f64(24.0));
        assert_eq!(report.first_baseline, Some(S::from_f64(9.0)));
        assert_eq!(report.last_baseline, Some(S::from_f64(21.0)));
        assert!(report.atomics.is_empty());
        assert_eq!(report.controls.len(), 1);
        assert_eq!(report.controls[0].source_index, 7);
        assert_eq!(report.controls[0].line_index, 0);
        assert_eq!(report.controls[0].visual_index, None);
        assert_eq!(
            report.post_line_clear_intents,
            [PostLineClearIntent::Both, PostLineClearIntent::None]
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn supplied_edges_and_real_atomic_overflow_preserve_discard_and_endpoint_roles_both_scalars() {
    use surgeist_text::{SuppliedInlineParticipant, SuppliedLine};
    let slots = [
        SuppliedInlineParticipant::Text {
            level: BidiLevel::try_new(0).unwrap(),
            edge: InlineWhitespaceEdge::DiscardAtLineStart,
        },
        SuppliedInlineParticipant::Text {
            level: BidiLevel::try_new(0).unwrap(),
            edge: InlineWhitespaceEdge::Preserve,
        },
        SuppliedInlineParticipant::Text {
            level: BidiLevel::try_new(0).unwrap(),
            edge: InlineWhitespaceEdge::DiscardAtLineEnd,
        },
    ];
    let edges = SuppliedLine::new(&slots).edges();
    assert_eq!(
        (0..3)
            .map(|index| edges.is_discarded(index))
            .collect::<Vec<_>>(),
        [Some(true), Some(false), Some(true)]
    );
    fn assert_lane<S: LayoutScalar>() {
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::definite(S::from_f64(15.0)),
            vec![
                mr02_atomic(11, 10.0, 10.0, 8.0, 0, InlineBreakOpportunity::Allowed),
                mr02_atomic(13, 30.0, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            ],
        ));
        assert_eq!(report.atomics.len(), 2);
        assert_eq!(
            (
                report.atomics[0].item.source_index,
                report.atomics[0].line_index,
                report.atomics[0].inline_start
            ),
            (11, 0, S::ZERO)
        );
        assert_eq!(
            (
                report.atomics[1].item.source_index,
                report.atomics[1].line_index,
                report.atomics[1].item.size.width
            ),
            (13, 1, S::from_f64(30.0))
        );
        let endpoint_breaks = layout_mixed_inline_run(mr02_input(
            AvailableOf::definite(S::from_f64(5.0)),
            vec![
                mr02_atomic(30, 5.0, 10.0, 8.0, 0, InlineBreakOpportunity::Allowed),
                mr02_atomic(31, 6.0, 10.0, 8.0, 0, InlineBreakOpportunity::Mandatory),
            ],
        ));
        assert_eq!(endpoint_breaks.atomics.len(), 2);
        assert_eq!(endpoint_breaks.atomics[0].line_index, 0);
        assert_eq!(endpoint_breaks.atomics[1].line_index, 1);
        assert_eq!(endpoint_breaks.line_bands.len(), 3);
        assert_eq!(endpoint_breaks.first_baseline, Some(S::from_f64(8.0)));
        assert_eq!(endpoint_breaks.last_baseline, Some(S::from_f64(28.0)));
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

/// Edge flags remain a Text semantic fact. The real Layout consumer must retain
/// the original association while finalizing only the surviving Ahem glyph.
#[test]
fn actual_prepared_collapse_keeps_original_source_while_suppressing_line_edges_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        use crate::test_support::{
            layout_tree::PublicLayoutTreeOf,
            text::{PreparedParagraph, ahem_style},
        };
        let source = surgeist_text::Source::new("  X  ");
        let style = surgeist_text::Style {
            white_space: surgeist_text::WhiteSpace::Collapse,
            ..ahem_style(10.0, 10.0)
        };
        let paragraph = PreparedParagraph::plain(1, source.clone(), style);
        let origin = paragraph.origin(1);
        assert_eq!(origin.view().local_range(), surgeist_text::Range::new(0, 5));
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1));
        let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
        let batch = compute_layout_with_text(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(15.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap();
        let layout = batch.placed_text()[0].layout();
        assert_eq!(
            layout.source().text().code_units(),
            source.text().code_units()
        );
        assert_eq!(layout.metrics().line_count(), 1);
        // The supplied line owns its 15-unit allocation; the collapsed X
        // retains its independent one-em 10-unit shaping/fit contribution.
        assert_eq!(layout.lines()[0].bounds().size.width, 15.0);
        assert_eq!(
            layout
                .glyph_runs()
                .iter()
                .map(|run| run.advance())
                .sum::<f64>(),
            10.0
        );
        let glyphs: Vec<_> = layout
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs().iter().copied())
            .collect();
        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].range(), surgeist_text::Range::new(2, 3));
        assert_eq!(glyphs[0].advance(), 10.0);
        let source_fragment = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .origin()
                    .as_ref()
                    == Some(&origin)
            })
            .unwrap();
        assert_eq!(
            source_fragment
                .fragment()
                .finalized_fragment()
                .finalized_line()
                .feedback()
                .advance,
            10.0
        );
        assert!(batch.final_inline_fragments().iter().any(|entry| {
            entry
                .fragment()
                .finalized_fragment()
                .view()
                .origin()
                .as_ref()
                == Some(&origin)
        }));
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_mr02_inline_linear_bidi_mixed_sources_preserve_visual_order_and_baselines_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let boundary_metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(6.0), S::from_f64(2.0)).unwrap();
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::MAX_CONTENT,
            vec![
                mr02_atomic(40, 10.0, 10.0, 8.0, 1, InlineBreakOpportunity::Prohibited),
                mixed_boundary(
                    41,
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
                    boundary_metrics,
                ),
                mr02_atomic(42, 5.0, 12.0, 9.0, 2, InlineBreakOpportunity::Prohibited),
                mr02_atomic(43, 7.0, 10.0, 8.0, 1, InlineBreakOpportunity::Prohibited),
            ],
        ));

        assert_eq!(report.inline_extent, S::from_f64(22.0));
        assert_eq!(report.block_extent, S::from_f64(12.0));
        assert_eq!(report.first_baseline, Some(S::from_f64(9.0)));
        assert_eq!(report.last_baseline, Some(S::from_f64(9.0)));
        assert_eq!(report.atomics.len(), 3);
        assert_eq!(report.atomics[0].item.source_index, 40);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(12.0));
        assert_eq!(report.atomics[0].block_start, S::from_f64(1.0));
        assert_eq!(report.atomics[0].visual_index, Some(3));
        assert_eq!(report.atomics[2].item.source_index, 43);
        assert_eq!(report.atomics[2].inline_start, S::ZERO);
        assert_eq!(report.atomics[2].visual_index, Some(0));
        assert_eq!(report.atomics[1].item.source_index, 42);
        assert_eq!(report.atomics[1].inline_start, S::from_f64(7.0));
        assert_eq!(report.atomics[1].visual_index, Some(1));
        assert_eq!(report.controls.len(), 1);
        assert_eq!(report.controls[0].source_index, 41);
        assert_eq!(report.controls[0].inline_start, S::from_f64(12.0));
        assert_eq!(report.controls[0].visual_index, Some(2));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn mr02_intrinsic_extents<S: LayoutScalar>() -> (S, S) {
    // These are genuine numeric boxes. Text replacement semantics are exercised
    // separately through an actual prepared paragraph, never a replacement width.
    let min = layout_mixed_inline_run(mr02_input(
        AvailableOf::MIN_CONTENT,
        vec![
            mr02_atomic(51, 0.1, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            mr02_atomic(52, 0.2, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            mr02_atomic(53, 0.05, 10.0, 8.0, 0, InlineBreakOpportunity::Allowed),
        ],
    ));
    let max = layout_mixed_inline_run(mr02_input(
        AvailableOf::MAX_CONTENT,
        vec![
            mr02_atomic(51, 0.1, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            mr02_atomic(52, 0.2, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            mr02_atomic(53, 0.3, 10.0, 8.0, 0, InlineBreakOpportunity::Mandatory),
        ],
    ));
    (min.inline_extent, max.inline_extent)
}

#[test]
fn fri06_mr02_inline_linear_intrinsic_extents_preserve_scalar_addition_order_both_scalars() {
    let (min_f32, max_f32) = mr02_intrinsic_extents::<f32>();
    let expected_min_f32 = ((0.0_f32 + 0.1) + 0.2) + 0.05;
    let expected_max_f32 = ((0.0_f32 + 0.1) + 0.2) + 0.3;
    assert_eq!(min_f32.to_bits(), expected_min_f32.to_bits());
    assert_eq!(max_f32.to_bits(), expected_max_f32.to_bits());

    let (min_f64, max_f64) = mr02_intrinsic_extents::<f64>();
    let expected_min_f64 = ((0.0_f64 + 0.1) + 0.2) + 0.05;
    let expected_max_f64 = ((0.0_f64 + 0.1) + 0.2) + 0.3;
    assert_eq!(min_f64.to_bits(), expected_min_f64.to_bits());
    assert_eq!(max_f64.to_bits(), expected_max_f64.to_bits());
}

#[test]
fn fri06_mr02_inline_linear_float_band_retry_preserves_queries_and_reselection_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let mut queries = Vec::new();
        let report = layout_mixed_inline_run_with_band_source(
            mr02_input(
                AvailableOf::definite(S::from_f64(100.0)),
                vec![
                    mr02_atomic(60, 30.0, 10.0, 8.0, 0, InlineBreakOpportunity::Allowed),
                    mr02_atomic(61, 30.0, 10.0, 8.0, 0, InlineBreakOpportunity::Allowed),
                    mr02_atomic(62, 30.0, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
                ],
            ),
            |block_start, block_end| {
                queries.push((block_start, block_end));
                if block_start == S::ZERO {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(50.0),
                        inline_end: S::from_f64(50.0),
                        next_transition: Some(S::from_f64(10.0)),
                    }
                } else if block_start == S::from_f64(10.0) {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(20.0),
                        inline_end: S::from_f64(60.0),
                        next_transition: None,
                    }
                } else {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::ZERO,
                        inline_end: S::from_f64(100.0),
                        next_transition: None,
                    }
                }
            },
            |block, _| block,
        );

        assert_eq!(
            queries,
            [
                (S::ZERO, S::from_f64(10.0)),
                (S::from_f64(10.0), S::from_f64(20.0)),
                (S::from_f64(20.0), S::from_f64(30.0)),
            ]
        );
        assert_eq!(report.line_bands.len(), 2);
        assert_eq!(report.atomics[0].item.source_index, 60);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(20.0));
        assert_eq!(report.atomics[0].block_start, S::from_f64(10.0));
        assert_eq!(report.atomics[1].line_index, 1);
        assert_eq!(report.atomics[2].line_index, 1);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn mr02_candidate_scan_work(participant_count: usize) -> usize {
    let participants = (0..participant_count)
        .map(|source_index| {
            mr02_atomic::<f64>(
                source_index,
                1.0,
                10.0,
                8.0,
                0,
                InlineBreakOpportunity::Prohibited,
            )
        })
        .collect();
    reset_inline_candidate_scan_visits();
    let report =
        layout_mixed_inline_run(mr02_input(AvailableOf::definite(1_000_000.0), participants));
    assert_eq!(report.atomics.len(), participant_count);
    inline_candidate_scan_visits()
}

#[test]
fn fri06_mr02_inline_linear_candidate_scan_work_is_bounded_without_timing() {
    let work_64 = mr02_candidate_scan_work(64);
    let work_128 = mr02_candidate_scan_work(128);

    eprintln!("linear candidate scan work: n=64 => {work_64}, n=128 => {work_128}");
    assert_eq!(work_64, 64 * 2);
    assert_eq!(work_128, 128 * 2);
    assert_eq!(work_128, work_64 * 2);
}

fn forced_line_break(source_index: usize, metrics: InlineMetrics) -> InlineParticipant {
    forced_line_break_for(
        source_index,
        WritingMode::HorizontalTb,
        Direction::Ltr,
        metrics,
    )
}

fn forced_line_break_for(
    source_index: usize,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
) -> InlineParticipant {
    InlineParticipant::forced_line_break(crate::inline::ForcedLineBreakControlOf::new(
        source_index,
        crate::inline::InlineFlowOf::new(writing_mode, direction, Available::MAX_CONTENT),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    ))
}

fn inline_boundary_control(
    source_index: usize,
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
    alignment: crate::inline::InlineControlAlignment,
) -> crate::inline::InlineBoundaryControlOf {
    crate::inline::InlineBoundaryControlOf::new(
        source_index,
        kind,
        crate::inline::InlineFlowOf::new(writing_mode, direction, Available::MAX_CONTENT),
        metrics,
        alignment,
    )
}

fn inline_boundary_participant(
    source_index: usize,
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    metrics: InlineMetrics,
) -> InlineParticipant {
    InlineParticipant::inline_boundary(inline_boundary_control(
        source_index,
        kind,
        writing_mode,
        direction,
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
    ))
}

fn mixed_forced_line_break<S: LayoutScalar>(
    source_index: usize,
    flow_axes: FlowAxes,
    metrics: InlineMetricsOf<S>,
    clear: Clear,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::ForcedLineBreak(crate::inline::ForcedLineBreakControlOf::new(
        source_index,
        crate::inline::InlineFlowOf::new(
            flow_axes.writing_mode(),
            flow_axes.direction(),
            AvailableOf::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        clear,
    ))
}

fn fri06_c12_t07_endpoint_break_private_line_membership() -> (usize, usize, usize) {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let metrics = InlineMetrics::from_ascent_descent(8.0, 2.0).unwrap();
    let report = layout_mixed_inline_run(mr02_input(
        Available::MAX_CONTENT,
        vec![
            mr02_atomic(0, 20.0, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            mixed_forced_line_break(1, flow_axes, metrics, Clear::None),
            mr02_atomic(2, 20.0, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
        ],
    ));
    let previous_line = report
        .atomics
        .iter()
        .find(|atomic| atomic.item.source_index == 0)
        .expect("previous atomic must be present")
        .line_index;
    let control_line = report
        .controls
        .iter()
        .find(|control| control.source_index == 1)
        .expect("forced break control must be present")
        .line_index;
    let following_line = report
        .atomics
        .iter()
        .find(|atomic| atomic.item.source_index == 2)
        .expect("following atomic must be present")
        .line_index;
    (previous_line, control_line, following_line)
}

#[test]
fn fri06_c12_t07_endpoint_break_commits_following_atomic_to_next_line() {
    let (previous_line, control_line, following_line) =
        fri06_c12_t07_endpoint_break_private_line_membership();

    assert_eq!(previous_line, 0);
    assert_eq!(control_line, 0);
    assert_eq!(following_line, 1);
}

fn mixed_boundary<S: LayoutScalar>(
    source_index: usize,
    flow_axes: FlowAxes,
    metrics: InlineMetricsOf<S>,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::Boundary(crate::inline::InlineBoundaryControlOf::new(
        source_index,
        InlineBoundaryKind::Start,
        crate::inline::InlineFlowOf::new(
            flow_axes.writing_mode(),
            flow_axes.direction(),
            AvailableOf::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
    ))
}

#[test]
fn fri06_c03_clear_private_builder_maps_none_start_end_and_both_in_all_flows() {
    let writing_modes = [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ];
    for writing_mode in writing_modes {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::None),
                PostLineClearIntent::None
            );
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::Left),
                PostLineClearIntent::LineStart
            );
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::Right),
                PostLineClearIntent::LineEnd
            );
            assert_eq!(
                mapped_post_line_clear_intent(flow_axes, Clear::Both),
                PostLineClearIntent::Both
            );
        }
    }
}

#[test]
fn fri06_c03_strut_private_builder_retains_control_baselines_and_post_line_intent_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let boundary_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(18.0), S::from_f64(13.0))
                .unwrap();
        let break_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let report = layout_mixed_inline_run(MixedInlineRunInputOf {
            available_inline_extent: AvailableOf::MAX_CONTENT,
            flow_axes,
            text_align: TextAlign::Start,
            text_align_last: crate::TextAlignLast::Auto,
            participants: vec![
                mixed_boundary(0, flow_axes, boundary_metrics),
                mixed_forced_line_break(1, flow_axes, break_metrics, Clear::Both),
            ],
        });

        assert_eq!(report.block_extent, S::from_f64(40.0));
        assert_eq!(report.first_baseline, Some(S::from_f64(15.0)));
        assert_eq!(report.last_baseline, Some(S::from_f64(35.0)));
        assert_eq!(
            report.post_line_clear_intents,
            [PostLineClearIntent::Both, PostLineClearIntent::None]
        );
        assert_eq!(report.controls[0].visual_index, Some(0));
        assert_eq!(report.controls[1].visual_index, None);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_private_callback_retries_transition_and_reselects_same_cursor_both_scalars()
{
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let participant = |source_index, following_break| MixedInlineParticipantOf::Atomic {
            item: AtomicInlineBoxParticipant {
                source_index,
                size: Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                content_size: Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                margin: Edges::ZERO,
                padding: Edges::ZERO,
                border: Edges::ZERO,
                scrollbar_size: Size::ZERO,
                first_baseline: Some(S::from_f64(8.0)),
                alignment: InlineControlAlignment::Baseline,
            },
            participation: AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                following_break,
            ),
        };
        let mut queries = Vec::new();
        let report = layout_mixed_inline_run_with_band_source(
            MixedInlineRunInputOf {
                available_inline_extent: AvailableOf::definite(S::from_f64(100.0)),
                flow_axes,
                text_align: TextAlign::Start,
                text_align_last: crate::TextAlignLast::Auto,
                participants: vec![
                    participant(0, InlineBreakOpportunity::Allowed),
                    participant(1, InlineBreakOpportunity::Allowed),
                    participant(2, InlineBreakOpportunity::Prohibited),
                ],
            },
            |block_start, block_end| {
                queries.push((block_start, block_end));
                if block_start == S::ZERO {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(50.0),
                        inline_end: S::from_f64(50.0),
                        next_transition: Some(S::from_f64(10.0)),
                    }
                } else if block_start == S::from_f64(10.0) {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::from_f64(20.0),
                        inline_end: S::from_f64(60.0),
                        next_transition: None,
                    }
                } else {
                    LogicalLineBandQueryResultOf {
                        inline_start: S::ZERO,
                        inline_end: S::from_f64(100.0),
                        next_transition: None,
                    }
                }
            },
            |block, _| block,
        );

        assert_eq!(
            queries,
            [
                (S::ZERO, S::from_f64(10.0)),
                (S::from_f64(10.0), S::from_f64(20.0)),
                (S::from_f64(20.0), S::from_f64(30.0)),
            ]
        );
        assert_eq!(report.atomics.len(), 3);
        assert_eq!(report.atomics[0].item.source_index, 0);
        assert_eq!(report.atomics[0].line_index, 0);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(20.0));
        assert_eq!(report.atomics[0].block_start, S::from_f64(10.0));
        assert_eq!(report.atomics[1].item.source_index, 1);
        assert_eq!(report.atomics[1].line_index, 1);
        assert_eq!(report.atomics[2].item.source_index, 2);
        assert_eq!(report.atomics[2].line_index, 1);
        assert_eq!(report.line_bands.len(), 2);
        assert_eq!(report.line_bands[0].block_start, S::from_f64(10.0));
        assert_eq!(report.line_bands[0].inline_start, S::from_f64(20.0));
        assert_eq!(report.line_bands[0].inline_end, S::from_f64(60.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn inline_axis_mapping_maps_horizontal_tb_ltr() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);

    assert_eq!(
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(30.0, 12.0)),
        Size::new(30.0, 12.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(30.0, 80.0),
        ),
        Point::new(5.0, 7.0)
    );
}

#[test]
fn inline_axis_mapping_maps_horizontal_tb_rtl() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);

    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(30.0, 80.0),
        ),
        Point::new(15.0, 7.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_rl_ltr() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);

    assert_eq!(
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(30.0, 12.0)),
        Size::new(12.0, 30.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(69.0, 5.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_rl_rtl() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);

    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(69.0, 15.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_lr_ltr() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);

    assert_eq!(
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(30.0, 12.0)),
        Size::new(12.0, 30.0)
    );
    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(7.0, 5.0)
    );
}

#[test]
fn inline_axis_mapping_maps_vertical_lr_rtl() {
    let flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);

    assert_eq!(
        flow_axes.physical_point(
            crate::geometry::LogicalPointOf::new(5.0, 7.0),
            crate::geometry::LogicalSizeOf::new(10.0, 4.0),
            Size::new(80.0, 30.0),
        ),
        Point::new(7.0, 15.0)
    );
}

#[test]
fn atomic_inline_line_aligns_items_to_max_baseline() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(200.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(7.0)),
            InlineParticipant::new(1, Size::new(10.0, 20.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 5.0));
    assert_eq!(report.items[1].location, Point::new(20.0, 0.0));
}

#[test]
fn atomic_inline_items_wrap_between_items_for_definite_width() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(25.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_min_content_available_wraps_to_max_item_advance() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MIN_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::new(1, Size::new(60.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::new(2, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(60.0, 30.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(30.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 20.0));
}

#[test]
fn atomic_inline_intrinsic_widths_use_max_item_and_sum() {
    let items = vec![
        InlineParticipant::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        InlineParticipant::new(
            1,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        InlineParticipant::new(2, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(inline_run_min_content_width(&items), 110.0);
    assert_eq!(inline_run_max_content_width(&items), 195.0);
}

#[test]
fn atomic_inline_horizontal_rtl_maps_item_origins_in_report() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Rtl,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            forced_line_break(1, metrics),
            InlineParticipant::new(2, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.items[0].location, Point::new(10.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(10.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 10.0));
}

#[test]
fn forced_line_break_control_preserves_layout_ready_fields() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let control = crate::inline::ForcedLineBreakControlOf::new(
        7,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Rtl,
            Available::definite(320.0),
        ),
        metrics,
        crate::inline::InlineControlAlignment::Top,
        Clear::Both,
    );

    assert_eq!(control.source_index(), 7);
    assert_eq!(control.flow().writing_mode(), WritingMode::HorizontalTb);
    assert_eq!(control.flow().direction(), Direction::Rtl);
    assert_eq!(
        control.flow().available_inline_extent(),
        Available::definite(320.0)
    );
    assert_eq!(control.metrics(), metrics);
    assert_eq!(
        control.alignment(),
        crate::inline::InlineControlAlignment::Top
    );
    assert_eq!(control.clear(), Clear::Both);
}

#[test]
fn inline_boundary_control_preserves_layout_ready_fields() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let control = inline_boundary_control(
        9,
        InlineBoundaryKind::Start,
        WritingMode::VerticalRl,
        Direction::Rtl,
        metrics,
        crate::inline::InlineControlAlignment::Top,
    );

    assert_eq!(control.source_index(), 9);
    assert_eq!(control.kind(), InlineBoundaryKind::Start);
    assert_eq!(control.flow().writing_mode(), WritingMode::VerticalRl);
    assert_eq!(control.flow().direction(), Direction::Rtl);
    assert_eq!(
        control.flow().available_inline_extent(),
        Available::MAX_CONTENT
    );
    assert_eq!(control.metrics(), metrics);
    assert_eq!(
        control.alignment(),
        crate::inline::InlineControlAlignment::Top
    );
    assert_eq!(
        InlineParticipant::inline_boundary(control),
        InlineParticipant::Boundary(control)
    );
}

#[test]
fn inline_boundaries_expand_horizontal_line_metrics_without_advance() {
    let start_metrics = InlineMetrics::from_line_height_and_baseline(10.0, 8.0).unwrap();
    let end_metrics = InlineMetrics::from_line_height_and_baseline(30.0, 20.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            inline_boundary_participant(
                0,
                InlineBoundaryKind::Start,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                start_metrics,
            ),
            InlineParticipant::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(8.0)),
            inline_boundary_participant(
                2,
                InlineBoundaryKind::End,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                end_metrics,
            ),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 30.0));
    assert_eq!(report.first_baseline, Some(20.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(
        report.items[0].kind,
        InlineParticipantLayoutKind::InlineBoundaryStart
    );
    assert_eq!(report.items[0].location, Point::new(0.0, 20.0));
    assert_eq!(report.items[0].size, Size::ZERO);
    assert_eq!(report.items[1].kind, InlineParticipantLayoutKind::Box);
    assert_eq!(report.items[1].location, Point::new(0.0, 12.0));
    assert_eq!(
        report.items[2].kind,
        InlineParticipantLayoutKind::InlineBoundaryEnd
    );
    assert_eq!(report.items[2].location, Point::new(20.0, 20.0));
    assert_eq!(report.items[2].size, Size::ZERO);
}

#[test]
fn inline_boundaries_before_overwide_first_box_do_not_create_leading_line() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(50.0, 35.0).unwrap();
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(20.0),
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            inline_boundary_participant(
                0,
                InlineBoundaryKind::Start,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                boundary_metrics,
            ),
            InlineParticipant::new(1, Size::new(40.0, 10.0), Edges::ZERO, Some(8.0)),
        ],
    });

    assert_eq!(report.size, Size::new(40.0, 50.0));
    assert_eq!(report.first_baseline, Some(35.0));
    assert_eq!(report.last_baseline, Some(35.0));
    assert_eq!(
        report.items[0].kind,
        InlineParticipantLayoutKind::InlineBoundaryStart
    );
    assert_eq!(report.items[0].location, Point::new(0.0, 35.0));
    assert_eq!(report.items[1].kind, InlineParticipantLayoutKind::Box);
    assert_eq!(report.items[1].location, Point::new(0.0, 27.0));
}

#[test]
fn forced_line_break_control_can_be_used_as_atomic_inline_item() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 8.0).unwrap();
    let control = crate::inline::ForcedLineBreakControlOf::new(
        1,
        crate::inline::InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            Available::MAX_CONTENT,
        ),
        metrics,
        crate::inline::InlineControlAlignment::Baseline,
        Clear::None,
    );

    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            InlineParticipant::forced_line_break(control),
            InlineParticipant::new(2, Size::new(15.0, 12.0), Edges::ZERO, Some(8.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 24.0));
    assert_eq!(
        report.items[1].kind,
        InlineParticipantLayoutKind::ForcedLineBreak
    );
    assert_eq!(report.items[1].source_index, 1);
    assert_eq!(report.items[1].location, Point::new(20.0, 10.0));
    assert_eq!(report.items[1].size, Size::ZERO);
}

#[test]
fn atomic_inline_intrinsic_widths_split_at_forced_line_breaks() {
    let items = vec![
        InlineParticipant::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        forced_line_break(1, InlineMetrics::default()),
        InlineParticipant::new(
            2,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        InlineParticipant::new(3, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
        forced_line_break(4, InlineMetrics::default()),
    ];

    assert_eq!(inline_run_min_content_width(&items), 110.0);
    assert_eq!(inline_run_max_content_width(&items), 160.0);
}

#[test]
fn atomic_inline_vertical_margins_participate_in_line_metrics() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: vec![InlineParticipant::new(
            0,
            Size::new(20.0, 10.0),
            Edges::new(3.0, 0.0, 7.0, 0.0),
            Some(6.0),
        )],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(9.0));
    assert_eq!(report.last_baseline, Some(9.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 3.0));
}

#[test]
fn atomic_inline_sideways_lr_ltr_maps_inline_progression_bottom_to_top() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::definite(70.0),
        writing_mode: WritingMode::SidewaysLr,
        direction: Direction::Ltr,
        items: vec![
            InlineParticipant::new(0, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
            InlineParticipant::new(1, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
        ],
    });

    assert_eq!(report.items[0].location, Point::new(0.0, 20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 0.0));
}

#[test]
fn atomic_inline_empty_items_report_zero_size_and_no_baselines() {
    let report = layout_inline_run(InlineRunInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: Vec::new(),
    });

    assert_eq!(report.size, Size::ZERO);
    assert_eq!(report.content_size, Size::ZERO);
    assert_eq!(report.first_baseline, None);
    assert_eq!(report.last_baseline, None);
    assert!(report.items.is_empty());
}

mod root_oracle {
    use crate::test_support::oracle::{grid, inline};

    #[test]
    fn oracle_atomic_inline_item_metrics_include_margins_and_baseline() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges {
                top: 2.0,
                right: 3.0,
                bottom: 4.0,
                left: 5.0,
            },
            first_baseline: Some(7.0),
        };

        let metrics = inline::AtomicInlineMetrics::from_item(item);

        assert_eq!(metrics.advance, 28.0);
        assert_eq!(metrics.baseline, 9.0);
        assert_eq!(metrics.descent, 7.0);
        assert_eq!(metrics.margin_box_size, inline::InlineSize::new(28.0, 16.0));
    }

    #[test]
    fn oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: None,
        };

        let metrics = inline::AtomicInlineMetrics::from_item(item);

        assert_eq!(metrics.baseline, 10.0);
        assert_eq!(metrics.descent, 0.0);
        assert!(metrics.synthesized_baseline);
    }

    #[test]
    fn oracle_atomic_inline_item_rejects_baseline_before_top_edge() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(-1.0),
        };

        assert_eq!(
            inline::AtomicInlineMetrics::try_from_item(item),
            Err(inline::AtomicInlineError::BaselineOutOfRange {
                id: "a",
                first_baseline: -1.0,
                height: 10.0,
            })
        );
    }

    #[test]
    fn oracle_atomic_inline_item_rejects_baseline_after_bottom_edge() {
        let item = inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(11.0),
        };

        assert_eq!(
            inline::AtomicInlineMetrics::try_from_item(item),
            Err(inline::AtomicInlineError::BaselineOutOfRange {
                id: "a",
                first_baseline: 11.0,
                height: 10.0,
            })
        );
    }

    #[test]
    fn oracle_atomic_inline_line_aligns_items_to_max_baseline() {
        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(200.0),
            items: vec![
                inline::AtomicInlineItemFacts {
                    id: "short",
                    size: inline::InlineSize::new(20.0, 10.0),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: Some(7.0),
                },
                inline::AtomicInlineItemFacts {
                    id: "tall",
                    size: inline::InlineSize::new(10.0, 20.0),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: Some(12.0),
                },
            ],
        });

        assert_eq!(report.size, inline::InlineSize::new(30.0, 20.0));
        assert_eq!(report.first_baseline, Some(12.0));
        assert_eq!(report.last_baseline, Some(12.0));
        assert_eq!(report.lines.len(), 1);
        assert_eq!(
            report.lines[0],
            inline::AtomicInlineLine {
                start_item: 0,
                end_item: 2,
                y: 0.0,
                width: 30.0,
                height: 20.0,
                baseline: 12.0,
                descent: 8.0,
            }
        );
        assert_eq!(report.items[0].id, "short");
        assert_eq!(report.items[0].location, inline::InlinePoint::new(0.0, 5.0));
        assert_eq!(
            report.items[1].location,
            inline::InlinePoint::new(20.0, 0.0)
        );
    }

    #[test]
    fn oracle_atomic_inline_line_positions_margin_boxes_and_border_boxes() {
        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(200.0),
            items: vec![
                inline::AtomicInlineItemFacts {
                    id: "a",
                    size: inline::InlineSize::new(20.0, 10.0),
                    margin: inline::InlineEdges {
                        top: 2.0,
                        right: 3.0,
                        bottom: 4.0,
                        left: 5.0,
                    },
                    first_baseline: Some(7.0),
                },
                inline::AtomicInlineItemFacts {
                    id: "b",
                    size: inline::InlineSize::new(10.0, 20.0),
                    margin: inline::InlineEdges {
                        top: 1.0,
                        right: 2.0,
                        bottom: 3.0,
                        left: 4.0,
                    },
                    first_baseline: Some(12.0),
                },
            ],
        });

        assert_eq!(report.size, inline::InlineSize::new(44.0, 24.0));
        assert_eq!(
            report.lines[0],
            inline::AtomicInlineLine {
                start_item: 0,
                end_item: 2,
                y: 0.0,
                width: 44.0,
                height: 24.0,
                baseline: 13.0,
                descent: 11.0,
            }
        );
        assert_eq!(report.items[0].location, inline::InlinePoint::new(5.0, 6.0));
        assert_eq!(
            report.items[1].location,
            inline::InlinePoint::new(32.0, 1.0)
        );
    }

    #[test]
    fn oracle_atomic_inline_wraps_between_items_for_definite_width() {
        let item = |id| inline::AtomicInlineItemFacts {
            id,
            size: inline::InlineSize::new(20.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        };

        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(25.0),
            items: vec![item("a"), item("b")],
        });

        assert_eq!(report.size, inline::InlineSize::new(20.0, 20.0));
        assert_eq!(report.lines.len(), 2);
        assert_eq!(
            report.lines[0],
            inline::AtomicInlineLine {
                start_item: 0,
                end_item: 1,
                y: 0.0,
                width: 20.0,
                height: 10.0,
                baseline: 10.0,
                descent: 0.0,
            }
        );
        assert_eq!(
            report.lines[1],
            inline::AtomicInlineLine {
                start_item: 1,
                end_item: 2,
                y: 10.0,
                width: 20.0,
                height: 10.0,
                baseline: 10.0,
                descent: 0.0,
            }
        );
        assert_eq!(report.items[0].location, inline::InlinePoint::new(0.0, 0.0));
        assert_eq!(
            report.items[1].location,
            inline::InlinePoint::new(0.0, 10.0)
        );
        assert_eq!(report.first_baseline, Some(10.0));
        assert_eq!(report.last_baseline, Some(20.0));
    }

    #[test]
    fn oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum() {
        let items = vec![
            inline::AtomicInlineItemFacts {
                id: "a",
                size: inline::InlineSize::new(25.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(10.0),
            },
            inline::AtomicInlineItemFacts {
                id: "b",
                size: inline::InlineSize::new(100.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(10.0),
            },
            inline::AtomicInlineItemFacts {
                id: "c",
                size: inline::InlineSize::new(95.0, 10.0),
                margin: inline::InlineEdges {
                    left: 10.0,
                    right: 7.0,
                    ..inline::InlineEdges::ZERO
                },
                first_baseline: Some(10.0),
            },
        ];

        assert_eq!(inline::atomic_inline_min_content_width(&items), 112.0);
        assert_eq!(inline::atomic_inline_max_content_width(&items), 237.0);
    }

    #[test]
    fn oracle_atomic_inline_min_content_wraps_at_max_item_advance() {
        let items = vec![
            inline::AtomicInlineItemFacts {
                id: "wide",
                size: inline::InlineSize::new(95.0, 10.0),
                margin: inline::InlineEdges {
                    left: 10.0,
                    right: 7.0,
                    ..inline::InlineEdges::ZERO
                },
                first_baseline: Some(10.0),
            },
            inline::AtomicInlineItemFacts {
                id: "next",
                size: inline::InlineSize::new(50.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(10.0),
            },
        ];

        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::MinContent,
            items,
        });

        assert_eq!(report.size, inline::InlineSize::new(112.0, 20.0));
        assert_eq!(report.lines.len(), 2);
        assert_eq!(report.lines[0].width, 112.0);
        assert_eq!(report.lines[1].width, 50.0);
    }

    #[test]
    fn oracle_atomic_inline_too_wide_item_overflows_without_empty_line() {
        let item = |id, width| inline::AtomicInlineItemFacts {
            id,
            size: inline::InlineSize::new(width, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        };

        let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(25.0),
            items: vec![item("wide", 40.0), item("next", 10.0)],
        });

        assert_eq!(report.size, inline::InlineSize::new(40.0, 20.0));
        assert_eq!(report.lines.len(), 2);
        assert_eq!(report.lines[0].start_item, 0);
        assert_eq!(report.lines[0].end_item, 1);
        assert_eq!(report.lines[0].width, 40.0);
        assert_eq!(report.lines[1].start_item, 1);
        assert_eq!(report.lines[1].end_item, 2);
        assert_eq!(report.lines[1].width, 10.0);
    }

    #[test]
    fn oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles() {
        let cases = [
            (
                inline::InlineOuterDisplay::Block,
                inline::InnerFormattingContext::Block,
            ),
            (
                inline::InlineOuterDisplay::Grid,
                inline::InnerFormattingContext::Grid,
            ),
            (
                inline::InlineOuterDisplay::GridLanes,
                inline::InnerFormattingContext::GridLanes,
            ),
        ];

        for (outer_display, inner_context) in cases {
            let wrapper = inline::AtomicInlineWrapperFacts::new(
                "wrapper",
                outer_display,
                inline::InlineSize::new(40.0, 20.0),
                inline::InlineEdges::ZERO,
                Some(15.0),
            );

            assert_eq!(wrapper.outer_display, outer_display);
            assert_eq!(wrapper.inner_context(), inner_context);
            assert_eq!(
                wrapper.as_item(),
                inline::AtomicInlineItemFacts {
                    id: "wrapper",
                    size: inline::InlineSize::new(40.0, 20.0),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: Some(15.0),
                }
            );
        }
    }

    #[test]
    fn oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins() {
        let cases = [
            inline::InlineOuterDisplay::Block,
            inline::InlineOuterDisplay::Grid,
            inline::InlineOuterDisplay::GridLanes,
        ];

        for outer_display in cases {
            let wrapper = inline::AtomicInlineWrapperFacts::new(
                "wrapper",
                outer_display,
                inline::InlineSize::new(40.0, 20.0),
                inline::InlineEdges {
                    top: 2.0,
                    right: 3.0,
                    bottom: 4.0,
                    left: 5.0,
                },
                Some(15.0),
            );

            let metrics = inline::AtomicInlineMetrics::from_item(wrapper.as_item());

            assert_eq!(metrics.advance, 48.0);
            assert_eq!(metrics.baseline, 17.0);
            assert_eq!(metrics.descent, 9.0);
            assert_eq!(metrics.margin_box_size, inline::InlineSize::new(48.0, 26.0));
        }
    }

    #[test]
    fn oracle_atomic_inline_wrapper_produces_grid_contribution_facts() {
        let wrapper = inline::AtomicInlineWrapperFacts::new(
            "inline-grid",
            inline::InlineOuterDisplay::Grid,
            inline::InlineSize::new(80.0, 30.0),
            inline::InlineEdges::ZERO,
            Some(24.0),
        );

        let contribution = inline::atomic_inline_grid_item_facts(
            wrapper,
            grid::GridArea::new(1, 1, 1, 1),
            80.0,
            80.0,
        );

        assert_eq!(contribution.id, "inline-grid");
        assert_eq!(contribution.outer_display, inline::InlineOuterDisplay::Grid);
        assert_eq!(
            contribution.inner_context(),
            inline::InnerFormattingContext::Grid
        );
        assert_eq!(contribution.item.area, grid::GridArea::new(1, 1, 1, 1));
        assert_eq!(contribution.item.min_content, 80.0);
        assert_eq!(contribution.item.max_content, 80.0);
        assert_eq!(
            contribution.item.preferred,
            grid::ContributionSize::Definite(80.0)
        );
        assert_eq!(contribution.item.margin_before, 0.0);
        assert_eq!(contribution.item.margin_after, 0.0);
        assert_eq!(contribution.item.contributions().max_content, 80.0);
    }

    #[test]
    fn oracle_atomic_inline_grid_lanes_contribution_preserves_margins() {
        let wrapper = inline::AtomicInlineWrapperFacts::new(
            "inline-grid-lanes",
            inline::InlineOuterDisplay::GridLanes,
            inline::InlineSize::new(80.0, 30.0),
            inline::InlineEdges {
                left: 5.0,
                right: 7.0,
                ..inline::InlineEdges::ZERO
            },
            Some(24.0),
        );

        let contribution = inline::atomic_inline_grid_item_facts(
            wrapper,
            grid::GridArea::new(1, 1, 1, 1),
            60.0,
            80.0,
        );

        assert_eq!(contribution.id, "inline-grid-lanes");
        assert_eq!(
            contribution.outer_display,
            inline::InlineOuterDisplay::GridLanes
        );
        assert_eq!(
            contribution.inner_context(),
            inline::InnerFormattingContext::GridLanes
        );
        assert_eq!(contribution.item.margin_before, 5.0);
        assert_eq!(contribution.item.margin_after, 7.0);
        assert_eq!(contribution.item.contributions().min_content, 72.0);
        assert_eq!(contribution.item.contributions().max_content, 92.0);
    }
}

mod root_layout_oracle {
    use crate::test_support::layout_tree::OracleTree;
    use crate::test_support::oracle::inline;
    use crate::{
        AtomicInlineParticipation, Available, BidiLevel, Display, InlineBreakOpportunity,
        NodeInput, PreferredSize, Size, TrackComponent, compute_root, round_layout,
    };

    fn assert_atomic_inline_layout_matches_oracle(display: Display) {
        let item_sizes = [
            ("first", Size::new(20.0, 10.0)),
            ("second", Size::new(30.0, 20.0)),
            ("third", Size::new(15.0, 10.0)),
        ];
        let mut tree = OracleTree::new().children(0, [1, 2, 3]).style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

        for (node, (_, size)) in (1_u32..).zip(item_sizes) {
            tree = tree.style(node, atomic_inline_node_input(display, size));
        }

        compute_root(
            &mut tree.borrow_compute(),
            0,
            Size::splat(Available::definite(50.0)),
        )
        .unwrap();
        round_layout(&mut tree.borrow_compute(), 0).unwrap();

        let expected = inline::layout_atomic_inline(inline::AtomicInlineInput {
            available_width: inline::InlineAvailable::Definite(50.0),
            items: item_sizes
                .into_iter()
                .map(|(id, size)| inline::AtomicInlineItemFacts {
                    id,
                    size: inline::InlineSize::new(size.width, size.height),
                    margin: inline::InlineEdges::ZERO,
                    first_baseline: None,
                })
                .collect(),
        });

        let root = tree.final_layout(0).expect("root layout");
        assert_layout_close(root.size.width, 50.0, "root width");
        assert_layout_close(root.size.height, expected.size.height, "root height");

        for (index, expected_item) in expected.items.iter().enumerate() {
            let node = (index + 1) as u32;
            let actual = tree.final_layout(node).expect("child layout");
            assert_layout_close(
                actual.location.x,
                expected_item.location.x,
                &format!("node {node} x"),
            );
            assert_layout_close(
                actual.location.y,
                expected_item.location.y,
                &format!("node {node} y"),
            );
            assert_layout_close(
                actual.size.width,
                expected_item.size.width,
                &format!("node {node} width"),
            );
            assert_layout_close(
                actual.size.height,
                expected_item.size.height,
                &format!("node {node} height"),
            );
        }
    }

    fn atomic_inline_node_input(display: Display, size: Size<f32>) -> NodeInput {
        let atomic_inline_participation = Some(AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            InlineBreakOpportunity::Allowed,
        ));
        match display.inner_display() {
            Display::Grid => NodeInput {
                display,
                grid_template_columns: vec![TrackComponent::px(size.width)],
                grid_template_rows: vec![TrackComponent::px(size.height)],
                atomic_inline_participation,
                ..NodeInput::DEFAULT
            },
            Display::GridLanes => NodeInput {
                display,
                grid_template_columns: vec![TrackComponent::px(size.width)],
                grid_template_rows: vec![TrackComponent::px(size.height)],
                atomic_inline_participation,
                ..NodeInput::DEFAULT
            },
            _ => NodeInput {
                display,
                size: Size::new(
                    PreferredSize::px(size.width),
                    PreferredSize::px(size.height),
                ),
                atomic_inline_participation,
                ..NodeInput::DEFAULT
            },
        }
    }

    fn assert_layout_close(actual: f32, expected: f32, label: &str) {
        assert!(
            (actual - expected).abs() <= 0.000_1,
            "{label}: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn oracle_layout_inline_block_line_matches_layout() {
        assert_atomic_inline_layout_matches_oracle(Display::InlineBlock);
    }

    #[test]
    fn oracle_layout_inline_grid_line_matches_layout() {
        assert_atomic_inline_layout_matches_oracle(Display::InlineGrid);
    }

    #[test]
    fn oracle_layout_inline_grid_lanes_line_matches_layout() {
        assert_atomic_inline_layout_matches_oracle(Display::InlineGridLanes);
    }
}

#[test]
fn resolved_bidi_level_126_preserves_l2_mixed_line_order_both_scalars() {
    // UAX #9 rev52 distinguishes explicit levels through 125 from resolved
    // levels through 126 after I1/I2. This carrier feeds real L2 reordering.
    for level in 0..=125 {
        assert_eq!(BidiLevel::try_new(level).unwrap().get(), level);
    }
    for level in 127..=u8::MAX {
        assert_eq!(
            BidiLevel::try_new(level),
            Err(BidiLevelError::OutOfRange { level })
        );
    }
    let resolved = BidiLevel::try_new(126).expect("implicit resolution can produce level 126");
    assert_eq!(resolved.get(), 126);
    assert!(!resolved.is_rtl());

    fn assert_line<S: LayoutScalar>() {
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::<S>::MAX_CONTENT,
            vec![
                mr02_atomic(0, 3.0, 10.0, 8.0, 125, InlineBreakOpportunity::Prohibited),
                mr02_atomic(1, 5.0, 10.0, 8.0, 126, InlineBreakOpportunity::Prohibited),
                mr02_atomic(2, 7.0, 10.0, 8.0, 125, InlineBreakOpportunity::Prohibited),
            ],
        ));
        // L2 first reverses the singleton 126, then the whole 125-or-higher run.
        // Visual order is therefore [2,1,0], independently of scalar precision.
        let mut visual: Vec<_> = report
            .atomics
            .iter()
            .map(|atomic| (atomic.visual_index, atomic.item.source_index))
            .collect();
        visual.sort_by_key(|&(index, _)| index);
        assert_eq!(visual, [(Some(0), 2), (Some(1), 1), (Some(2), 0)]);
        assert_eq!(report.inline_extent, S::from_f64(15.0));
        assert_eq!(report.atomics[0].item.source_index, 0);
        assert_eq!(report.atomics[0].inline_start, S::from_f64(12.0));
        assert_eq!(report.atomics[2].item.source_index, 2);
        assert_eq!(report.atomics[2].inline_start, S::ZERO);
        assert_eq!(report.atomics[1].inline_start, S::from_f64(7.0));
    }
    assert_line::<f32>();
    assert_line::<f64>();
}

#[test]
fn supplied_line_projection_and_forced_break_queries_visit_linear_real_formatter_work() {
    use crate::inline::{inline_work_visits, reset_inline_work_visits};
    for count in [64, 128] {
        let participants = (0..count)
            .map(|index| {
                mr02_atomic::<f64>(index, 1.0, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited)
            })
            .collect();
        reset_inline_work_visits();
        reset_inline_candidate_scan_visits();
        let report =
            layout_mixed_inline_run(mr02_input(AvailableOf::definite(1_000_000.0), participants));
        assert_eq!(report.atomics.len(), count);
        assert_eq!(
            report.inline_extent,
            f64::from(u32::try_from(count).unwrap())
        );
        assert_eq!(inline_work_visits(), [count, count, 2]);
        assert_eq!(inline_candidate_scan_visits(), count * 2);

        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let metrics = InlineMetricsOf::<f64>::from_ascent_descent(8.0, 2.0).unwrap();
        let participants = (0..count)
            .flat_map(|index| {
                [
                    mixed_boundary(index * 2, axes, metrics),
                    mixed_forced_line_break(index * 2 + 1, axes, metrics, Clear::None),
                ]
            })
            .collect();
        reset_inline_work_visits();
        let report =
            layout_mixed_inline_run(mr02_input(AvailableOf::definite(100.0), participants));
        assert!(report.atomics.is_empty());
        assert_eq!(report.controls.len(), count * 2);
        assert_eq!(report.inline_extent, 0.0);
        assert_eq!(inline_work_visits(), [count * 2, count * 2, count * 2]);
        assert_eq!(report.controls.first().unwrap().source_index, 0);
        assert_eq!(report.controls.last().unwrap().source_index, count * 2 - 1);

        let participants = (0..count)
            .map(|index| {
                mr02_atomic::<f64>(index, 1.0, 10.0, 8.0, 0, InlineBreakOpportunity::Allowed)
            })
            .collect();
        reset_inline_work_visits();
        let report = layout_mixed_inline_run(mr02_input(AvailableOf::definite(1.0), participants));
        assert_eq!(report.atomics.len(), count);
        assert_eq!(report.atomics.last().unwrap().line_index, count - 1);
        assert_eq!(report.atomics.last().unwrap().inline_start, 0.0);
        assert_eq!(inline_work_visits(), [count, count, count * 2]);
    }
}

fn assert_mandatory_atomic_queries_are_confined_to_each_actual_line<S: LayoutScalar>() {
    use crate::inline::{inline_leading_query_units, inline_work_visits, reset_inline_work_visits};
    for count in [4, 8] {
        let participants = (0..count)
            .map(|index| {
                mr02_atomic::<S>(index, 1.0, 10.0, 8.0, 1, InlineBreakOpportunity::Mandatory)
            })
            .collect();
        reset_inline_work_visits();
        reset_inline_candidate_scan_visits();
        let report = layout_mixed_inline_run(mr02_input(
            AvailableOf::definite(S::from_f64(100.0)),
            participants,
        ));
        assert_eq!(report.atomics.len(), count);
        assert_eq!(report.inline_extent, S::ONE);
        assert_eq!(
            report.block_extent,
            S::from_f64(f64::from(u32::try_from(count + 1).unwrap()) * 10.0)
        );
        assert_eq!(report.line_bands.len(), count + 1);
        assert_eq!(report.first_baseline, Some(S::from_f64(8.0)));
        assert_eq!(
            report.last_baseline,
            Some(S::from_f64(
                f64::from(u32::try_from(count).unwrap()) * 10.0 + 8.0
            ))
        );
        assert_eq!(
            report
                .atomics
                .iter()
                .map(|item| item.line_index)
                .collect::<Vec<_>>(),
            (0..count).collect::<Vec<_>>()
        );
        assert_eq!(inline_candidate_scan_visits(), count * 2);
        assert_eq!(inline_work_visits(), [count, count, count * 2]);
        assert_eq!(inline_leading_query_units(), count * 2);
        // Text remains the actual edge owner. A separately supplied line of
        // discardable text slots retains each logical slot while suppressing ink.
        let text = vec![
            surgeist_text::SuppliedInlineParticipant::Text {
                level: BidiLevel::try_new(1).unwrap(),
                edge: InlineWhitespaceEdge::DiscardAtBoth
            };
            count
        ];
        let edges = surgeist_text::SuppliedLine::new(&text).edges();
        assert_eq!(
            surgeist_text::SuppliedLine::new(&text).leading_discard_len(),
            count
        );
        assert!((0..count).all(|index| edges.is_discarded(index) == Some(true)));
    }
}

#[test]
fn mandatory_atomic_line_queries_are_bounded_in_f32() {
    assert_mandatory_atomic_queries_are_confined_to_each_actual_line::<f32>();
}
#[test]
fn mandatory_atomic_line_queries_are_bounded_in_f64() {
    assert_mandatory_atomic_queries_are_confined_to_each_actual_line::<f64>();
}

#[test]
fn numeric_unexpandable_alignment_uses_actual_last_line_policy_both_scalars() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            for (align, expected) in [
                (TextAlign::Start, 0.0),
                (TextAlign::End, 30.0),
                (TextAlign::Left, if rtl { 30.0 } else { 0.0 }),
                (TextAlign::Right, if rtl { 0.0 } else { 30.0 }),
                (TextAlign::Center, 15.0),
            ] {
                let mut input = mr02_input(
                    AvailableOf::definite(S::from_f64(50.0)),
                    vec![mr02_atomic(
                        0,
                        20.0,
                        10.0,
                        8.0,
                        if rtl { 1 } else { 0 },
                        InlineBreakOpportunity::Prohibited,
                    )],
                );
                input.flow_axes = FlowAxes::new(
                    WritingMode::HorizontalTb,
                    if rtl { Direction::Rtl } else { Direction::Ltr },
                );
                input.text_align = align;
                let report = layout_mixed_inline_run(input);
                assert_eq!(
                    report.atomics[0].inline_start,
                    S::from_f64(expected),
                    "{align:?}/{rtl}"
                );
            }
        }

        for (last, expected) in [
            (TextAlignLast::Auto, 0.0),
            (TextAlignLast::End, 30.0),
            (TextAlignLast::Justify, 15.0),
        ] {
            let mut input = mr02_input(
                AvailableOf::definite(S::from_f64(50.0)),
                vec![mr02_atomic(
                    0,
                    20.0,
                    10.0,
                    8.0,
                    0,
                    InlineBreakOpportunity::Prohibited,
                )],
            );
            input.text_align = TextAlign::Justify;
            input.text_align_last = last;
            let report = layout_mixed_inline_run(input);
            assert_eq!(report.atomics[0].inline_start, S::from_f64(expected));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

fn assert_numeric_mandatory_atomic_last_alignment<S: LayoutScalar>() {
    // A mandatory break after the first actual 20px atomic is a hard line
    // ending even though it does not contain a separate break control.
    // A later 10px atomic makes that first hard line nonterminal.
    for (last, first_x, second_x) in [
        (TextAlignLast::Auto, 0.0, 0.0),
        (TextAlignLast::End, 30.0, 40.0),
    ] {
        let mut input = mr02_input(
            AvailableOf::definite(S::from_f64(50.0)),
            vec![
                mr02_atomic(0, 20.0, 10.0, 8.0, 0, InlineBreakOpportunity::Mandatory),
                mr02_atomic(1, 10.0, 10.0, 8.0, 0, InlineBreakOpportunity::Prohibited),
            ],
        );
        input.text_align = TextAlign::Start;
        input.text_align_last = last;
        let report = layout_mixed_inline_run(input);
        assert_eq!(report.atomics.len(), 2);
        assert_eq!(report.line_bands.len(), 2);
        assert_eq!(report.atomics[0].block_start, S::ZERO);
        assert_eq!(report.atomics[1].block_start, S::from_f64(10.0));
        assert_eq!(
            report.atomics[0].inline_start,
            S::from_f64(first_x),
            "nonterminal mandatory line/{last:?}"
        );
        assert_eq!(
            report.atomics[1].inline_start,
            S::from_f64(second_x),
            "paragraph end/{last:?}"
        );
    }
}

#[test]
fn numeric_mandatory_atomic_end_uses_last_alignment_before_more_source_f32() {
    assert_numeric_mandatory_atomic_last_alignment::<f32>();
}

#[test]
fn numeric_mandatory_atomic_end_uses_last_alignment_before_more_source_f64() {
    assert_numeric_mandatory_atomic_last_alignment::<f64>();
}
