//! Resolved physical attachment is distinct from an empty box's source cut.
use super::*;
use crate::{
    InlineDelimiterSide, InlineFormatterEdge, InlineGapAffinity, InlineGapAnchor,
    InlineGapGroupInput,
};

pub(super) fn replace_boundary<S: LayoutScalar>(
    f: &mut Fixture<S>,
    node: u32,
    boundary: InlineBoundaryInputOf<S>,
) {
    f.tree
        .insert_input(node, LayoutInputOf::inline_boundary(boundary));
    let control = f
        .participants
        .iter_mut()
        .find_map(|participant| match participant {
            MixedInlineParticipantOf::Boundary(control)
                if control.source_index() == node as usize =>
            {
                Some(control)
            }
            _ => None,
        })
        .unwrap();
    *control = InlineBoundaryControlOf::new(
        node as usize,
        boundary.kind(),
        InlineFlowOf::new(
            boundary.writing_mode(),
            boundary.direction(),
            AvailableOf::definite(f.width),
        ),
        boundary.metrics(),
        InlineControlAlignment::Baseline,
        boundary.edge(),
        boundary.parent_edge_axes(),
    );
}

fn reorder_fixture_participants<S: LayoutScalar>(f: &mut Fixture<S>) {
    f.participants.sort_by_key(|participant| {
        let source = match participant {
            MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
            MixedInlineParticipantOf::Boundary(control) => control.source_index(),
            MixedInlineParticipantOf::ForcedLineBreak(_) => unreachable!(),
        };
        f.nodes
            .iter()
            .position(|&node| node as usize == source)
            .unwrap()
    });
}

fn empty_box_fixture<S: LayoutScalar>() -> Fixture<S> {
    let mut f = super::bidi_metadata::split_box_fixture::<S>();
    // Move the real endpoints to the source cut between the last level-zero
    // atomic and the first level-one atomic. The box keeps its positive BP,
    // but contains no ordering operand.
    f.nodes = vec![10, 11, 12, 20, 21, 13, 14, 15];
    reorder_fixture_participants(&mut f);
    f.wrapping.child_order = vec![
        order(
            GeneratedBoxRef::Node(0),
            vec![
                child(GeneratedBoxRef::Node(10)),
                child(GeneratedBoxRef::Node(11)),
                child(GeneratedBoxRef::Node(12)),
                child(pair(20, 21)),
                child(GeneratedBoxRef::Node(13)),
                child(GeneratedBoxRef::Node(14)),
                child(GeneratedBoxRef::Node(15)),
            ],
        ),
        order(pair(20, 21), vec![]),
    ];
    f.finish();
    f
}

fn assert_order<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, expected: &[u32]) {
    let mut actual = (10..16u32)
        .map(|node| (node - 10, output(batch, node).location.x))
        .collect::<Vec<_>>();
    actual.sort_by(|a, b| a.1.to_f64().total_cmp(&b.1.to_f64()));
    assert_eq!(actual.iter().map(|row| row.0).collect::<Vec<_>>(), expected);
}

fn resolved_neighbor_gap<S: LayoutScalar>() {
    // These are the two real source-adjacent operands at the same source cut.
    // After L2 they bound different physical gaps. A resolved attachment must
    // retain that distinction, rather than recover the old marker's L2 rank.
    for (node, affinity, start, expected) in [
        (
            13,
            InlineGapAffinity::Before,
            50.0,
            [0.0, 10.0, 20.0, 58.0, 40.0, 30.0],
        ),
        (
            12,
            InlineGapAffinity::After,
            30.0,
            [0.0, 10.0, 20.0, 58.0, 48.0, 38.0],
        ),
    ] {
        let mut f = empty_box_fixture::<S>();
        f.wrapping.gaps.push(InlineGapGroupInput {
            anchor: InlineGapAnchor::Neighbor { node, affinity },
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        });
        let batch = f.compute();
        assert!(batch.placed_text().is_empty());
        assert_order(&batch, &[0, 1, 2, 5, 4, 3]);
        for (index, x) in expected.into_iter().enumerate() {
            assert_eq!(output(&batch, 10 + index as u32).location.x, n(x));
        }
        assert_eq!(output(&batch, 20).location.x, n(start));
        assert_eq!(output(&batch, 20).size.width, n(3.0));
        assert_eq!(output(&batch, 21).location.x, n(start + 3.0));
        assert_eq!(output(&batch, 21).size.width, n(5.0));
        let numeric = layout_mixed_inline_run(f.numeric()).unwrap();
        assert_eq!(numeric.inline_extent, n(68.0));
        for row in &numeric.atomics {
            assert_eq!(
                crate::inline::expect_logical_position(row.position).inline,
                n(expected[row.item.source_index - 10])
            );
        }
    }
}

#[test]
fn numeric_empty_box_retains_resolved_physical_neighbor_gap_f32() {
    resolved_neighbor_gap::<f32>();
}
#[test]
fn numeric_empty_box_retains_resolved_physical_neighbor_gap_f64() {
    resolved_neighbor_gap::<f64>();
}

pub(super) fn append_empty_box<S: LayoutScalar>(f: &mut Fixture<S>, start: u32, end: u32) {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (node, kind, width) in [
        (start, InlineBoundaryKind::Start, 3.0),
        (end, InlineBoundaryKind::End, 5.0),
    ] {
        f.boundary(node, kind, width);
        replace_boundary(
            f,
            node,
            InlineBoundaryInputOf::box_edge(
                kind,
                metrics(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
                axes,
            ),
        );
    }
    f.wrapping.scopes.push(WrappingScopeInput::NativePair {
        start,
        end,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    });
    f.wrapping.child_order.push(order(pair(start, end), vec![]));
}

fn implicit_body_edge_witness<S: LayoutScalar>() {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    append_empty_box(&mut f, 20, 21);
    append_empty_box(&mut f, 22, 23);
    let WrappingScopeInput::NativePair { edges, .. } = &mut f.wrapping.scopes[0] else {
        unreachable!()
    };
    edges.after = WrapEdge::Line;
    f.wrapping.child_order.insert(
        0,
        order(
            GeneratedBoxRef::Node(0),
            vec![child(pair(20, 21)), child(pair(22, 23))],
        ),
    );
    f.finish();
    // The one operand-empty body proves gap0 at both source edges. The real
    // wrapping force separates the selected cuts without adding another body.
    let batch = f.compute();
    for (node, x, baseline, width) in [
        (20, 0.0, 10.0, 3.0),
        (21, 3.0, 10.0, 5.0),
        (22, 0.0, 20.0, 3.0),
        (23, 3.0, 20.0, 5.0),
    ] {
        assert_eq!(output(&batch, node).location, Point::new(n(x), n(baseline)));
        assert_eq!(output(&batch, node).size.width, n(width));
    }
    assert_eq!(output(&batch, 0).size.height, n(20.0));
    let numeric = layout_mixed_inline_run(f.numeric()).unwrap();
    assert_eq!(numeric.lines.len(), 2);
    assert_eq!(numeric.inline_extent, n(8.0));
}

#[test]
fn numeric_implicit_gap_retains_equivalent_body_end_witness_on_suffix_f32() {
    implicit_body_edge_witness::<f32>();
}
#[test]
fn numeric_implicit_gap_retains_equivalent_body_end_witness_on_suffix_f64() {
    implicit_body_edge_witness::<f64>();
}

fn implicit_following_neighbor<S: LayoutScalar>() {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    f.atomic(
        10,
        WrapEdges {
            before: WrapEdge::Auto,
            after: WrapEdge::Line,
        },
        InlineBreakOpportunity::Prohibited,
        false,
    );
    append_empty_box(&mut f, 20, 21);
    f.atomic(
        11,
        WrapEdges::AUTO,
        InlineBreakOpportunity::Prohibited,
        false,
    );
    f.wrapping.child_order.insert(
        0,
        order(
            GeneratedBoxRef::Node(0),
            vec![
                child(GeneratedBoxRef::Node(10)),
                child(pair(20, 21)),
                child(GeneratedBoxRef::Node(11)),
            ],
        ),
    );
    f.finish();
    // A-after and B-before prove the same gap in the actual complete body.
    // Only B is retained on the second selected line; its witness is sufficient.
    let batch = f.compute();
    assert_eq!(output(&batch, 10).location, Point::new(S::ZERO, S::ZERO));
    assert_eq!(output(&batch, 11).location, Point::new(n(8.0), n(10.0)));
    assert_eq!(output(&batch, 20).location, Point::new(S::ZERO, n(20.0)));
    assert_eq!(output(&batch, 21).location, Point::new(n(3.0), n(20.0)));
    assert_eq!(output(&batch, 0).size.height, n(20.0));
    let numeric = layout_mixed_inline_run(f.numeric()).unwrap();
    assert_eq!(numeric.lines.len(), 2);
    assert_eq!(numeric.inline_extent, n(18.0));
}

#[test]
fn numeric_implicit_gap_retains_equivalent_following_neighbor_on_suffix_f32() {
    implicit_following_neighbor::<f32>();
}
#[test]
fn numeric_implicit_gap_retains_equivalent_following_neighbor_on_suffix_f64() {
    implicit_following_neighbor::<f64>();
}

fn unresolved_directional_gap<S: LayoutScalar>() {
    let f = empty_box_fixture::<S>();
    // The adjacent source operands delimit two different physical gaps after
    // L2. Neither the source cut nor positive BP chooses one of them.
    let error = match LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new()) {
        Ok(flow) => compute_layout_with_flow(&flow, 0, request(f.width))
            .expect_err("an unresolved directional gap must refuse"),
        Err(error) => error,
    };
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::MissingGap
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn numeric_unresolved_empty_box_directional_gap_refuses_f32() {
    unresolved_directional_gap::<f32>();
}
#[test]
fn numeric_unresolved_empty_box_directional_gap_refuses_f64() {
    unresolved_directional_gap::<f64>();
}

fn tied_empty_boxes<S: LayoutScalar>() {
    let mut f = empty_box_fixture::<S>();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (node, kind, width) in [
        (22, InlineBoundaryKind::Start, 3.0),
        (23, InlineBoundaryKind::End, 5.0),
    ] {
        f.boundary(node, kind, width);
        replace_boundary(
            &mut f,
            node,
            InlineBoundaryInputOf::box_edge(
                kind,
                metrics(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
                axes,
            ),
        );
    }
    f.nodes = vec![10, 11, 12, 20, 21, 22, 23, 13, 14, 15];
    reorder_fixture_participants(&mut f);
    f.wrapping.scopes.push(WrappingScopeInput::NativePair {
        start: 22,
        end: 23,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    });
    f.wrapping.child_order[0]
        .children
        .insert(4, child(pair(22, 23)));
    f.wrapping.child_order.push(order(pair(22, 23), vec![]));
    f.wrapping.gaps.push(InlineGapGroupInput {
        anchor: InlineGapAnchor::Neighbor {
            node: 13,
            affinity: InlineGapAffinity::Before,
        },
        // Explicit physical order deliberately differs from source order.
        boxes: vec![
            InlineBoxRef::NativePair { start: 22, end: 23 },
            InlineBoxRef::NativePair { start: 20, end: 21 },
        ],
    });
    f.finish();
    let batch = f.compute();
    assert_order(&batch, &[0, 1, 2, 5, 4, 3]);
    for (index, x) in [0.0, 10.0, 20.0, 66.0, 40.0, 30.0].into_iter().enumerate() {
        assert_eq!(output(&batch, 10 + index as u32).location.x, n(x));
    }
    for (node, x, width) in [
        (22, 50.0, 3.0),
        (23, 53.0, 5.0),
        (20, 58.0, 3.0),
        (21, 61.0, 5.0),
    ] {
        assert_eq!(output(&batch, node).location.x, n(x));
        assert_eq!(output(&batch, node).size.width, n(width));
    }
    assert_eq!(
        layout_mixed_inline_run(f.numeric()).unwrap().inline_extent,
        n(76.0)
    );
}

#[test]
fn numeric_tied_empty_boxes_use_explicit_physical_order_f32() {
    tied_empty_boxes::<f32>();
}
#[test]
fn numeric_tied_empty_boxes_use_explicit_physical_order_f64() {
    tied_empty_boxes::<f64>();
}

fn nested_empty_parent<S: LayoutScalar>() {
    let mut f = empty_box_fixture::<S>();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (node, kind, width) in [
        (22, InlineBoundaryKind::Start, 3.0),
        (23, InlineBoundaryKind::End, 5.0),
    ] {
        f.boundary(node, kind, width);
        replace_boundary(
            &mut f,
            node,
            InlineBoundaryInputOf::box_edge(
                kind,
                metrics(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
                axes,
            ),
        );
    }
    f.nodes = vec![10, 11, 12, 20, 22, 23, 21, 13, 14, 15];
    reorder_fixture_participants(&mut f);
    f.wrapping.scopes.push(WrappingScopeInput::NativePair {
        start: 22,
        end: 23,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    });
    f.wrapping.child_order[1].children = vec![child(pair(22, 23))];
    f.wrapping.child_order.push(order(pair(22, 23), vec![]));
    f.wrapping.gaps = vec![
        InlineGapGroupInput {
            anchor: InlineGapAnchor::Neighbor {
                node: 13,
                affinity: InlineGapAffinity::Before,
            },
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        },
        InlineGapGroupInput {
            anchor: InlineGapAnchor::EmptyParent(pair(20, 21)),
            boxes: vec![InlineBoxRef::NativePair { start: 22, end: 23 }],
        },
    ];
    f.finish();
    let batch = f.compute();
    assert_order(&batch, &[0, 1, 2, 5, 4, 3]);
    for (index, x) in [0.0, 10.0, 20.0, 66.0, 40.0, 30.0].into_iter().enumerate() {
        assert_eq!(output(&batch, 10 + index as u32).location.x, n(x));
    }
    for (node, x, width) in [
        (20, 50.0, 3.0),
        (22, 53.0, 3.0),
        (23, 56.0, 5.0),
        (21, 61.0, 5.0),
    ] {
        assert_eq!(output(&batch, node).location.x, n(x));
        assert_eq!(output(&batch, node).size.width, n(width));
    }
    assert_eq!(
        layout_mixed_inline_run(f.numeric()).unwrap().inline_extent,
        n(76.0)
    );
}

#[test]
fn numeric_nested_positive_bp_boxes_use_ordering_empty_parent_attachment_f32() {
    nested_empty_parent::<f32>();
}
#[test]
fn numeric_nested_positive_bp_boxes_use_ordering_empty_parent_attachment_f64() {
    nested_empty_parent::<f64>();
}

fn empty_terminal_body<S: LayoutScalar>() -> Fixture<S> {
    let mut f = Fixture::new(Direction::Ltr, 100.0);
    f.atomic(
        10,
        WrapEdges::AUTO,
        InlineBreakOpportunity::Prohibited,
        false,
    );
    f.tree.insert_input(
        60,
        LayoutInputOf::line_break(crate::LineBreakInputOf::new().with_metrics(metrics())),
    );
    f.nodes.push(60);
    f.participants
        .push(MixedInlineParticipantOf::ForcedLineBreak(
            crate::inline::ForcedLineBreakControlOf::new(
                60,
                InlineFlowOf::new(
                    WritingMode::HorizontalTb,
                    Direction::Ltr,
                    AvailableOf::definite(f.width),
                ),
                metrics(),
                InlineControlAlignment::Baseline,
                crate::Clear::None,
            ),
        ));
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (node, kind, width) in [
        (20, InlineBoundaryKind::Start, 3.0),
        (21, InlineBoundaryKind::End, 5.0),
    ] {
        f.boundary(node, kind, width);
        replace_boundary(
            &mut f,
            node,
            InlineBoundaryInputOf::box_edge(
                kind,
                metrics(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
                axes,
            ),
        );
    }
    f.wrapping = WrappingInput {
        native_inline_boxes: Vec::new(),
        scopes: vec![WrappingScopeInput::NativePair {
            start: 20,
            end: 21,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![
            order(
                GeneratedBoxRef::Node(0),
                vec![child(GeneratedBoxRef::Node(10)), child(pair(20, 21))],
            ),
            order(pair(20, 21), vec![]),
        ],
        gaps: Vec::new(),
    };
    f.finish();
    f
}

fn actual_terminal_body_attachment<S: LayoutScalar>() {
    for anchor in [
        InlineGapAnchor::Formatter {
            node: 0,
            edge: InlineFormatterEdge::End,
        },
        InlineGapAnchor::Delimiter {
            node: 60,
            side: InlineDelimiterSide::After,
        },
    ] {
        let mut f = empty_terminal_body::<S>();
        f.wrapping.gaps.push(InlineGapGroupInput {
            anchor,
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        });
        let batch = f.compute();
        assert_eq!(output(&batch, 10).location, Point::new(S::ZERO, S::ZERO));
        // Endpoint strips have zero block extent and sit on the control
        // baseline: second-line start10 plus its actual baseline10.
        assert_eq!(output(&batch, 20).location, Point::new(S::ZERO, n(20.0)));
        assert_eq!(output(&batch, 21).location, Point::new(n(3.0), n(20.0)));
        assert_eq!(output(&batch, 20).size.width, n(3.0));
        assert_eq!(output(&batch, 21).size.width, n(5.0));
        assert_eq!(output(&batch, 0).size.height, n(20.0));
        assert!(batch.placed_text().is_empty());
        let numeric = layout_mixed_inline_run(f.numeric()).unwrap();
        assert_eq!(numeric.lines.len(), 2);
        assert_eq!(numeric.inline_extent, n(10.0));
    }
    for anchor in [
        InlineGapAnchor::Formatter {
            node: 0,
            edge: InlineFormatterEdge::Start,
        },
        InlineGapAnchor::Delimiter {
            node: 60,
            side: InlineDelimiterSide::Before,
        },
    ] {
        let mut f = empty_terminal_body::<S>();
        f.wrapping.gaps.push(InlineGapGroupInput {
            anchor,
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        });
        let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
            .err()
            .expect("an attachment in the preceding body is foreign to the terminal empty box");
        assert!(
            matches!(
                error.kind(),
                crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                    WrappingScopeError::InvalidGapAnchor
                ))
            ),
            "{error:?}"
        );
    }
}

#[test]
fn numeric_gap_formatter_and_delimiter_sides_retain_operand_empty_terminal_body_f32() {
    actual_terminal_body_attachment::<f32>();
}
#[test]
fn numeric_gap_formatter_and_delimiter_sides_retain_operand_empty_terminal_body_f64() {
    actual_terminal_body_attachment::<f64>();
}

fn invalid_attachments<S: LayoutScalar>() {
    for (anchor, expected) in [
        (
            InlineGapAnchor::Neighbor {
                node: 99,
                affinity: InlineGapAffinity::Before,
            },
            WrappingScopeError::NodeNotReachable,
        ),
        (
            InlineGapAnchor::Formatter {
                node: 99,
                edge: InlineFormatterEdge::Start,
            },
            WrappingScopeError::NodeNotReachable,
        ),
        (
            InlineGapAnchor::Delimiter {
                node: 99,
                side: InlineDelimiterSide::After,
            },
            WrappingScopeError::NodeNotReachable,
        ),
        (
            InlineGapAnchor::EmptyParent(GeneratedBoxRef::Node(99)),
            WrappingScopeError::NodeNotReachable,
        ),
        (
            InlineGapAnchor::EmptyParent(pair(20, 99)),
            WrappingScopeError::NodeNotReachable,
        ),
        (
            InlineGapAnchor::EmptyParent(pair(99, 21)),
            WrappingScopeError::NodeNotReachable,
        ),
        (
            InlineGapAnchor::Formatter {
                node: 10,
                edge: InlineFormatterEdge::Start,
            },
            WrappingScopeError::InvalidGapAnchor,
        ),
        (
            InlineGapAnchor::EmptyParent(pair(10, 11)),
            WrappingScopeError::InvalidGapAnchor,
        ),
        (
            InlineGapAnchor::EmptyParent(GeneratedBoxRef::Node(0)),
            WrappingScopeError::InvalidGapAnchor,
        ),
    ] {
        let mut f = empty_box_fixture::<S>();
        f.wrapping.gaps.push(InlineGapGroupInput {
            anchor,
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        });
        let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
            .err()
            .expect("a missing neighbor or operand-bearing parent is not empty-gap authority");
        let crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
            actual,
        )) = error.kind()
        else {
            panic!("{error:?}");
        };
        assert_eq!(actual, &expected);
    }
    for reference in [
        InlineBoxRef::NativePair { start: 20, end: 99 },
        InlineBoxRef::NativePair { start: 99, end: 21 },
    ] {
        let mut f = empty_box_fixture::<S>();
        f.wrapping.gaps.push(InlineGapGroupInput {
            anchor: InlineGapAnchor::Neighbor {
                node: 13,
                affinity: InlineGapAffinity::Before,
            },
            boxes: vec![reference],
        });
        let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
            .err()
            .expect("a missing referenced endpoint remains an absent-node error");
        assert!(
            matches!(
                error.kind(),
                crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                    WrappingScopeError::NodeNotReachable
                ))
            ),
            "{error:?}"
        );
    }
    let mut f = empty_box_fixture::<S>();
    for (node, affinity) in [
        (13, InlineGapAffinity::Before),
        (12, InlineGapAffinity::After),
    ] {
        f.wrapping.gaps.push(InlineGapGroupInput {
            anchor: InlineGapAnchor::Neighbor { node, affinity },
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        });
    }
    let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
        .err()
        .expect("one real box cannot acquire conflicting resolved gaps");
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::ConflictingGap
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn numeric_empty_gap_rejects_foreign_nonempty_and_conflicting_authority_f32() {
    invalid_attachments::<f32>();
}
#[test]
fn numeric_empty_gap_rejects_foreign_nonempty_and_conflicting_authority_f64() {
    invalid_attachments::<f64>();
}

fn gap_outside_forced_selection<S: LayoutScalar>() {
    let mut f = empty_box_fixture::<S>();
    f.wrapping.gaps.push(InlineGapGroupInput {
        anchor: InlineGapAnchor::Neighbor {
            node: 13,
            affinity: InlineGapAffinity::Before,
        },
        boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
    });
    let WrappingScopeInput::NativePair { edges, .. } = &mut f.wrapping.scopes[0] else {
        unreachable!()
    };
    // A genuine wrapping force ends the selected line after this source-empty
    // pair. Its anchor is body-valid, but belongs to the following line. No
    // clamping or moving the pair through the forced cut is permitted.
    edges.after = WrapEdge::Line;
    let flow = f.flow();
    let error = compute_layout_with_flow(&flow, 0, request(f.width))
        .expect_err("body membership alone cannot authenticate the selected gap");
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::GapOutsideSelection
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn numeric_empty_gap_outside_forced_selected_line_refuses_f32() {
    gap_outside_forced_selection::<f32>();
}
#[test]
fn numeric_empty_gap_outside_forced_selected_line_refuses_f64() {
    gap_outside_forced_selection::<f64>();
}

fn ordinary_contributions<S: LayoutScalar>() {
    for [start, end] in [[3.0, 5.0], [0.0, 0.0]] {
        let mut f = super::bidi_metadata::split_box_fixture::<S>();
        for (node, kind, width) in [
            (20, InlineBoundaryKind::Start, start),
            (21, InlineBoundaryKind::End, end),
        ] {
            let edge = InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap();
            let boundary = InlineBoundaryInputOf::contribution(kind, metrics(), edge).unwrap();
            assert_eq!(boundary.parent_edge_axes(), None);
            replace_boundary(&mut f, node, boundary);
        }
        // A wrapping NativePair alone grants no normal-family classification,
        // including when both genuine Contributions have zero advance.
        let batch = f.compute();
        assert_order(&batch, &[0, 1, 2, 3, 5, 4]);
        for (index, x) in [
            0.0,
            10.0,
            20.0 + start,
            30.0 + start,
            50.0 + start + end,
            40.0 + start + end,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(output(&batch, 10 + index as u32).location.x, n(x));
        }
        assert_eq!(
            layout_mixed_inline_run(f.numeric()).unwrap().inline_extent,
            n(60.0 + start + end)
        );
    }
}

#[test]
fn numeric_ordinary_contributions_retain_complete_supplied_order_f32() {
    ordinary_contributions::<f32>();
}
#[test]
fn numeric_ordinary_contributions_retain_complete_supplied_order_f64() {
    ordinary_contributions::<f64>();
}
