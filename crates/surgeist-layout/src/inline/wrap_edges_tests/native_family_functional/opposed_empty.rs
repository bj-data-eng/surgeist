//! Opposed parent axes and authenticated empty native box membership.
use super::*;
use crate::{InlineGapAffinity, InlineGapAnchor, InlineGapGroupInput};

fn physical_edge<S: LayoutScalar>(
    f: &mut Fixture<S>,
    node: u32,
    kind: InlineBoundaryKind,
    width: f64,
    parent: FlowAxes,
) {
    let input = InlineBoundaryInputOf::box_edge(
        kind,
        metrics(),
        InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
        parent,
    )
    .with_writing_mode(WritingMode::HorizontalTb)
    .with_direction(Direction::Ltr);
    assert_eq!(input.parent_edge_axes(), Some(parent));
    assert_eq!(
        input.direction(),
        Direction::Ltr,
        "boundary axes remain the actual formatter axes"
    );
    gap_attachments::replace_boundary(f, node, input);
}

fn opposed_parent<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], true);
    let formatter = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let parent = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    // Add a genuine normal parent P. C's RTL parent fact does not pretend that
    // the containing LTR Block formatter itself has RTL axes. P's own endpoints
    // still use Root-LTR parent axes and contribute affirmative zero BP.
    f.boundary(22, InlineBoundaryKind::Start, 0.0);
    f.boundary(23, InlineBoundaryKind::End, 0.0);
    physical_edge(&mut f, 22, InlineBoundaryKind::Start, 0.0, formatter);
    physical_edge(&mut f, 23, InlineBoundaryKind::End, 0.0, formatter);
    physical_edge(&mut f, 20, InlineBoundaryKind::Start, 3.0, parent);
    physical_edge(&mut f, 21, InlineBoundaryKind::End, 5.0, parent);
    f.nodes = vec![22, 10, 11, 20, 12, 13, 21, 14, 15, 23];
    f.participants.sort_by_key(|row| {
        let source = match row {
            MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
            MixedInlineParticipantOf::Boundary(control) => control.source_index(),
            _ => unreachable!("the genuine fixture contains no visible break"),
        };
        f.nodes
            .iter()
            .position(|node| *node as usize == source)
            .unwrap()
    });
    f.wrapping.scopes.push(WrappingScopeInput::NativePair {
        start: 22,
        end: 23,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    });
    f.wrapping.child_order = vec![
        order(GeneratedBoxRef::Node(0), vec![child(pair(22, 23))]),
        order(
            pair(22, 23),
            vec![
                child(GeneratedBoxRef::Node(10)),
                child(GeneratedBoxRef::Node(11)),
                child(pair(20, 21)),
                child(GeneratedBoxRef::Node(14)),
                child(GeneratedBoxRef::Node(15)),
            ],
        ),
        order(
            pair(20, 21),
            vec![
                child(GeneratedBoxRef::Node(12)),
                child(GeneratedBoxRef::Node(13)),
            ],
        ),
    ];
    f.finish();
    assert_eq!(
        crate::LayoutTree::node_input(&f.tree, 0).direction,
        Direction::Ltr
    );
    for (index, node) in (10..16u32).enumerate() {
        assert_eq!(
            crate::LayoutTree::node_input(&f.tree, node)
                .atomic_inline_participation
                .unwrap()
                .bidi_level()
                .get(),
            u8::from(index >= 3)
        );
    }
    let ordinary = f.flow();
    let prior = compute_layout_with_flow(&ordinary, 0, request(n(100.0))).unwrap();
    let expected_wide = [
        (0.0, 0.0),
        (10.0, 0.0),
        (25.0, 0.0),
        (55.0, 0.0),
        (45.0, 0.0),
        (35.0, 0.0),
    ];
    assert_eq!(atom_positions(&prior), expected_wide);
    boundary_position(&prior, 20, 65.0, 0.0, 3.0);
    boundary_position(&prior, 21, 20.0, 0.0, 5.0);

    let mut demanded = f.wrapping.clone();
    demanded
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = LayoutFlowInput::try_new(&f.tree, 0, vec![], demanded, Vec::new()).unwrap();
    let wide = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert_eq!(atom_positions(&wide), expected_wide);
    boundary_position(&wide, 20, 65.0, 0.0, 3.0);
    boundary_position(&wide, 21, 20.0, 0.0, 5.0);
    let c = box_piece(&wide, 12);
    let a = box_piece(&wide, 13);
    assert_eq!(member_ids(c), [(12, 4, 2)]);
    assert_eq!(member_ids(a), [(13, 5, 3)]);
    assert_ne!(c.fragment(), a.fragment());
    assert_eq!(c.fragment().parent().unwrap().owner(), pair(22, 23));
    assert_eq!(a.fragment().parent(), c.fragment().parent());
    // RTL parent Start is physical RIGHT and End is physical LEFT. These are
    // executed-edge associations, not the logical source-frontier components.
    assert_eq!(edges(c), [(21, 6, InlineBoundaryKind::End, 20.0, 25.0)]);
    assert_eq!(edges(a), [(20, 3, InlineBoundaryKind::Start, 65.0, 68.0)]);
    let narrow = compute_layout_with_flow(&flow, 0, request(n(30.0))).unwrap();
    assert_eq!(
        atom_positions(&narrow),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (5.0, 10.0),
            (15.0, 10.0),
            (10.0, 20.0),
            (0.0, 20.0)
        ]
    );
    boundary_position(&narrow, 20, 25.0, 10.0, 3.0);
    boundary_position(&narrow, 21, 0.0, 10.0, 5.0);
    let cn = box_piece(&narrow, 12);
    let an = box_piece(&narrow, 13);
    assert_eq!(cn.fragment(), c.fragment());
    assert_eq!(an.fragment(), a.fragment());
    assert_eq!(edges(cn), [(21, 6, InlineBoundaryKind::End, 0.0, 5.0)]);
    assert_eq!(edges(an), [(20, 3, InlineBoundaryKind::Start, 25.0, 28.0)]);
    assert_eq!(atom_positions(&prior), expected_wide);
}

#[test]
fn rtl_inline_parent_keeps_ltr_content_order_and_physical_component_edges() {
    opposed_parent::<f32>();
    opposed_parent::<f64>();
}

fn empty_zero_bp<S: LayoutScalar>() {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    gap_attachments::append_empty_box(&mut f, 20, 21);
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    physical_edge(&mut f, 20, InlineBoundaryKind::Start, 0.0, axes);
    physical_edge(&mut f, 21, InlineBoundaryKind::End, 0.0, axes);
    f.atomic(
        10,
        WrapEdges::AUTO,
        InlineBreakOpportunity::Prohibited,
        false,
    );
    f.wrapping.child_order.insert(
        0,
        order(
            GeneratedBoxRef::Node(0),
            vec![child(pair(20, 21)), child(GeneratedBoxRef::Node(10))],
        ),
    );
    f.wrapping.gaps.push(InlineGapGroupInput {
        anchor: InlineGapAnchor::Neighbor {
            node: 10,
            affinity: InlineGapAffinity::Before,
        },
        boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
    });
    f.finish();
    assert_eq!(f.nodes, [20, 21, 10]);
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = f.flow();
    let accepted = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert!(accepted.placed_text().is_empty());
    assert_eq!(output(&accepted, 10).location, Point::new(S::ZERO, S::ZERO));
    assert_eq!(output(&accepted, 10).size, Size::splat(n(10.0)));
    boundary_position(&accepted, 20, 0.0, 0.0, 0.0);
    boundary_position(&accepted, 21, 0.0, 0.0, 0.0);
    let pieces = accepted
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == pair(20, 21))
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    let piece = pieces[0];
    assert_eq!(
        piece.fragment().parent().unwrap().owner(),
        GeneratedBoxRef::Node(0)
    );
    assert_eq!(piece.line_index(), 0);
    assert_eq!(piece.run_source_range(), 0..3);
    assert_eq!(piece.selected_source_range(), 0..3);
    let members = piece.selected_members().collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].empty_box(), Some(pair(20, 21)));
    assert_eq!(members[0].node(), None);
    assert_eq!(members[0].child_rank(), None);
    assert_eq!(members[0].operand_ordinal(), None);
    assert_eq!(
        edges(piece),
        [
            (20, 0, InlineBoundaryKind::Start, 0.0, 0.0),
            (21, 1, InlineBoundaryKind::End, 0.0, 0.0),
        ]
    );
    // Same maintained gap_outside_forced_selection law: explicit Neighbor
    // authority is body-valid but cannot be borrowed onto the preceding cut.
    let mut forced = f.wrapping.clone();
    let WrappingScopeInput::NativePair { edges, .. } = &mut forced.scopes[0] else {
        unreachable!()
    };
    edges.after = WrapEdge::Line;
    let forced_flow = LayoutFlowInput::try_new(&f.tree, 0, vec![], forced, Vec::new()).unwrap();
    let error = compute_layout_with_flow(&forced_flow, 0, request(n(100.0)))
        .expect_err("an unselected following neighbor cannot witness the empty box's selected gap");
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
            WrappingScopeError::GapOutsideSelection
        ))
    ));
    assert_eq!(piece.fragment().owner(), pair(20, 21));
    assert_eq!(output(&accepted, 10).location, Point::new(S::ZERO, S::ZERO));
}

#[test]
fn zero_bp_empty_family_retains_real_membership_and_selected_neighbor_authority() {
    empty_zero_bp::<f32>();
    empty_zero_bp::<f64>();
}
