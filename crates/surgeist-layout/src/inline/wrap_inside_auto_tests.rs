//! Current numeric Auto behavior, before any scope-pair policy is expressible.

use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    InlineBoundaryInputOf, LayoutInputOf, LayoutRootRequestOf, LengthAutoOf, NodeInputOf,
    NodeOutputOf, Point, PreferredSizeOf, compute_layout,
};

fn signed_margin<S: LayoutScalar>(direction: Direction) -> Edges<S> {
    // Physical sides encode logical start +5 and end -5 in each actual flow.
    match direction {
        Direction::Ltr => Edges::new(S::ZERO, S::from_f64(-5.0), S::ZERO, S::from_f64(5.0)),
        Direction::Rtl => Edges::new(S::ZERO, S::from_f64(5.0), S::ZERO, S::from_f64(-5.0)),
    }
}

fn metrics<S: LayoutScalar>() -> InlineMetricsOf<S> {
    InlineMetricsOf::from_line_height_and_baseline(S::from_f64(10.0), S::from_f64(10.0)).unwrap()
}

fn participation(following: InlineBreakOpportunity) -> AtomicInlineParticipation {
    // Preserve the existing native RTL profile: resolved atomic level0 and a
    // separately supplied containing FlowAxes. Do not infer level from direction.
    AtomicInlineParticipation::new(BidiLevel::try_new(0).unwrap(), following)
}

fn numeric_input<S: LayoutScalar>(
    permissions: [InlineBreakOpportunity; 3],
    direction: Direction,
) -> MixedInlineRunInputOf<S> {
    let flow = InlineFlowOf::new(
        WritingMode::HorizontalTb,
        direction,
        AvailableOf::definite(S::from_f64(25.0)),
    );
    let atomic = |source_index, following| MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            source_index,
            size: Size::splat(S::from_f64(10.0)),
            content_size: Size::ZERO,
            margin: signed_margin(direction),
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: Some(S::from_f64(10.0)),
            alignment: InlineControlAlignment::Baseline,
        },
        participation: participation(following),
    };
    let boundary = |source_index, kind| {
        MixedInlineParticipantOf::Boundary(InlineBoundaryControlOf::new(
            source_index,
            kind,
            flow,
            metrics(),
            InlineControlAlignment::Baseline,
            InlineBoundaryEdgeOf::ZERO,
        ))
    };
    MixedInlineRunInputOf {
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            direction,
        ),
        observe_tail: true,
        wrapping: None,
        available_inline_extent: AvailableOf::definite(S::from_f64(25.0)),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, direction),
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants: vec![
            atomic(0, permissions[0]),
            boundary(1, InlineBoundaryKind::Start),
            atomic(2, permissions[1]),
            atomic(3, permissions[2]),
            boundary(4, InlineBoundaryKind::End),
        ],
    }
}

fn source_indices<S: LayoutScalar>(line: &SelectedInlineLineOf<S>) -> Vec<usize> {
    line.units
        .iter()
        .map(|unit| match unit.participant {
            MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
            MixedInlineParticipantOf::Boundary(control) => control.source_index(),
            MixedInlineParticipantOf::ForcedLineBreak(control) => control.source_index(),
        })
        .collect()
}

fn assert_numeric_receipts<S: LayoutScalar>(
    permissions: [InlineBreakOpportunity; 3],
    direction: Direction,
    first_end: surgeist_text::LineEnd,
) {
    let input = numeric_input::<S>(permissions, direction);
    let supplied = project_inline_participants(&input.participants);
    let run = ProjectedInlineRun {
        participants: &input.participants,
        supplied: &supplied,
    };
    let select = |cursor, pending| {
        select_next_inline_line(
            run,
            cursor,
            supplied.len(),
            S::from_f64(25.0),
            true,
            pending,
            input.flow_axes,
        )
    };
    let first = select(InlineSourceCursor::default(), None).unwrap();
    assert!(match first_end {
        surgeist_text::LineEnd::Soft => first.end == SelectedInlineLineEnd::Soft,
        surgeist_text::LineEnd::Forced => first.end == SelectedInlineLineEnd::Forced,
        surgeist_text::LineEnd::Paragraph => unreachable!("the first line has a real successor"),
    });
    assert_eq!(source_indices(&first.line), [0, 1, 2]);
    assert_eq!(first.next_cursor.source, 3);
    assert_eq!(first.next_cursor.descriptor, 3);
    // Actual selected numeric flow is 2 * (10 + 5 - 5), not the border envelope25.
    assert_eq!(first.line.used_inline_extent, S::from_f64(20.0));
    let second = select(first.next_cursor, first.pending_strut).unwrap();
    assert!(second.end == SelectedInlineLineEnd::Paragraph);
    assert_eq!(source_indices(&second.line), [3, 4]);
    assert_eq!(second.next_cursor.source, 5);
    assert_eq!(second.next_cursor.descriptor, 5);
    assert_eq!(second.line.used_inline_extent, S::from_f64(10.0));
    assert!(select(second.next_cursor, second.pending_strut).is_none());

    let report = layout_mixed_inline_run(input).unwrap();
    assert_eq!(
        report.lines.iter().map(|line| line.end).collect::<Vec<_>>(),
        [first_end, surgeist_text::LineEnd::Paragraph]
    );
    assert_eq!(report.line_bands.len(), 2);
    assert_eq!(report.inline_extent, S::from_f64(20.0));
    assert_eq!(report.block_extent, S::from_f64(20.0));
    // These are flow-logical border starts, not physical x. The level0
    // atomics keep L2 indices0/2/0; RTL logical coordinates run oppositely.
    let starts = match direction {
        Direction::Ltr => [5.0, 15.0, 5.0],
        Direction::Rtl => [15.0, 5.0, 5.0],
    };
    assert_eq!(
        report
            .atomics
            .iter()
            .map(|row| (
                row.item.source_index,
                row.line_index,
                row.inline_start,
                row.block_start
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, S::from_f64(starts[0]), S::ZERO),
            (2, 0, S::from_f64(starts[1]), S::ZERO),
            (3, 1, S::from_f64(starts[2]), S::from_f64(10.0)),
        ]
    );
    assert_eq!(
        report
            .atomics
            .iter()
            .map(|row| (row.item.source_index, row.visual_index))
            .collect::<Vec<_>>(),
        [(0, Some(0)), (2, Some(2)), (3, Some(0))]
    );
    assert_eq!(
        report
            .controls
            .iter()
            .map(|row| (
                row.source_index,
                row.line_index,
                row.kind,
                row.inline_extent
            ))
            .collect::<Vec<_>>(),
        [
            (
                1,
                0,
                InlineParticipantLayoutKind::InlineBoundaryStart,
                S::ZERO
            ),
            (
                4,
                1,
                InlineParticipantLayoutKind::InlineBoundaryEnd,
                S::ZERO
            ),
        ]
    );
}

fn assert_layout_placement<S: LayoutScalar>(
    permissions: [InlineBreakOpportunity; 3],
    direction: Direction,
) {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3, 4, 5])
        .style(
            0,
            NodeInputOf {
                display: crate::Display::Block,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(25.0)),
                    PreferredSizeOf::AUTO,
                ),
                text_align: TextAlign::Start,
                ..NodeInputOf::default()
            },
        );
    for (node, following) in [1, 3, 4].into_iter().zip(permissions) {
        tree = tree.style(
            node,
            NodeInputOf {
                display: crate::Display::InlineBlock,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                margin: signed_margin::<S>(direction).map(LengthAutoOf::px),
                atomic_inline_participation: Some(participation(following)),
                ..NodeInputOf::default()
            },
        );
    }
    // Genuine endpoint nodes retain their individual roles. Today's ordinary
    // native operation has no supplied pair/scope policy, so none is invented.
    for (node, kind) in [(2, InlineBoundaryKind::Start), (5, InlineBoundaryKind::End)] {
        tree = tree.input(
            node,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::new(kind, metrics(), InlineBoundaryEdgeOf::ZERO)
                    .with_direction(direction),
            ),
        );
    }
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(25.0)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = |node| -> NodeOutputOf<S> {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        assert_eq!(
            output(0).size,
            Size::new(S::from_f64(25.0), S::from_f64(20.0))
        );
        let starts = match direction {
            Direction::Ltr => [5.0, 15.0, 5.0],
            // Level0 atoms retain physical left-to-right source order. The
            // two-box RTL row starts at25-20=5, then physical margin-left -5;
            // the single-box row starts at25-10=15 with the same -5 margin.
            Direction::Rtl => [0.0, 10.0, 10.0],
        };
        for ((node, source_index), (x, y)) in [1, 3, 4]
            .into_iter()
            .zip([0, 2, 3])
            .zip(starts.into_iter().zip([0.0, 0.0, 10.0]))
        {
            let actual = output(node);
            assert_eq!(actual.source_index.get(), source_index);
            assert_eq!(actual.location, Point::new(S::from_f64(x), S::from_f64(y)));
            assert_eq!(actual.size, Size::splat(S::from_f64(10.0)));
            assert_eq!(actual.margin, signed_margin(direction));
        }
        let origin = match direction {
            Direction::Ltr => S::ZERO,
            Direction::Rtl => S::from_f64(25.0),
        };
        let envelope = |nodes: &[u32]| {
            nodes.iter().fold((origin, origin), |(start, end), &node| {
                let actual = output(node);
                (
                    start.min(actual.location.x),
                    end.max(actual.location.x + actual.size.width),
                )
            })
        };
        assert_eq!(envelope(&[1, 3]), (S::ZERO, S::from_f64(25.0)));
        assert_eq!(
            envelope(&[4]),
            match direction {
                Direction::Ltr => (S::ZERO, S::from_f64(15.0)),
                Direction::Rtl => (S::from_f64(10.0), S::from_f64(25.0)),
            }
        );
        for (node, source_index) in [(2, 1), (5, 4)] {
            assert_eq!(output(node).source_index.get(), source_index);
            assert_eq!(output(node).size, Size::ZERO);
        }
    }
}

fn assert_both_routes(permissions: [InlineBreakOpportunity; 3], first_end: surgeist_text::LineEnd) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        assert_numeric_receipts::<f32>(permissions, direction, first_end);
        assert_numeric_receipts::<f64>(permissions, direction, first_end);
        assert_layout_placement::<f32>(permissions, direction);
        assert_layout_placement::<f64>(permissions, direction);
    }
}

#[test]
fn allowed_auto_keeps_used_flow_distinct_from_placed_signed_margin_borders() {
    assert_both_routes(
        [InlineBreakOpportunity::Allowed; 3],
        surgeist_text::LineEnd::Soft,
    );
}

#[test]
fn prohibited_after_first_box_keeps_the_real_second_box_cut() {
    assert_both_routes(
        [
            InlineBreakOpportunity::Prohibited,
            InlineBreakOpportunity::Allowed,
            InlineBreakOpportunity::Allowed,
        ],
        surgeist_text::LineEnd::Soft,
    );
}

#[test]
fn mandatory_after_second_box_keeps_forced_then_paragraph_without_an_extra_line() {
    assert_both_routes(
        [
            InlineBreakOpportunity::Allowed,
            InlineBreakOpportunity::Mandatory,
            InlineBreakOpportunity::Allowed,
        ],
        surgeist_text::LineEnd::Forced,
    );
}
