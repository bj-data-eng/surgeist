//! Independent numeric box-edge acceptance: real boxes/pairs, no Text stand-in.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    ChildOrderCompleteness, CompletedLayoutBatchOf, GeneratedBoxParticipation, GeneratedBoxRef,
    GeneratedChildBoxInput, InlineBoundaryInputOf, InlineBoxRef, InlineChildOrderInput,
    LayoutFlowInput, LayoutInputOf, LayoutRootRequestOf, LengthAutoOf, NodeInputOf, NodeOutputOf,
    Point, PreferredSizeOf, WrapEdge, WrapEdges, WrapInside, WrappingInput, WrappingScopeError,
    WrappingScopeInput, compute_layout_with_flow,
};

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn metrics<S: LayoutScalar>() -> InlineMetricsOf<S> {
    InlineMetricsOf::from_line_height_and_baseline(
        crate::InlineBaselineContext::Physical,
        n(10.0),
        n(10.0),
    )
    .unwrap()
}
fn participation(following: InlineBreakOpportunity) -> AtomicInlineParticipation {
    // This is the already qualified native level0 atomic profile, with separate
    // containing axes. Changing Direction does not manufacture resolved levels.
    AtomicInlineParticipation::new(BidiLevel::try_new(0).unwrap(), following)
}
fn signed_margin<S: LayoutScalar>(direction: Direction, signed: bool) -> Edges<S> {
    if !signed {
        return Edges::ZERO;
    }
    match direction {
        Direction::Ltr => Edges::new(S::ZERO, n(-5.0), S::ZERO, n(5.0)),
        Direction::Rtl => Edges::new(S::ZERO, n(5.0), S::ZERO, n(-5.0)),
    }
}

struct Fixture<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    nodes: Vec<u32>,
    participants: Vec<MixedInlineParticipantOf<S>>,
    wrapping: WrappingInput<u32>,
    direction: Direction,
    width: S,
}
impl<S: LayoutScalar> Fixture<S> {
    fn new(direction: Direction, width: f64) -> Self {
        Self {
            tree: PublicLayoutTreeOf::new().style(
                0,
                NodeInputOf {
                    display: crate::Display::Block,
                    direction,
                    size: Size::new(PreferredSizeOf::px(n(width)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            ),
            nodes: vec![],
            participants: vec![],
            wrapping: WrappingInput::default(),
            direction,
            width: n(width),
        }
    }
    fn atomic(
        &mut self,
        node: u32,
        edges: WrapEdges,
        following: InlineBreakOpportunity,
        signed: bool,
    ) {
        let margin = signed_margin(self.direction, signed);
        self.tree.insert_input(
            node,
            LayoutInputOf::box_input(NodeInputOf {
                display: crate::Display::InlineBlock,
                direction: self.direction,
                size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                margin: margin.map(LengthAutoOf::px),
                atomic_inline_participation: Some(participation(following)),
                wrap_edges: edges,
                ..NodeInputOf::default()
            }),
        );
        self.nodes.push(node);
        self.participants.push(MixedInlineParticipantOf::Atomic {
            item: AtomicInlineBoxParticipant {
                baseline_reference: crate::InlineBaselineContext::Physical,
                source_index: node as usize,
                size: Size::splat(n(10.0)),
                content_size: Size::ZERO,
                margin,
                padding: Edges::ZERO,
                border: Edges::ZERO,
                scrollbar_size: Size::ZERO,
                first_baseline: Some(n(10.0)),
                alignment: InlineControlAlignment::Baseline,
            },
            participation: participation(following),
        });
    }
    fn boundary(&mut self, node: u32, kind: InlineBoundaryKind, advance: f64) {
        let edge = InlineBoundaryEdgeOf::try_new(S::ZERO, n(advance), S::ZERO).unwrap();
        self.tree.insert_input(
            node,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(kind, metrics(), edge)
                    .unwrap()
                    .with_direction(self.direction),
            ),
        );
        self.nodes.push(node);
        self.participants.push(MixedInlineParticipantOf::Boundary(
            InlineBoundaryControlOf::new(
                node as usize,
                kind,
                InlineFlowOf::new(
                    WritingMode::HorizontalTb,
                    self.direction,
                    AvailableOf::definite(self.width),
                ),
                metrics(),
                InlineControlAlignment::Baseline,
                edge,
                None,
            ),
        ));
    }
    fn finish(&mut self) {
        self.tree.insert_children(0, self.nodes.iter().copied());
    }
    fn flow(&self) -> LayoutFlowInput<'_, PublicLayoutTreeOf<S>> {
        LayoutFlowInput::try_new(&self.tree, 0, vec![], self.wrapping.clone(), Vec::new()).unwrap()
    }
    fn numeric(&self) -> MixedInlineRunInputOf<S> {
        let flow = self.flow();
        let projection = flow
            .wrapping
            .numeric_projection(&flow.graph, 0, &self.nodes)
            .unwrap();
        MixedInlineRunInputOf {
            inline_baseline_context: crate::InlineBaselineContext::Physical,
            groups: crate::group_alignment::Snapshot::default(),
            text_wrap_style: crate::TextWrapStyle::Auto,
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                self.direction,
            ),
            observe_tail: true,
            wrapping: Some(projection),
            available_inline_extent: AvailableOf::definite(self.width),
            flow_axes: FlowAxes::new(WritingMode::HorizontalTb, self.direction),
            text_align: TextAlign::Start,
            text_align_last: crate::TextAlignLast::Auto,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            participants: self.participants.clone(),
        }
    }
    fn compute(&self) -> CompletedLayoutBatchOf<u32, S> {
        compute_layout_with_flow(&self.flow(), 0, request(self.width)).unwrap()
    }
}
fn request<S: LayoutScalar>(width: S) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(width),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}
fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|e| e.node() == node)
        .unwrap()
        .output()
}
fn groups<S: LayoutScalar>(report: &MixedInlineRunReportOf<S>) -> Vec<Vec<usize>> {
    (0..report.lines.len())
        .map(|line| {
            report
                .atomics
                .iter()
                .filter(|a| a.line_index == line)
                .map(|a| a.item.source_index)
                .collect()
        })
        .collect()
}
fn simple<S: LayoutScalar>(
    direction: Direction,
    width: f64,
    edges: [WrapEdges; 3],
    permissions: [InlineBreakOpportunity; 3],
    signed: bool,
) -> Fixture<S> {
    let mut f = Fixture::new(direction, width);
    for ((node, edge), permission) in [1, 2, 3].into_iter().zip(edges).zip(permissions) {
        f.atomic(node, edge, permission, signed);
    }
    f.finish();
    f
}
fn check_simple<S: LayoutScalar>(
    f: &Fixture<S>,
    expected: &[&[usize]],
    ends: &[surgeist_text::LineEnd],
) {
    let report = layout_mixed_inline_run(f.numeric()).unwrap();
    assert_eq!(
        groups(&report),
        expected.iter().map(|s| s.to_vec()).collect::<Vec<_>>()
    );
    assert_eq!(report.lines.iter().map(|l| l.end).collect::<Vec<_>>(), ends);
    let batch = f.compute();
    assert_eq!(
        output(&batch, 0).size.height,
        n(10.0 * expected.len() as f64)
    );
    for (line, nodes) in expected.iter().enumerate() {
        for (position, &node) in nodes.iter().enumerate() {
            let margin = match &f.participants[0] {
                MixedInlineParticipantOf::Atomic { item, .. } => item.margin.left.to_f64().abs(),
                _ => 0.0,
            };
            let logical = 10.0 * position as f64 + margin;
            let x = match f.direction {
                Direction::Ltr => logical,
                // All atomics have resolved level0: physical L2 order is
                // source order even inside RTL. The used row is right-aligned;
                // its physical left margin is -margin, not logical start +margin.
                Direction::Rtl => {
                    f.width.to_f64() - 10.0 * nodes.len() as f64 + 10.0 * position as f64 - margin
                }
            };
            let actual = output(&batch, node as u32);
            assert_eq!(actual.location, Point::new(n(x), n(10.0 * line as f64)));
            assert_eq!(actual.size, Size::splat(n(10.0)));
        }
    }
}
fn simple_matrix<S: LayoutScalar>() {
    use surgeist_text::LineEnd::{Paragraph, Soft};
    for direction in [Direction::Ltr, Direction::Rtl] {
        for edge in [
            WrapEdge::Auto,
            WrapEdge::AvoidFlex,
            WrapEdge::Flex,
            WrapEdge::Avoid,
            WrapEdge::AvoidLine,
        ] {
            let mut edges = [WrapEdges::AUTO; 3];
            edges[1].after = edge;
            let f = simple::<S>(
                direction,
                25.0,
                edges,
                [InlineBreakOpportunity::Allowed; 3],
                true,
            );
            let expected: &[&[usize]] = if matches!(edge, WrapEdge::Avoid | WrapEdge::AvoidLine) {
                &[&[1], &[2, 3]]
            } else {
                &[&[1, 2], &[3]]
            };
            check_simple(&f, expected, &[Soft, Paragraph]);
            let report = layout_mixed_inline_run(f.numeric()).unwrap();
            assert_eq!(report.inline_extent, n(20.0));
            // Fit uses10/20 flow; the origin-anchored physical footprint of a
            // one/two-box signed-margin line is15/25, respectively.
            let batch = f.compute();
            for nodes in expected {
                let origin = if direction == Direction::Ltr {
                    S::ZERO
                } else {
                    f.width
                };
                let (lo, hi) = nodes.iter().fold((origin, origin), |(lo, hi), &node| {
                    let a = output(&batch, node as u32);
                    (lo.min(a.location.x), hi.max(a.location.x + a.size.width))
                });
                assert_eq!(hi - lo, n(10.0 * nodes.len() as f64 + 5.0));
            }
            let input = f.numeric();
            let supplied = project_inline_participants(&input.participants);
            let run = ProjectedInlineRun {
                participants: &input.participants,
                supplied: &supplied,
            };
            assert_eq!(
                inline_min_content(run, input.flow_axes, input.wrapping.as_ref()).unwrap(),
                n(10.0)
            );
            assert_eq!(
                inline_max_content(run, input.flow_axes, input.wrapping.as_ref()).unwrap(),
                n(30.0)
            );
        }
        let mut edges = [WrapEdges::AUTO; 3];
        edges[1].after = WrapEdge::Avoid;
        // No other fitting legal cut: Avoid cannot turn Prohibited into a cut
        // or hide the only fitting second-box opportunity.
        let f = simple::<S>(
            direction,
            25.0,
            edges,
            [
                InlineBreakOpportunity::Prohibited,
                InlineBreakOpportunity::Allowed,
                InlineBreakOpportunity::Allowed,
            ],
            false,
        );
        check_simple(&f, &[&[1, 2], &[3]], &[Soft, Paragraph]);
    }
}
#[test]
fn actual_numeric_edge_avoidance_preserves_legal_fit_and_signed_border_geometry() {
    simple_matrix::<f32>();
    simple_matrix::<f64>();
}

fn force_matrix<S: LayoutScalar>() {
    use surgeist_text::LineEnd::{Forced, Paragraph};
    for direction in [Direction::Ltr, Direction::Rtl] {
        for before in [false, true] {
            let mut edges = [WrapEdges::AUTO; 3];
            if before {
                edges[1].before = WrapEdge::Line;
                edges[0].after = WrapEdge::Avoid;
            } else {
                edges[0].after = WrapEdge::Line;
                edges[1].before = WrapEdge::AvoidLine;
            }
            // All soft opportunities are genuinely prohibited. This is the
            // native no-soft profile, not a fabricated computed CSS Style.
            let f = simple::<S>(
                direction,
                100.0,
                edges,
                [InlineBreakOpportunity::Prohibited; 3],
                false,
            );
            check_simple(&f, &[&[1], &[2, 3]], &[Forced, Paragraph]);
            let input = f.numeric();
            let supplied = project_inline_participants(&input.participants);
            let run = ProjectedInlineRun {
                participants: &input.participants,
                supplied: &supplied,
            };
            assert_eq!(
                inline_min_content(run, input.flow_axes, input.wrapping.as_ref()).unwrap(),
                n(20.0)
            );
            assert_eq!(
                inline_max_content(run, input.flow_axes, input.wrapping.as_ref()).unwrap(),
                n(20.0)
            );
        }
    }
}
#[test]
fn matching_line_force_beats_avoid_and_no_soft_permissions_and_splits_intrinsics() {
    force_matrix::<f32>();
    force_matrix::<f64>();
}

fn empty_edges<S: LayoutScalar>() {
    use surgeist_text::LineEnd::{Forced, Paragraph};
    for direction in [Direction::Ltr, Direction::Rtl] {
        let mut edges = [WrapEdges::AUTO; 3];
        edges[0].before = WrapEdge::Line;
        let leading = simple::<S>(
            direction,
            100.0,
            edges,
            [InlineBreakOpportunity::Allowed; 3],
            false,
        );
        let r = layout_mixed_inline_run(leading.numeric()).unwrap();
        assert_eq!(groups(&r), [vec![], vec![1, 2, 3]]);
        assert_eq!(
            r.lines.iter().map(|l| l.end).collect::<Vec<_>>(),
            [Forced, Paragraph]
        );
        assert_eq!(r.block_extent, n(10.0));
        assert_eq!(
            r.atomics
                .iter()
                .map(|a| crate::inline::expect_logical_position(a.position).block)
                .collect::<Vec<_>>(),
            [S::ZERO; 3]
        );
        assert_eq!(output(&leading.compute(), 0).size.height, n(10.0));
        // There is no incoming/container numeric strut. Future box metrics may
        // not be borrowed to give the empty leading line a fictitious height.
        edges = [WrapEdges::AUTO; 3];
        edges[2].after = WrapEdge::Line;
        let terminal = simple::<S>(
            direction,
            100.0,
            edges,
            [InlineBreakOpportunity::Allowed; 3],
            false,
        );
        let r = layout_mixed_inline_run(terminal.numeric()).unwrap();
        // The genuine terminal Force ends the atomic line; its pending
        // Paragraph strut does not supply another formatted line.
        assert_eq!(groups(&r), [vec![1, 2, 3]]);
        assert_eq!(r.lines.iter().map(|l| l.end).collect::<Vec<_>>(), [Forced]);
        assert_eq!(r.block_extent, n(10.0));
        assert_eq!(r.baselines.numeric().first, Some(n(10.0)));
        assert_eq!(r.baselines.numeric().last, Some(n(10.0)));
        assert_eq!(output(&terminal.compute(), 0).size.height, n(10.0));
        edges = [WrapEdges::AUTO; 3];
        edges[0].after = WrapEdge::Line;
        edges[1].before = WrapEdge::Line;
        check_simple(
            &simple::<S>(
                direction,
                100.0,
                edges,
                [InlineBreakOpportunity::Allowed; 3],
                false,
            ),
            &[&[1], &[2, 3]],
            &[Forced, Paragraph],
        );
    }
}
#[test]
fn leading_terminal_and_adjacent_forces_have_genuine_numeric_empty_line_metrics() {
    empty_edges::<f32>();
    empty_edges::<f64>();
}

fn pair(start: u32, end: u32) -> GeneratedBoxRef<u32> {
    GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end })
}
fn child(box_: GeneratedBoxRef<u32>) -> GeneratedChildBoxInput<u32> {
    GeneratedChildBoxInput {
        box_,
        participation: GeneratedBoxParticipation::InFlow,
    }
}
fn order(
    parent: GeneratedBoxRef<u32>,
    children: Vec<GeneratedChildBoxInput<u32>>,
) -> InlineChildOrderInput<u32> {
    InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    }
}
fn nested<S: LayoutScalar>(
    direction: Direction,
    after: bool,
    empty: bool,
    oof: bool,
) -> Fixture<S> {
    let mut f = Fixture::new(direction, 100.0);
    f.atomic(1, WrapEdges::AUTO, InlineBreakOpportunity::Allowed, false);
    f.boundary(10, InlineBoundaryKind::Start, 5.0);
    if empty {
        f.boundary(14, InlineBoundaryKind::Start, 0.0);
        f.boundary(15, InlineBoundaryKind::End, 0.0);
    }
    if oof {
        f.tree.insert_input(
            9,
            LayoutInputOf::box_input(NodeInputOf {
                position: crate::Position::Absolute,
                size: Size::new(PreferredSizeOf::px(n(1.0)), PreferredSizeOf::px(n(1.0))),
                ..NodeInputOf::default()
            }),
        );
    }
    f.boundary(11, InlineBoundaryKind::Start, 3.0);
    f.atomic(
        2,
        if after {
            WrapEdges {
                before: WrapEdge::Auto,
                after: WrapEdge::Line,
            }
        } else {
            WrapEdges {
                before: WrapEdge::Line,
                after: WrapEdge::Auto,
            }
        },
        InlineBreakOpportunity::Allowed,
        false,
    );
    f.boundary(12, InlineBoundaryKind::End, 2.0);
    f.boundary(13, InlineBoundaryKind::End, 4.0);
    f.atomic(3, WrapEdges::AUTO, InlineBreakOpportunity::Allowed, false);
    for (start, end) in [(10, 13), (11, 12)] {
        f.wrapping.scopes.push(WrappingScopeInput::NativePair {
            metric_contribution: None,
            start,
            end,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
        });
    }
    let mut outer = vec![];
    if empty {
        f.wrapping.scopes.push(WrappingScopeInput::NativePair {
            metric_contribution: None,
            start: 14,
            end: 15,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
        });
        outer.push(child(pair(14, 15)));
    }
    if oof {
        outer.push(GeneratedChildBoxInput {
            box_: GeneratedBoxRef::Node(9),
            participation: GeneratedBoxParticipation::OutOfFlow,
        });
    }
    outer.push(child(pair(11, 12)));
    f.wrapping.child_order = vec![
        order(
            GeneratedBoxRef::Node(0),
            vec![
                child(GeneratedBoxRef::Node(1)),
                child(pair(10, 13)),
                child(GeneratedBoxRef::Node(3)),
            ],
        ),
        order(pair(10, 13), outer),
        order(pair(11, 12), vec![child(GeneratedBoxRef::Node(2))]),
    ];
    f.finish();
    if oof {
        let mut all = f.nodes.clone();
        all.insert(2, 9);
        f.tree.insert_children(0, all);
    }
    f
}
fn propagation<S: LayoutScalar>() {
    use surgeist_text::LineEnd::{Forced, Paragraph};
    for direction in [Direction::Ltr, Direction::Rtl] {
        for (empty, oof) in [(false, false), (true, false), (false, true)] {
            let f = nested::<S>(direction, false, empty, oof);
            let report = layout_mixed_inline_run(f.numeric()).unwrap();
            assert_eq!(groups(&report), [vec![1], vec![2, 3]]);
            let expected_flow = if empty { [15.0, 29.0] } else { [10.0, 34.0] };
            assert_eq!(
                report.lines.iter().map(|l| l.end).collect::<Vec<_>>(),
                [Forced, Paragraph]
            );
            // Endpoint BP stays on the real selected line; no post-subtraction.
            let supplied = project_inline_participants(&f.participants);
            let run = ProjectedInlineRun {
                participants: &f.participants,
                supplied: &supplied,
            };
            let input = f.numeric();
            let mut cursor = InlineSourceCursor::default();
            let mut pending = None;
            for extent in expected_flow {
                let InlineSelectionStep::Selected(Some(selected)) = select_scope_inline_line(
                    NumericLineQuery {
                        goal: None,
                        indent: n(0.0),
                        run,
                        cursor,
                        next_boundary_descriptor: supplied.len(),
                        available: n(100.0),
                        wraps: true,
                        pending,
                        axes: input.flow_axes,
                    },
                    input.wrapping.as_ref(),
                    |_, _| Ok(None),
                    |r| Ok(r.summary.used_inline_extent <= n(100.0)),
                )
                .unwrap() else {
                    panic!("resolved metric-only selection does not retry its receiver")
                };
                assert_eq!(selected.line.used_inline_extent, n(extent));
                cursor = selected.next_cursor;
                pending = selected.pending_strut;
            }
            let b = f.compute();
            // Atomic levels remain0. RTL L2 reverses only adjacent level1
            // boundary runs. Used widths are10/34, or15/29 with the real empty
            // sibling: their line-left bases are90/66, or85/71.
            for (node, ltr, rtl, y) in [
                (1, 0.0, if empty { 85.0 } else { 90.0 }, 0.0),
                (2, if empty { 3.0 } else { 8.0 }, 74.0, 10.0),
                (3, if empty { 19.0 } else { 24.0 }, 90.0, 10.0),
            ] {
                let x = if direction == Direction::Ltr {
                    ltr
                } else {
                    rtl
                };
                assert_eq!(output(&b, node).location, Point::new(n(x), n(y)));
            }
            assert_eq!(output(&b, 0).size.height, n(20.0));
        }
        let f = nested::<S>(direction, true, false, false);
        let r = layout_mixed_inline_run(f.numeric()).unwrap();
        assert_eq!(groups(&r), [vec![1, 2], vec![3]]);
        let b = f.compute();
        assert_eq!(output(&b, 1).location.y, S::ZERO);
        assert_eq!(output(&b, 2).location.y, S::ZERO);
        assert_eq!(output(&b, 3).location.y, n(10.0));
    }
}
#[test]
fn forced_propagation_consumes_real_pair_bp_and_empty_but_not_out_of_flow_siblings() {
    propagation::<f32>();
    propagation::<f64>();
}

fn side_proof<S: LayoutScalar>() {
    let mut f = nested::<S>(Direction::Ltr, false, false, false);
    // Before consumes prefix, not suffix. Unrequested evidence is not demanded.
    for row in &mut f.wrapping.child_order {
        row.suffix = ChildOrderCompleteness::Unknown;
    }
    assert_eq!(
        groups(&layout_mixed_inline_run(f.numeric()).unwrap()),
        [vec![1], vec![2, 3]]
    );
    f.wrapping.child_order[1].prefix = ChildOrderCompleteness::Unknown;
    let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
        .err()
        .expect("requested propagation lacks actual first-child proof");
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::MissingChildOrder
        ))
    ));
}
#[test]
fn forced_propagation_demands_only_the_relevant_child_order_side() {
    side_proof::<f32>();
    side_proof::<f64>();
}

fn opaque_context<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let mut f = nested::<S>(direction, false, false, false);
        // A owns its genuine internal IFC. Only its child has a Line edge;
        // A's exterior remains Auto and may not transmit that internal force.
        f.tree.insert_input(
            2,
            LayoutInputOf::box_input(NodeInputOf {
                display: crate::Display::InlineBlock,
                direction,
                size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                atomic_inline_participation: Some(participation(InlineBreakOpportunity::Allowed)),
                ..NodeInputOf::default()
            }),
        );
        f.tree.insert_children(2, [4]);
        f.tree.insert_input(
            4,
            LayoutInputOf::box_input(NodeInputOf {
                display: crate::Display::InlineBlock,
                direction,
                size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                atomic_inline_participation: Some(participation(InlineBreakOpportunity::Allowed)),
                wrap_edges: WrapEdges {
                    before: WrapEdge::Line,
                    after: WrapEdge::Auto,
                },
                ..NodeInputOf::default()
            }),
        );
        let mut inner = order(
            GeneratedBoxRef::Node(2),
            vec![child(GeneratedBoxRef::Node(4))],
        );
        inner.formatting_context = 2;
        f.wrapping.child_order.push(inner);
        let b = f.compute();
        // No imported internal strut or guessed atomic baseline. P and Q's
        // equal real10/0 metrics place them on the same outer line; 5+3+2+4
        // real BP and three actual10 boxes give44 total inline flow.
        assert_eq!(output(&b, 1).location.y, output(&b, 3).location.y);
        // RTL line-left is100-44=56. Actual L2 order is P, innerStart3,
        // outerStart5, A, outerEnd4, innerEnd2, Q; atomic x is56/74/90.
        for (node, ltr, rtl) in [(1, 0.0, 56.0), (2, 18.0, 74.0), (3, 34.0, 90.0)] {
            let x = if direction == Direction::Ltr {
                ltr
            } else {
                rtl
            };
            assert_eq!(output(&b, node).location.x, n(x));
        }
        assert_eq!(output(&b, 4).size, Size::splat(n(10.0)));
        assert_eq!(
            output(&b, 4).location.y,
            S::ZERO,
            "zero-metric leading internal force does not import a future strut"
        );
    }
}
#[test]
fn internal_atomic_formatting_context_is_opaque_to_outer_forced_propagation() {
    opaque_context::<f32>();
    opaque_context::<f64>();
}

// Faithful host cache transaction: no precomputed outputs or policy-derived
// answers. The normal engine emits and applies every retained cache entry.
#[derive(Clone)]
struct Retained<S: LayoutScalar> {
    caches: std::collections::HashMap<u32, crate::CacheOf<S>>,
    outputs: std::collections::HashMap<u32, NodeOutputOf<S>>,
}
struct Cached<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    retained: Retained<S>,
    hits: std::cell::Cell<usize>,
}
impl<S: LayoutScalar> Cached<S> {
    fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            retained: Retained {
                caches: Default::default(),
                outputs: Default::default(),
            },
            hits: std::cell::Cell::new(0),
        }
    }
}
impl<S: LayoutScalar> crate::Traverse for Cached<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as crate::Traverse>::Children<'a>
    where
        S: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        crate::Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        crate::Traverse::child_count(&self.tree, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        crate::Traverse::child(&self.tree, node, index)
    }
}
impl<S: LayoutScalar> crate::LayoutTree for Cached<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        crate::LayoutTree::node_input(&self.tree, node)
    }
    fn layout_input(&self, node: u32) -> crate::LayoutInputViewOf<'_, S> {
        crate::LayoutTree::layout_input(&self.tree, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
    ) -> Option<crate::ComputeOutputOf<S>> {
        let result = self
            .retained
            .caches
            .get(&node)?
            .get_with_context(input, context);
        if result.is_some() {
            self.hits.set(self.hits.get() + 1);
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.retained.outputs.get(&node).copied()
    }
}
impl<S: LayoutScalar> crate::LayoutBatchSink<u32, S> for Cached<S> {
    type Error = core::convert::Infallible;
    type Prepared = Retained<S>;
    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Retained<S>, Self::Error> {
        let mut next = self.retained.clone();
        for e in batch.cache_clear_entries() {
            next.caches.remove(&e.node());
        }
        for e in batch.cache_store_entries() {
            next.caches.entry(e.node()).or_default().store_with_context(
                e.input(),
                e.context(),
                e.output(),
            );
        }
        for e in batch.unrounded_entries() {
            next.outputs.insert(e.node(), e.output());
        }
        Ok(next)
    }
    fn commit_layout_batch(&mut self, next: Retained<S>) {
        self.retained = next;
    }
}
fn warm_force<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = simple::<S>(
            direction,
            100.0,
            [WrapEdges::AUTO; 3],
            [InlineBreakOpportunity::Allowed; 3],
            false,
        );
        let mut tree = f
            .tree
            .children(100, [0, 90])
            .style(
                100,
                NodeInputOf {
                    display: crate::Display::Block,
                    direction,
                    ..NodeInputOf::default()
                },
            )
            .style(
                90,
                NodeInputOf {
                    display: crate::Display::Block,
                    size: Size::new(PreferredSizeOf::px(n(1.0)), PreferredSizeOf::px(n(1.0))),
                    ..NodeInputOf::default()
                },
            );
        let run = |host: &Cached<S>| {
            let flow =
                LayoutFlowInput::try_new(host, 100, vec![], WrappingInput::default(), Vec::new())
                    .unwrap();
            compute_layout_with_flow(&flow, 100, request(n(100.0))).unwrap()
        };
        let mut warm = Cached::new(tree.clone());
        let old = run(&warm);
        assert_eq!(output(&old, 0).size.height, n(10.0));
        old.apply_to(&mut warm).unwrap();
        let mut box2 = crate::LayoutTree::node_input(&tree, 2).clone();
        box2.wrap_edges.before = WrapEdge::Line;
        tree.insert_input(2, LayoutInputOf::box_input(box2));
        warm.tree = tree.clone();
        let forced = run(&warm);
        let cold = run(&Cached::new(tree.clone()));
        assert_eq!(forced.final_entries(), cold.final_entries());
        assert_eq!(output(&forced, 0).size.height, n(20.0));
        assert_eq!(output(&forced, 1).location.y, S::ZERO);
        assert_eq!(output(&forced, 2).location.y, n(10.0));
        assert!(
            warm.hits.get() > 0,
            "unaffected block sibling really uses the retained host cache"
        );
        forced.apply_to(&mut warm).unwrap();
        let mut box2 = crate::LayoutTree::node_input(&tree, 2).clone();
        box2.wrap_edges = WrapEdges::AUTO;
        tree.insert_input(2, LayoutInputOf::box_input(box2));
        warm.tree = tree.clone();
        let restored = run(&warm);
        let cold = run(&Cached::new(tree));
        assert_eq!(restored.final_entries(), cold.final_entries());
        assert_eq!(output(&restored, 0).size.height, n(10.0));
        assert_eq!(output(&restored, 2).location.y, S::ZERO);
        assert_eq!(output(&old, 0).size.height, n(10.0));
        assert_eq!(output(&forced, 0).size.height, n(20.0));
    }
}
#[test]
fn force_only_declaration_changes_invalidate_warm_results_and_preserve_old_batches() {
    warm_force::<f32>();
    warm_force::<f64>();
}

fn mixed_constraints<S: LayoutScalar>(direction: Direction, width: f64) -> Fixture<S> {
    let mut f = Fixture::new(direction, width);
    f.atomic(
        1,
        WrapEdges {
            before: WrapEdge::Auto,
            after: WrapEdge::Avoid,
        },
        InlineBreakOpportunity::Allowed,
        false,
    );
    f.boundary(10, InlineBoundaryKind::Start, 0.0);
    f.atomic(2, WrapEdges::AUTO, InlineBreakOpportunity::Allowed, false);
    f.boundary(11, InlineBoundaryKind::End, 0.0);
    f.atomic(3, WrapEdges::AUTO, InlineBreakOpportunity::Allowed, false);
    f.boundary(12, InlineBoundaryKind::Start, 0.0);
    f.atomic(4, WrapEdges::AUTO, InlineBreakOpportunity::Allowed, false);
    f.boundary(13, InlineBoundaryKind::End, 0.0);
    f.atomic(5, WrapEdges::AUTO, InlineBreakOpportunity::Allowed, false);
    // Genuine separate boxes, not a synthetic class array. A cut after2 or4
    // is interior to the respective complete native pair; after3 is outside.
    for (start, end) in [(10, 11), (12, 13)] {
        f.wrapping.scopes.push(WrappingScopeInput::NativePair {
            metric_contribution: None,
            start,
            end,
            wrap_inside: WrapInside::Avoid,
            edges: WrapEdges::AUTO,
        });
    }
    f.finish();
    f
}
fn assert_mixed_output<S: LayoutScalar>(f: &Fixture<S>, expected: &[&[usize]]) {
    let report = layout_mixed_inline_run(f.numeric()).unwrap();
    assert_eq!(
        groups(&report),
        expected.iter().map(|s| s.to_vec()).collect::<Vec<_>>()
    );
    let batch = f.compute();
    assert_eq!(
        output(&batch, 0).size.height,
        n(10.0 * expected.len() as f64)
    );
    for (line, nodes) in expected.iter().enumerate() {
        for (position, &node) in nodes.iter().enumerate() {
            let logical = position as f64 * 10.0;
            let x = if f.direction == Direction::Ltr {
                logical
            } else {
                // Level0 atomics retain line-left source order; genuine
                // level1 boundaries here have zero advance.
                f.width.to_f64() - 10.0 * nodes.len() as f64 + logical
            };
            assert_eq!(
                output(&batch, node as u32).location,
                Point::new(n(x), n(line as f64 * 10.0))
            );
        }
    }
}
fn mixed_inside_edge<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        // At25 only cuts after1 and2 fit. The former violates only edge1;
        // the latter only inside(pair10,11). They are incomparable, so the
        // normal furthest-fitting candidate after2 must remain preferred.
        let narrow = mixed_constraints::<S>(direction, 25.0);
        assert_mixed_output(&narrow, &[&[1, 2], &[3], &[4, 5]]);
        let mut inside_only = mixed_constraints::<S>(direction, 25.0);
        let mut first = crate::LayoutTree::node_input(&inside_only.tree, 1).clone();
        first.wrap_edges = WrapEdges::AUTO;
        inside_only
            .tree
            .insert_input(1, LayoutInputOf::box_input(first));
        assert_mixed_output(&inside_only, &[&[1], &[2, 3], &[4, 5]]);

        // At45 the normal choice is after4. It violates inside(pair12,13).
        // The earlier third candidate after3 violates nothing and is a strict
        // subset of the edge-only, first-inside and second-inside candidates.
        let wide = mixed_constraints::<S>(direction, 45.0);
        assert_mixed_output(&wide, &[&[1, 2, 3], &[4, 5]]);
        let mut neutral = mixed_constraints::<S>(direction, 45.0);
        neutral.wrapping = WrappingInput::default();
        let mut first = crate::LayoutTree::node_input(&neutral.tree, 1).clone();
        first.wrap_edges = WrapEdges::AUTO;
        neutral
            .tree
            .insert_input(1, LayoutInputOf::box_input(first));
        assert_mixed_output(&neutral, &[&[1, 2, 3, 4], &[5]]);

        let input = wide.numeric();
        let wrapping = input.wrapping.as_ref().unwrap();
        // Actual normalized participant ends after1,2,3,4, including the real
        // endpoint occurrences. No AvoidClass fields or ancestry are forged.
        let cuts = [1, 3, 5, 7];
        for order in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            let mut frontier = wrapping.frontier();
            for candidate in order {
                frontier
                    .consider(wrapping.class(cuts[candidate]).unwrap(), candidate)
                    .unwrap();
            }
            assert_eq!(frontier.into_preferred(), Some(2));
        }
        for order in [[0, 1], [1, 0]] {
            let mut frontier = wrapping.frontier();
            for candidate in order {
                frontier
                    .consider(wrapping.class(cuts[candidate]).unwrap(), candidate)
                    .unwrap();
            }
            assert_eq!(frontier.into_preferred(), Some(order[1]));
        }
    }
}
#[test]
fn mixed_inside_and_edge_constraints_keep_incomparable_preference_and_actual_subset_winner() {
    mixed_inside_edge::<f32>();
    mixed_inside_edge::<f64>();
}

fn expect_missing_child_order<S: LayoutScalar>(f: &Fixture<S>) {
    let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
        .err()
        .expect("a demanded unknown generated-child incidence cannot be declared complete");
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::MissingChildOrder
        ))
    ));
}
fn with_neutral_atomic_edge<S: LayoutScalar>(f: &mut Fixture<S>) {
    let mut atomic = crate::LayoutTree::node_input(&f.tree, 2).clone();
    atomic.wrap_edges = WrapEdges::AUTO;
    f.tree.insert_input(2, LayoutInputOf::box_input(atomic));
}
fn unknown_child_incidence<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for after in [false, true] {
            let mut unpaired = nested::<S>(direction, after, false, false);
            // Endpoints11/12 are real independent native controls. Removing
            // their declared box does not license inferring a hidden pair or
            // omitting those occurrences from the consumed-side proof.
            unpaired.wrapping.scopes.retain(|scope| {
                matches!(
                    scope,
                    WrappingScopeInput::NativePair {
                        start: 10,
                        end: 13,
                        ..
                    }
                )
            });
            unpaired.wrapping.child_order.clear();
            expect_missing_child_order(&unpaired);
            with_neutral_atomic_edge(&mut unpaired);
            let neutral = unpaired.compute();
            assert_eq!(output(&neutral, 0).size.height, n(10.0));
            assert_eq!(
                output(&neutral, 1).location.y,
                output(&neutral, 3).location.y
            );

            let mut visible_break = nested::<S>(direction, after, false, false);
            visible_break.wrapping.child_order.clear();
            visible_break.tree.insert_input(
                9,
                LayoutInputOf::line_break(
                    crate::LineBreakInputOf::new()
                        .with_metrics(metrics())
                        .with_direction(direction),
                ),
            );
            // A genuine visible LineBreak is not a GeneratedBoxRef::Node.
            // Its participation cannot disappear from the side proof merely
            // because the known generated-box inventory contains only inner.
            let nodes = if after {
                vec![1, 10, 11, 2, 12, 9, 13, 3]
            } else {
                vec![1, 10, 9, 11, 2, 12, 13, 3]
            };
            visible_break.tree.insert_children(0, nodes);
            expect_missing_child_order(&visible_break);
            with_neutral_atomic_edge(&mut visible_break);
            let neutral = visible_break.compute();
            assert_eq!(output(&neutral, 0).size.height, n(20.0));
            assert_eq!(output(&neutral, 1).location.y, S::ZERO);
            assert_eq!(
                output(&neutral, 2).location.y,
                if after { S::ZERO } else { n(10.0) }
            );
            assert_eq!(output(&neutral, 3).location.y, n(10.0));

            // Positive control retains the explicitly declared genuine pair
            // and complete generated order. It proves the requested chain
            // without claiming every standalone control denotes a child box.
            let proved = nested::<S>(direction, after, false, false);
            let report = layout_mixed_inline_run(proved.numeric()).unwrap();
            assert_eq!(
                groups(&report),
                if after {
                    vec![vec![1, 2], vec![3]]
                } else {
                    vec![vec![1], vec![2, 3]]
                }
            );
            let batch = proved.compute();
            assert_eq!(output(&batch, 0).size.height, n(20.0));
        }
    }
}
#[test]
fn unpaired_endpoint_and_visible_break_need_truthful_demanded_child_incidence() {
    unknown_child_incidence::<f32>();
    unknown_child_incidence::<f64>();
}

fn separated_numeric_runs<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for (early, late, expected) in [
            (WrapEdges::AUTO, WrapEdges::AUTO, [0.0, 10.0, 20.0, 30.0]),
            (
                WrapEdges::AUTO,
                WrapEdges {
                    before: WrapEdge::Line,
                    after: WrapEdge::Auto,
                },
                [0.0, 10.0, 20.0, 30.0],
            ),
            (
                WrapEdges::AUTO,
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Line,
                },
                [0.0, 10.0, 20.0, 30.0],
            ),
            (
                WrapEdges {
                    before: WrapEdge::Line,
                    after: WrapEdge::Auto,
                },
                WrapEdges::AUTO,
                [0.0, 10.0, 20.0, 30.0],
            ),
            (
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Line,
                },
                WrapEdges::AUTO,
                [0.0, 10.0, 20.0, 30.0],
            ),
            (
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Line,
                },
                WrapEdges {
                    before: WrapEdge::Line,
                    after: WrapEdge::Auto,
                },
                [0.0, 10.0, 20.0, 30.0],
            ),
            (
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Line,
                },
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Line,
                },
                [0.0, 10.0, 20.0, 30.0],
            ),
        ] {
            // The normal block is an actual run separator. All three boxes
            // have border size10, no margin/offset and no external font strut.
            let mut tree = PublicLayoutTreeOf::<S>::new().children(0, [1, 2, 3]).style(
                0,
                NodeInputOf {
                    display: crate::Display::Block,
                    direction,
                    size: Size::new(PreferredSizeOf::px(n(100.0)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            );
            for (node, display, edges) in [
                (1, crate::Display::InlineBlock, early),
                (2, crate::Display::Block, WrapEdges::AUTO),
                (3, crate::Display::InlineBlock, late),
            ] {
                tree = tree.style(
                    node,
                    NodeInputOf {
                        display,
                        direction,
                        wrap_edges: edges,
                        size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                        atomic_inline_participation: (display == crate::Display::InlineBlock)
                            .then(|| participation(InlineBreakOpportunity::Allowed)),
                        ..NodeInputOf::default()
                    },
                );
            }
            let batch = crate::compute_layout(&tree, 0, request(n(100.0))).unwrap();
            // Equal geometry after phantom suppression cannot by itself prove
            // locality. Observe each actual run through its own host-derived
            // projection: an after force must end only that owner's atomic.
            let flow =
                LayoutFlowInput::try_new(&tree, 0, vec![], WrappingInput::default(), Vec::new())
                    .unwrap();
            for (node, edges) in [(1, early), (3, late)] {
                // Allocation fields are genuine owner inputs from the already
                // required unrounded public batch, never ending/cursor oracles.
                let allocation = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output();
                let projection = flow
                    .wrapping
                    .numeric_projection(&flow.graph, 0, &[node])
                    .unwrap();
                let report = layout_mixed_inline_run_with_formatted_observation(
                    MixedInlineRunInputOf {
                        inline_baseline_context: crate::InlineBaselineContext::Physical,
                        groups: crate::group_alignment::Snapshot::default(),
                        text_wrap_style: crate::TextWrapStyle::Auto,
                        line_box_context: crate::line_box::LineBoxContext::new(
                            crate::LineBoxDirectionPolicy::Normal,
                            direction,
                        ),
                        observe_tail: false,
                        wrapping: Some(projection),
                        available_inline_extent: AvailableOf::definite(n(100.0)),
                        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, direction),
                        text_align: TextAlign::Start,
                        text_align_last: crate::TextAlignLast::Auto,
                        indent: crate::text_indent::ResolvedTextIndent::ZERO,
                        indent_state: crate::text_indent::IndentLineState::default(),
                        // Empty blocks have no inner baseline. The numeric
                        // owner uses their actual block-end fallback (10+0),
                        // rather than a fabricated Some(10) baseline witness.
                        // The declared Visible overflow reserves no scrollbar.
                        participants: vec![MixedInlineParticipantOf::Atomic {
                            item: AtomicInlineBoxParticipant {
                                baseline_reference: crate::InlineBaselineContext::Physical,
                                source_index: allocation.source_index.get(),
                                size: allocation.size,
                                content_size: allocation.content_size,
                                margin: allocation.margin,
                                padding: allocation.padding,
                                border: allocation.border,
                                scrollbar_size: Size::ZERO,
                                first_baseline: None,
                                alignment: InlineControlAlignment::Baseline,
                            },
                            participation: participation(InlineBreakOpportunity::Allowed),
                        }],
                    },
                    |_, _| LogicalLineBandQueryResultOf {
                        inline_start: S::ZERO,
                        inline_end: n(100.0),
                        next_transition: None,
                    },
                    |block, _| block,
                    true,
                    crate::inline::unavailable_atomic_receiver,
                )
                .unwrap();
                let mut expected_endings = Vec::new();
                if edges.before == WrapEdge::Line {
                    expected_endings.push((surgeist_text::LineEnd::Forced, 0, 0));
                }
                expected_endings.push((
                    if edges.after == WrapEdge::Line {
                        surgeist_text::LineEnd::Forced
                    } else {
                        surgeist_text::LineEnd::Paragraph
                    },
                    1,
                    1,
                ));
                assert_eq!(
                    report
                        .formatted_lines
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|line| (line.ending, line.source_end, line.descriptor_end))
                        .collect::<Vec<_>>(),
                    expected_endings,
                    "actual run owner {node}, declarations {early:?}/{late:?}"
                );
            }
            // An owner's before force starts its own run with the already
            // qualified zero-metric numeric leading Forced line. Its after
            // force retains a nominal10/0 terminal strut, but the empty
            // Paragraph receiver is nonformatted. Neither force can spend
            // height in the other run across the actual intervening block.
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                let get = |node| {
                    entries
                        .iter()
                        .find(|entry| entry.node() == node)
                        .unwrap()
                        .output()
                };
                assert_eq!(
                    [
                        get(1).location.y,
                        get(2).location.y,
                        get(3).location.y,
                        get(0).size.height
                    ],
                    expected.map(n)
                );
                for node in [1, 2, 3] {
                    assert_eq!(get(node).size, Size::splat(n(10.0)));
                    assert_eq!(get(node).source_index.get(), (node - 1) as usize);
                }
                let inline_x = if direction == Direction::Ltr {
                    S::ZERO
                } else {
                    n(90.0)
                };
                assert_eq!(get(1).location.x, inline_x);
                assert_eq!(get(3).location.x, inline_x);
                assert_eq!(get(0).size.width, n(100.0));
            }
        }
    }
}
#[test]
fn edge_forces_stay_in_their_own_numeric_run_across_a_real_block_separator() {
    separated_numeric_runs::<f32>();
    separated_numeric_runs::<f64>();
}

#[path = "wrap_edges_tests/bidi_metadata.rs"]
mod bidi_metadata;
mod gap_attachments;
