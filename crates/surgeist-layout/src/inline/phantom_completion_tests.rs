//! Owning numeric completion tests: genuine participants, providers and state.
use super::*;
use std::cell::{Cell, RefCell};

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn boundary<S: LayoutScalar>(
    source: usize,
    kind: InlineBoundaryKind,
    direction: Direction,
    baseline: f64,
    extent: f64,
    border: f64,
    padding: f64,
) -> InlineBoundaryControlOf<S> {
    InlineBoundaryControlOf::new(
        source,
        kind,
        InlineFlowOf::new(
            WritingMode::HorizontalTb,
            direction,
            AvailableOf::definite(n(60.0)),
        ),
        InlineMetricsOf::try_new(
            crate::InlineBaselineContext::Physical,
            n(baseline),
            n(extent),
        )
        .unwrap(),
        InlineControlAlignment::Baseline,
        InlineBoundaryEdgeOf::try_new(n(border), n(padding)).unwrap(),
    )
}
fn input<S: LayoutScalar>(
    direction: Direction,
    participants: Vec<MixedInlineParticipantOf<S>>,
) -> MixedInlineRunInputOf<S> {
    MixedInlineRunInputOf {
        inline_baseline_context: crate::InlineBaselineContext::Physical,
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Plaintext,
            direction,
        ),
        observe_tail: true,
        wrapping: None,
        available_inline_extent: AvailableOf::definite(n(60.0)),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, direction),
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants,
    }
}
fn phantom<S: LayoutScalar>(direction: Direction) {
    let start = boundary::<S>(
        17,
        InlineBoundaryKind::Start,
        direction,
        8.0,
        10.0,
        0.0,
        0.0,
    );
    let end = boundary::<S>(41, InlineBoundaryKind::End, direction, 8.0, 10.0, 0.0, 0.0);
    let original = input::<S>(
        direction,
        vec![
            MixedInlineParticipantOf::Boundary(start),
            MixedInlineParticipantOf::Boundary(end),
        ],
    );
    for available in [
        AvailableOf::definite(n(60.0)),
        AvailableOf::MIN_CONTENT,
        AvailableOf::MAX_CONTENT,
    ] {
        let mut query = original.clone();
        query.available_inline_extent = available;
        let report = layout_mixed_inline_run_with_band_source(
            query,
            |_, _| panic!("a phantom completion must not request an actual float band"),
            |_, _| panic!("a phantom completion must not request actual clearance"),
        )
        .unwrap();
        assert_eq!(report.inline_extent, S::ZERO);
        assert_eq!(report.block_extent, S::ZERO);
        assert_eq!(report.first_baseline, None);
        assert_eq!(report.last_baseline, None);
        assert_eq!(report.float_edge_phase, None);
        assert!(report.line_bands.is_empty());
        assert!(report.post_line_clear_intents.is_empty());
        assert!(report.lines.is_empty());
        assert!(report.atomics.is_empty());
        assert!(report.controls.is_empty());
        assert_eq!(report.indent_state, original.indent_state);
        assert_eq!(report.line_box_context, original.line_box_context);
        assert_eq!(report.nonformatted_controls.len(), 2);
        for (actual, control) in report.nonformatted_controls.iter().zip([start, end]) {
            // Full original witness includes source index, kind, metrics, edge,
            // alignment and axes; nominal8/10 is retained, never replaced by0.
            assert_eq!(actual.control, control);
            assert_eq!(actual.control.metrics().baseline(), n(8.0));
            assert_eq!(actual.control.metrics().line_extent(), n(10.0));
            assert_eq!(actual.inline_start, S::ZERO);
            assert_eq!(actual.block_start, S::ZERO);
        }
    }
    let mut indented = original.clone();
    indented.indent = crate::text_indent::ResolvedTextIndent {
        amount: n(7.0),
        each_line: true,
        hanging: false,
    };
    let observed = layout_mixed_inline_run_with_formatted_observation(
        indented,
        |_, _| panic!("indent does not authorize a phantom band"),
        |_, _| panic!("indent does not authorize phantom clearance"),
        true,
    )
    .unwrap();
    assert_eq!(observed.inline_extent, S::ZERO);
    assert_eq!(observed.block_extent, S::ZERO);
    assert_eq!(observed.formatted_lines, Some(vec![]));
    assert_eq!(observed.indent_state, original.indent_state);
    // Plaintext's previous weak direction starts absent. A phantom must not
    // publish an accepted weak-line direction merely because resolution exists.
    assert_eq!(observed.line_box_context, original.line_box_context);
}
#[test]
fn phantom_completion_avoids_providers_and_preserves_witnesses_and_formatter_state() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        phantom::<f32>(direction);
        phantom::<f64>(direction);
    }
}

fn zero_atomic<S: LayoutScalar>(direction: Direction) {
    let calls = Cell::new(0);
    let clears = RefCell::new(Vec::new());
    let atomic = AtomicInlineBoxParticipant {
        baseline_reference: crate::InlineBaselineContext::Physical,
        source_index: 37,
        size: Size::ZERO,
        content_size: Size::ZERO,
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_size: Size::ZERO,
        first_baseline: Some(S::ZERO),
        alignment: InlineControlAlignment::Baseline,
    };
    let query = input::<S>(
        direction,
        vec![MixedInlineParticipantOf::Atomic {
            item: atomic,
            participation: AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                InlineBreakOpportunity::Allowed,
            ),
        }],
    );
    let original_indent = query.indent_state;
    let report = layout_mixed_inline_run_with_formatted_observation(
        query,
        |_, _| {
            calls.set(calls.get() + 1);
            LogicalLineBandQueryResultOf {
                inline_start: S::ZERO,
                inline_end: n(60.0),
                next_transition: None,
            }
        },
        |block, intent| {
            clears.borrow_mut().push(intent);
            block
        },
        true,
    )
    .unwrap();
    assert!(
        calls.get() > 0,
        "zero-size InFlow atomic still requires its actual band"
    );
    assert_eq!(*clears.borrow(), [PostLineClearIntent::None]);
    assert_eq!(report.line_bands.len(), 1);
    assert_eq!(report.line_bands[0].block_start, S::ZERO);
    assert_eq!(report.line_bands[0].block_end, S::ZERO);
    assert_eq!(report.first_baseline, Some(S::ZERO));
    assert_eq!(report.last_baseline, Some(S::ZERO));
    assert_eq!(report.block_extent, S::ZERO);
    assert_eq!(report.atomics.len(), 1);
    assert_eq!(report.atomics[0].item, atomic);
    assert_eq!(report.atomics[0].line_index, 0);
    assert!(report.nonformatted_controls.is_empty());
    assert_eq!(report.formatted_lines.as_ref().unwrap().len(), 1);
    assert_eq!(
        report.formatted_lines.as_ref().unwrap()[0].ending,
        surgeist_text::LineEnd::Paragraph
    );
    let mut committed = original_indent;
    committed.commit(false);
    assert_eq!(report.indent_state, committed);
}
#[test]
fn zero_extent_in_flow_atomic_still_uses_real_band_and_commits_one_line() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        zero_atomic::<f32>(direction);
        zero_atomic::<f64>(direction);
    }
}

fn zero_forced<S: LayoutScalar>(direction: Direction) {
    let calls = Cell::new(0);
    let clears = RefCell::new(Vec::new());
    let force = ForcedLineBreakControlOf::new(
        53,
        InlineFlowOf::new(
            WritingMode::HorizontalTb,
            direction,
            AvailableOf::definite(n(60.0)),
        ),
        InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, S::ZERO, S::ZERO).unwrap(),
        InlineControlAlignment::Baseline,
        Clear::Both,
    );
    let query = input::<S>(
        direction,
        vec![MixedInlineParticipantOf::ForcedLineBreak(force)],
    );
    let original_indent = query.indent_state;
    let report = layout_mixed_inline_run_with_formatted_observation(
        query,
        |_, _| {
            calls.set(calls.get() + 1);
            LogicalLineBandQueryResultOf {
                inline_start: S::ZERO,
                inline_end: n(60.0),
                next_transition: None,
            }
        },
        |block, intent| {
            clears.borrow_mut().push(intent);
            block
        },
        true,
    )
    .unwrap();
    assert!(calls.get() > 0);
    // The real Forced ending consumes clearance once. Its empty Paragraph
    // receiver does not inherit Forced, clear or formatted-line authority.
    assert_eq!(*clears.borrow(), [PostLineClearIntent::Both]);
    assert_eq!(report.line_bands.len(), 1);
    assert_eq!(report.first_baseline, Some(S::ZERO));
    assert_eq!(report.last_baseline, Some(S::ZERO));
    assert_eq!(report.block_extent, S::ZERO);
    assert_eq!(report.controls.len(), 1);
    assert_eq!(report.controls[0].source_index, 53);
    assert_eq!(report.controls[0].line_index, 0);
    assert_eq!(
        report.controls[0].kind,
        InlineParticipantLayoutKind::ForcedLineBreak
    );
    assert_eq!(report.formatted_lines.as_ref().unwrap().len(), 1);
    assert_eq!(
        report.formatted_lines.as_ref().unwrap()[0].ending,
        surgeist_text::LineEnd::Forced
    );
    assert_eq!(report.lines.len(), 1);
    assert_eq!(report.lines[0].end, surgeist_text::LineEnd::Forced);
    let mut committed = original_indent;
    committed.commit(true);
    assert_eq!(report.indent_state, committed);
}
#[test]
fn zero_metric_forced_control_demands_band_and_clearance_without_promoting_receiver() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        zero_forced::<f32>(direction);
        zero_forced::<f64>(direction);
    }
}

fn individual_decoration<S: LayoutScalar>(direction: Direction) {
    for (side, border, padding) in [
        (InlineBoundaryKind::Start, 1.0, 0.0),
        (InlineBoundaryKind::Start, 0.0, 2.0),
        (InlineBoundaryKind::End, 1.0, 0.0),
        (InlineBoundaryKind::End, 0.0, 2.0),
    ] {
        let start = boundary::<S>(
            7,
            InlineBoundaryKind::Start,
            direction,
            0.0,
            0.0,
            if side == InlineBoundaryKind::Start {
                border
            } else {
                0.0
            },
            if side == InlineBoundaryKind::Start {
                padding
            } else {
                0.0
            },
        );
        let end = boundary::<S>(
            19,
            InlineBoundaryKind::End,
            direction,
            0.0,
            0.0,
            if side == InlineBoundaryKind::End {
                border
            } else {
                0.0
            },
            if side == InlineBoundaryKind::End {
                padding
            } else {
                0.0
            },
        );
        let calls = Cell::new(0);
        let report = layout_mixed_inline_run_with_band_source(
            input(
                direction,
                vec![
                    MixedInlineParticipantOf::Boundary(start),
                    MixedInlineParticipantOf::Boundary(end),
                ],
            ),
            |_, _| {
                calls.set(calls.get() + 1);
                LogicalLineBandQueryResultOf {
                    inline_start: S::ZERO,
                    inline_end: n(60.0),
                    next_transition: None,
                }
            },
            |block, _| block,
        )
        .unwrap();
        assert!(
            calls.get() > 0,
            "individual BP is content even with zero metric extent"
        );
        assert_eq!(report.inline_extent, n(border + padding));
        assert_eq!(report.block_extent, S::ZERO);
        assert_eq!(report.first_baseline, Some(S::ZERO));
        assert_eq!(report.line_bands.len(), 1);
        assert_eq!(report.controls.len(), 2);
        assert!(
            report
                .controls
                .iter()
                .all(|control| control.line_index == 0)
        );
        assert!(report.nonformatted_controls.is_empty());
    }
}
#[test]
fn each_positive_inline_border_or_padding_uses_real_line_path_at_zero_metric_extent() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        individual_decoration::<f32>(direction);
        individual_decoration::<f64>(direction);
    }
}

fn authentic_mandatory_edge<S: LayoutScalar>(direction: Direction) {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    let start = boundary::<S>(1, InlineBoundaryKind::Start, direction, 8.0, 10.0, 0.0, 0.0);
    let end = boundary::<S>(2, InlineBoundaryKind::End, direction, 8.0, 10.0, 0.0, 0.0);
    let mut tree = PublicLayoutTreeOf::<S>::new().style(
        0,
        crate::NodeInputOf {
            display: crate::Display::Block,
            direction,
            ..crate::NodeInputOf::default()
        },
    );
    for control in [start, end] {
        tree.insert_input(
            control.source_index() as u32,
            crate::LayoutInputOf::inline_boundary(
                crate::InlineBoundaryInputOf::new(
                    control.kind(),
                    control.metrics(),
                    control.edge(),
                )
                .with_direction(direction),
            ),
        );
    }
    tree.insert_children(0, [1, 2]);
    let flow = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![],
        crate::WrappingInput {
            scopes: vec![crate::WrappingScopeInput::NativePair {
                start: 1,
                end: 2,
                wrap_inside: crate::WrapInside::Auto,
                edges: crate::WrapEdges {
                    before: crate::WrapEdge::Auto,
                    after: crate::WrapEdge::Line,
                },
                metric_contribution: None,
            }],
            child_order: vec![],
        },
    )
    .unwrap();
    let wrapping = flow
        .wrapping
        .numeric_projection(&flow.graph, 0, &[1, 2])
        .unwrap();
    let mut query = input::<S>(
        direction,
        vec![
            MixedInlineParticipantOf::Boundary(start),
            MixedInlineParticipantOf::Boundary(end),
        ],
    );
    query.wrapping = Some(wrapping);
    let calls = Cell::new(0);
    let clears = RefCell::new(Vec::new());
    let report = layout_mixed_inline_run_with_formatted_observation(
        query,
        |_, _| {
            calls.set(calls.get() + 1);
            LogicalLineBandQueryResultOf {
                inline_start: S::ZERO,
                inline_end: n(60.0),
                next_transition: None,
            }
        },
        |block, intent| {
            clears.borrow_mut().push(intent);
            block
        },
        true,
    )
    .unwrap();
    assert!(calls.get() > 0);
    assert_eq!(*clears.borrow(), [PostLineClearIntent::None]);
    assert_eq!(report.block_extent, n(10.0));
    assert_eq!(report.first_baseline, Some(n(8.0)));
    assert_eq!(report.line_bands.len(), 1);
    assert_eq!(report.controls.len(), 2);
    assert!(
        report
            .controls
            .iter()
            .all(|control| control.line_index == 0)
    );
    assert_eq!(report.formatted_lines.as_ref().unwrap().len(), 1);
    assert_eq!(
        report.formatted_lines.as_ref().unwrap()[0].ending,
        surgeist_text::LineEnd::Forced
    );
    assert_eq!(report.lines.len(), 1);
    assert_eq!(report.lines[0].end, surgeist_text::LineEnd::Forced);
    let observed = &report.formatted_lines.as_ref().unwrap()[0];
    assert_eq!(observed.source_end, 2);
    assert_eq!(observed.descriptor_end, 2);
}
#[test]
fn authentic_native_pair_after_line_edge_is_formatted_without_a_forced_control() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        authentic_mandatory_edge::<f32>(direction);
        authentic_mandatory_edge::<f64>(direction);
    }
}

fn scoped_metric_witness<S: LayoutScalar>(
    direction: Direction,
    declared_baseline: f64,
    conflicting: bool,
) {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    let start = boundary::<S>(1, InlineBoundaryKind::Start, direction, 8.0, 10.0, 0.0, 0.0);
    let end = boundary::<S>(2, InlineBoundaryKind::End, direction, 8.0, 10.0, 0.0, 0.0);
    let mut tree = PublicLayoutTreeOf::<S>::new().style(
        0,
        crate::NodeInputOf {
            display: crate::Display::Block,
            direction,
            ..crate::NodeInputOf::default()
        },
    );
    for control in [start, end] {
        tree.insert_input(
            control.source_index() as u32,
            crate::LayoutInputOf::inline_boundary(
                crate::InlineBoundaryInputOf::new(
                    control.kind(),
                    control.metrics(),
                    control.edge(),
                )
                .with_direction(direction),
            ),
        );
    }
    tree.insert_children(0, [1, 2]);
    let flow = crate::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![],
        crate::WrappingInput {
            scopes: vec![crate::WrappingScopeInput::NativePair {
                start: 1,
                end: 2,
                wrap_inside: crate::WrapInside::Auto,
                edges: crate::WrapEdges::AUTO,
                metric_contribution: Some(crate::InlineBoxMetricContribution::Strut(
                    surgeist_text::LineBoxStrut {
                        metrics: surgeist_text::LineStrut {
                            baseline: declared_baseline,
                            line_extent: 10.0,
                        },
                        alignment: surgeist_text::LineBoxAlignment::Baseline,
                    },
                )),
            }],
            child_order: vec![],
        },
    )
    .unwrap();
    let wrapping = flow
        .wrapping
        .numeric_projection(&flow.graph, 0, &[1, 2])
        .unwrap();
    let mut query = input::<S>(
        direction,
        vec![
            MixedInlineParticipantOf::Boundary(start),
            MixedInlineParticipantOf::Boundary(end),
        ],
    );
    query.wrapping = Some(wrapping);
    let expected_indent = query.indent_state;
    let expected_direction = query.line_box_context;
    let result = layout_mixed_inline_run_with_formatted_observation(
        query,
        |_, _| panic!("metric validation or phantom completion cannot demand a float band"),
        |_, _| panic!("metric validation or phantom completion cannot demand clearance"),
        true,
    );
    if conflicting {
        assert_eq!(
            result.expect_err("valid9/10 numbers contradict these actual8/10 endpoint witnesses"),
            crate::inline_wrapping::NumericWrappingError::Scope(
                crate::WrappingScopeError::ConflictingBoxMetrics,
            ),
        );
    } else {
        let report = result.expect("a matching actual box metric is valid but is not content");
        assert_eq!(report.inline_extent, S::ZERO);
        assert_eq!(report.block_extent, S::ZERO);
        assert_eq!(report.first_baseline, None);
        assert_eq!(report.last_baseline, None);
        assert_eq!(report.formatted_lines, Some(vec![]));
        assert!(report.line_bands.is_empty());
        assert!(report.lines.is_empty());
        assert!(report.controls.is_empty());
        assert_eq!(report.nonformatted_controls.len(), 2);
        for (actual, original) in report.nonformatted_controls.iter().zip([start, end]) {
            assert_eq!(actual.control, original);
            assert_eq!(actual.inline_start, S::ZERO);
            assert_eq!(actual.block_start, S::ZERO);
        }
        assert_eq!(report.indent_state, expected_indent);
        assert_eq!(report.line_box_context, expected_direction);
    }
}

#[test]
fn matching_actual_scoped_metric_stays_phantom_and_provider_free() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        scoped_metric_witness::<f32>(direction, 8.0, false);
        scoped_metric_witness::<f64>(direction, 8.0, false);
    }
}

#[test]
fn contradictory_actual_scoped_metric_refuses_before_provider_demand() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        scoped_metric_witness::<f32>(direction, 9.0, true);
        scoped_metric_witness::<f64>(direction, 9.0, true);
    }
}
