//! An ordering-empty normal box retains identity through a real float cut.
use super::*;

#[path = "empty_float_suffix/explicit_start.rs"]
mod explicit_start;

fn empty_float_fixture<S: LayoutScalar>(nested: bool) -> Fixture<S> {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    // Maintained checked recipe: actual StartBP3 / EndBP5 BoxEdges, physical
    // line-height10/baseline10, matching LTR formatter/parent, genuine Auto pair.
    gap_attachments::append_empty_box(&mut f, 20, 21);
    if nested {
        gap_attachments::append_empty_box(&mut f, 22, 23);
    }
    f.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            float: crate::Float::Left,
            size: Size::new(PreferredSizeOf::px(n(8.0)), PreferredSizeOf::px(n(4.0))),
            ..NodeInputOf::default()
        }),
    );
    f.nodes = if nested {
        vec![20, 22, 23, 30, 21]
    } else {
        vec![20, 30, 21]
    };
    f.participants.sort_by_key(|row| match row {
        MixedInlineParticipantOf::Boundary(control) => f
            .nodes
            .iter()
            .position(|node| *node as usize == control.source_index())
            .unwrap(),
        _ => unreachable!("only genuine endpoint controls are supplied here"),
    });
    f.finish();
    let mut inside = Vec::new();
    if nested {
        inside.push(child(pair(22, 23)));
    }
    inside.push(GeneratedChildBoxInput {
        box_: GeneratedBoxRef::Node(30),
        participation: GeneratedBoxParticipation::OutOfFlow,
    });
    f.wrapping.child_order = vec![
        order(GeneratedBoxRef::Node(0), vec![child(pair(20, 21))]),
        order(pair(20, 21), inside),
    ];
    if nested {
        f.wrapping.child_order.push(order(pair(22, 23), vec![]));
    }
    assert_eq!(
        f.nodes,
        if nested {
            vec![20, 22, 23, 30, 21]
        } else {
            vec![20, 30, 21]
        }
    );
    assert!(
        f.wrapping.gaps.is_empty(),
        "no extra explicit attachment is introduced"
    );
    assert!(f.wrapping.native_inline_boxes.is_empty());
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut endpoints = vec![
        (20, InlineBoundaryKind::Start, 3.0),
        (21, InlineBoundaryKind::End, 5.0),
    ];
    if nested {
        endpoints.extend([
            (22, InlineBoundaryKind::Start, 3.0),
            (23, InlineBoundaryKind::End, 5.0),
        ]);
    }
    for (node, kind, width) in endpoints {
        let crate::LayoutInputViewOf::InlineBoundary(input) =
            crate::LayoutTree::layout_input(&f.tree, node)
        else {
            panic!("the original endpoint must remain an actual inline-boundary operation");
        };
        assert_eq!(input.kind(), kind);
        assert_eq!(input.parent_edge_axes(), Some(axes));
        assert_eq!(input.metrics(), metrics());
        assert_eq!(input.edge().border_padding_advance(), n(width));
        assert_eq!(
            (input.writing_mode(), input.direction()),
            (WritingMode::HorizontalTb, Direction::Ltr)
        );
    }
    let float = crate::LayoutTree::node_input(&f.tree, 30);
    assert_eq!(float.display, crate::Display::Block);
    assert_eq!(float.float, crate::Float::Left);
    assert_eq!(float.position, crate::Position::Relative);
    assert_eq!(float.atomic_inline_participation, None);
    assert_eq!(
        float.size,
        Size::new(PreferredSizeOf::px(n(8.0)), PreferredSizeOf::px(n(4.0)))
    );
    // There are no atomics, supplied text, body levels or pseudo operands. The
    // stored inline controls are structural BoxEdges; the actual float
    // is excluded out-of-flow metadata, not a paragraph-body delimiter.
    assert_eq!(f.participants.len(), if nested { 4 } else { 2 });
    assert!(
        f.participants
            .iter()
            .all(|row| matches!(row, MixedInlineParticipantOf::Boundary(_)))
    );
    f
}

fn ordinary_empty_float_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    nested: bool,
) {
    assert!(batch.placed_text().is_empty());
    // Positive BP makes each control-only fitting run nonphantom. Each uses the
    // real10/0 metric band; endpoint locations are zero-height baseline points.
    boundary_position(batch, 20, 0.0, 0.0, 3.0);
    boundary_position(batch, 21, 8.0, 10.0, 5.0);
    assert_eq!(output(batch, 20).source_index.get(), 0);
    assert_eq!(
        output(batch, 21).source_index.get(),
        if nested { 4 } else { 2 }
    );
    if nested {
        boundary_position(batch, 22, 3.0, 0.0, 3.0);
        boundary_position(batch, 23, 6.0, 0.0, 5.0);
        assert_eq!(output(batch, 22).source_index.get(), 1);
        assert_eq!(output(batch, 23).source_index.get(), 2);
    }
    let float = output(batch, 30);
    // The prefix BP (3, or3+3+5 when nested) completes a ten-pixel line. A real8x4 left float then
    // starts at block10; its exclusion shifts the End-only run's strip to8..13.
    assert_eq!(float.location, Point::new(S::ZERO, n(10.0)));
    assert_eq!(float.size, Size::new(n(8.0), n(4.0)));
    assert_eq!(float.source_index.get(), if nested { 3 } else { 1 });
    assert_eq!(output(batch, 0).size, Size::new(n(100.0), n(20.0)));
}

fn empty_suffix_member<S: LayoutScalar>(
    piece: crate::NativeInlineBoxPieceView<'_, u32, S>,
    leaf: GeneratedBoxRef<u32>,
) {
    let members: Vec<_> = piece.selected_members().collect();
    assert_eq!(
        members.len(),
        1,
        "one real terminal empty-box incidence, not a new operand"
    );
    assert_eq!(members[0].empty_box(), Some(leaf));
    assert_eq!(members[0].node(), None);
    assert_eq!(members[0].child_rank(), None);
    assert_eq!(members[0].operand_ordinal(), None);
}

fn selected_closing_suffix<S: LayoutScalar>() {
    let f = empty_float_fixture::<S>(false);
    let ordinary_flow = f.flow();
    let prior = compute_layout_with_flow(&ordinary_flow, 0, request(n(100.0)))
        .expect("the genuine ordinary Start/float/End sequence must complete first");
    ordinary_empty_float_geometry(&prior, false);
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_pieces().count(), 0);

    // Only the explicit family demand changes. The same immutable tree, endpoint
    // operations, real float and complete generated-child declarations remain.
    let mut demanded = f.wrapping.clone();
    demanded
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = LayoutFlowInput::try_new(&f.tree, 0, vec![], demanded, Vec::new())
        .expect("the actual complete normal family must admit without fabricated content");
    let result = compute_layout_with_flow(&flow, 0, request(n(100.0)));
    // A failed family computation must leave the prior ordinary publication and
    // its still-valid operation usable; this check also precedes the success gate.
    ordinary_empty_float_geometry(&prior, false);
    let retry = compute_layout_with_flow(&ordinary_flow, 0, request(n(100.0))).unwrap();
    ordinary_empty_float_geometry(&retry, false);
    let accepted = result
        .expect("the selected closing End/body-end witness must retain the genuine empty fragment");
    ordinary_empty_float_geometry(&accepted, false);
    for node in [0, 20, 30, 21] {
        assert_eq!(
            output(&accepted, node).location,
            output(&prior, node).location
        );
        assert_eq!(output(&accepted, node).size, output(&prior, node).size);
        assert_eq!(
            output(&accepted, node).source_index,
            output(&prior, node).source_index
        );
    }
    let families: Vec<_> = accepted.native_inline_box_families().collect();
    assert_eq!(families.len(), 1);
    let family = &families[0];
    assert_eq!(family.formatter(), 0);
    let fragments: Vec<_> = family
        .fragments()
        .filter(|fragment| fragment.owner() == pair(20, 21))
        .collect();
    assert_eq!(
        fragments.len(),
        1,
        "the float does not split a body or donate a normal operand"
    );
    assert_eq!(
        family
            .fragments()
            .filter(|fragment| fragment.owner() == GeneratedBoxRef::Node(30))
            .count(),
        0
    );
    let mut pieces: Vec<_> = accepted
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == pair(20, 21))
        .collect();
    pieces.sort_by_key(|piece| piece.run_source_range().start);
    assert_eq!(
        pieces.len(),
        2,
        "both genuine control-only runs retain P's piece association"
    );
    let opening = pieces[0];
    let closing = pieces[1];
    assert_eq!(opening.family(), family);
    assert_eq!(closing.family(), family);
    assert_eq!(opening.fragment(), fragments[0]);
    assert_eq!(closing.fragment(), fragments[0]);
    let root = opening.fragment().body_root();
    assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(root.parent(), None);
    assert_eq!(opening.fragment().parent(), Some(root.clone()));
    assert_eq!(closing.fragment().parent(), Some(root.clone()));
    assert_eq!(closing.fragment().body_root(), root);
    assert_eq!(
        opening
            .fragment()
            .nearest_common_ancestor(&closing.fragment())
            .unwrap(),
        fragments[0]
    );
    assert_eq!((opening.line_index(), closing.line_index()), (0, 0));
    assert_eq!(opening.run_source_range(), 0..1);
    assert_eq!(opening.selected_source_range(), 0..1);
    assert_eq!(closing.run_source_range(), 2..3);
    assert_eq!(closing.selected_source_range(), 2..3);
    assert!(
        !closing.selected_source_range().contains(&0),
        "the closing piece cannot borrow the opening rank"
    );
    empty_suffix_member(opening, pair(20, 21));
    // The empty row has its actual opening site at0. The selected suffix only
    // carries P's real closing edge, not a donated or duplicated membership.
    assert_eq!(closing.selected_members().count(), 0);
    assert_eq!(opening.source_aliases().count(), 0);
    assert_eq!(closing.source_aliases().count(), 0);
    assert_eq!(
        edges(opening),
        [(20, 0, InlineBoundaryKind::Start, 0.0, 3.0)]
    );
    assert_eq!(
        edges(closing),
        [(21, 2, InlineBoundaryKind::End, 8.0, 13.0)]
    );
    ordinary_empty_float_geometry(&prior, false);
}

#[test]
fn ordering_empty_native_family_keeps_selected_closing_suffix_around_real_float_f32() {
    selected_closing_suffix::<f32>();
}
#[test]
fn ordering_empty_native_family_keeps_selected_closing_suffix_around_real_float_f64() {
    selected_closing_suffix::<f64>();
}

fn nested_selected_closing_suffix<S: LayoutScalar>() {
    let f = empty_float_fixture::<S>(true);
    let ordinary_flow = f.flow();
    let prior = compute_layout_with_flow(&ordinary_flow, 0, request(n(100.0)))
        .expect("ordinary nested empty boxes must complete both real runs around the float");
    ordinary_empty_float_geometry(&prior, true);
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_pieces().count(), 0);
    let mut demanded = f.wrapping.clone();
    demanded
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = LayoutFlowInput::try_new(&f.tree, 0, vec![], demanded, Vec::new()).unwrap();
    let result = compute_layout_with_flow(&flow, 0, request(n(100.0)));
    ordinary_empty_float_geometry(&prior, true);
    let retry = compute_layout_with_flow(&ordinary_flow, 0, request(n(100.0))).unwrap();
    ordinary_empty_float_geometry(&retry, true);
    let accepted = result
        .expect("P's selected End must retain P without borrowing the unselected C row or edges");
    ordinary_empty_float_geometry(&accepted, true);
    for node in [0, 20, 22, 23, 30, 21] {
        assert_eq!(
            output(&accepted, node).location,
            output(&prior, node).location
        );
        assert_eq!(output(&accepted, node).size, output(&prior, node).size);
        assert_eq!(
            output(&accepted, node).source_index,
            output(&prior, node).source_index
        );
    }
    let families: Vec<_> = accepted.native_inline_box_families().collect();
    assert_eq!(families.len(), 1);
    let family = &families[0];
    assert_eq!(family.formatter(), 0);
    let p_fragments: Vec<_> = family
        .fragments()
        .filter(|fragment| fragment.owner() == pair(20, 21))
        .collect();
    let c_fragments: Vec<_> = family
        .fragments()
        .filter(|fragment| fragment.owner() == pair(22, 23))
        .collect();
    assert_eq!(p_fragments.len(), 1);
    assert_eq!(c_fragments.len(), 1);
    let p = &p_fragments[0];
    let c = &c_fragments[0];
    assert_eq!(c.parent(), Some(p.clone()));
    let root = p.body_root();
    assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(root.parent(), None);
    assert_eq!(p.parent(), Some(root.clone()));
    assert_eq!(c.body_root(), root);
    assert_eq!(p.nearest_common_ancestor(c).unwrap(), *p);
    let mut p_pieces: Vec<_> = accepted
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == pair(20, 21))
        .collect();
    p_pieces.sort_by_key(|piece| piece.run_source_range().start);
    assert_eq!(p_pieces.len(), 2);
    let opening = p_pieces[0];
    let closing = p_pieces[1];
    assert_eq!(opening.fragment(), *p);
    assert_eq!(closing.fragment(), *p);
    assert_eq!(opening.family(), family);
    assert_eq!(closing.family(), family);
    assert_eq!((opening.line_index(), closing.line_index()), (0, 0));
    assert_eq!(opening.run_source_range(), 0..3);
    assert_eq!(opening.selected_source_range(), 0..3);
    assert_eq!(closing.run_source_range(), 4..5);
    assert_eq!(closing.selected_source_range(), 4..5);
    // P has no redundant empty row: its opening membership is the real empty
    // child C. That row's opening at raw1 is absent from the closing selection.
    empty_suffix_member(opening, pair(22, 23));
    assert_eq!(closing.selected_members().count(), 0);
    assert_eq!(opening.source_aliases().count(), 0);
    assert_eq!(closing.source_aliases().count(), 0);
    assert_eq!(
        edges(opening),
        [(20, 0, InlineBoundaryKind::Start, 0.0, 3.0)]
    );
    assert_eq!(
        edges(closing),
        [(21, 4, InlineBoundaryKind::End, 8.0, 13.0)]
    );
    let c_pieces: Vec<_> = accepted
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == pair(22, 23))
        .collect();
    assert_eq!(
        c_pieces.len(),
        1,
        "C cannot acquire a closing piece from P's body-end witness"
    );
    let child = c_pieces[0];
    assert_eq!(child.fragment(), *c);
    assert_eq!(child.run_source_range(), 0..3);
    assert_eq!(child.selected_source_range(), 0..3);
    assert_eq!(child.line_index(), 0);
    empty_suffix_member(child, pair(22, 23));
    assert_eq!(child.source_aliases().count(), 0);
    let child_edges: Vec<_> = child
        .executed_edges()
        .map(|edge| {
            assert_eq!(edge.owner(), pair(22, 23));
            assert!(!edge.is_source_alias());
            assert_eq!(edge.source_alias_position(), None);
            assert_eq!(
                edge.flow_axes(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
            );
            let flow = edge.flow_interval().unwrap();
            assert_eq!(edge.strip_interval(), Some(flow));
            (
                edge.node(),
                edge.child_rank(),
                edge.kind(),
                flow.0.to_f64(),
                flow.1.to_f64(),
            )
        })
        .collect();
    assert_eq!(
        child_edges,
        [
            (22, 1, InlineBoundaryKind::Start, 3.0, 6.0),
            (23, 2, InlineBoundaryKind::End, 6.0, 11.0)
        ]
    );
    // Inspect every actual suffix association, not just P's filtered view: no C
    // membership, fragment, physical edge or borrowed first-run coordinates may
    // be donated to the End-only run by relaxing the opening-site condition.
    for piece in accepted
        .native_inline_box_pieces()
        .filter(|piece| piece.run_source_range() == (4..5))
    {
        assert_ne!(piece.fragment().owner(), pair(22, 23));
        assert_eq!(piece.selected_members().count(), 0);
        for edge in piece.executed_edges() {
            assert_eq!(edge.node(), 21);
            assert_eq!(edge.child_rank(), 4);
            assert_eq!(edge.owner(), pair(20, 21));
            assert_eq!(edge.kind(), InlineBoundaryKind::End);
            assert_eq!(edge.flow_interval(), Some((n(8.0), n(13.0))));
            assert_eq!(edge.strip_interval(), Some((n(8.0), n(13.0))));
        }
    }
    ordinary_empty_float_geometry(&prior, true);
}

#[test]
fn nested_empty_native_family_closes_parent_without_donating_child_suffix_f32() {
    nested_selected_closing_suffix::<f32>();
}
#[test]
fn nested_empty_native_family_closes_parent_without_donating_child_suffix_f64() {
    nested_selected_closing_suffix::<f64>();
}
