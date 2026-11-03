//! Genuine nested empty incidence, physical tie order and zero-size operands.
use super::*;
use crate::{InlineGapAffinity, InlineGapAnchor, InlineGapGroupInput};

fn real_source_order<S: LayoutScalar>(f: &mut Fixture<S>, nodes: &[u32]) {
    f.nodes = nodes.to_vec();
    f.participants.sort_by_key(|row| {
        let source = match row {
            MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
            MixedInlineParticipantOf::Boundary(control) => control.source_index(),
            _ => unreachable!("no visible delimiter in this genuine fixture"),
        };
        nodes
            .iter()
            .position(|node| *node as usize == source)
            .unwrap()
    });
    f.finish();
}

fn selected_empty<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    owner: GeneratedBoxRef<u32>,
) -> crate::NativeInlineBoxPieceView<'_, u32, S> {
    let found = batch
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1);
    found[0]
}

fn empty_identity<S: LayoutScalar>(
    piece: crate::NativeInlineBoxPieceView<'_, u32, S>,
    leaf: GeneratedBoxRef<u32>,
) {
    let members = piece.selected_members().collect::<Vec<_>>();
    assert_eq!(
        members.len(),
        1,
        "one genuine leaf row, no redundant ancestor placeholder"
    );
    assert_eq!(members[0].empty_box(), Some(leaf));
    assert_eq!(members[0].node(), None);
    assert_eq!(members[0].child_rank(), None);
    assert_eq!(members[0].operand_ordinal(), None);
}

fn actual_edges<S: LayoutScalar>(
    piece: crate::NativeInlineBoxPieceView<'_, u32, S>,
    owner: GeneratedBoxRef<u32>,
) -> Vec<(u32, InlineBoundaryKind, f64, f64)> {
    let mut rows = piece
        .executed_edges()
        .map(|edge| {
            assert_eq!(edge.owner(), owner);
            assert!(!edge.is_source_alias());
            assert_eq!(
                edge.flow_axes(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
            );
            let flow = edge.flow_interval().unwrap();
            assert_eq!(edge.strip_interval(), Some(flow));
            (edge.node(), edge.kind(), flow.0.to_f64(), flow.1.to_f64())
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0);
    rows
}

fn nested_leaf<S: LayoutScalar>() {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    gap_attachments::append_empty_box(&mut f, 20, 21);
    gap_attachments::append_empty_box(&mut f, 22, 23);
    f.atomic(
        10,
        WrapEdges::AUTO,
        InlineBreakOpportunity::Prohibited,
        false,
    );
    f.wrapping.child_order = vec![
        order(
            GeneratedBoxRef::Node(0),
            vec![child(pair(20, 21)), child(GeneratedBoxRef::Node(10))],
        ),
        order(pair(20, 21), vec![child(pair(22, 23))]),
        order(pair(22, 23), vec![]),
    ];
    f.wrapping.gaps = vec![
        InlineGapGroupInput {
            anchor: InlineGapAnchor::Neighbor {
                node: 10,
                affinity: InlineGapAffinity::Before,
            },
            boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
        },
        InlineGapGroupInput {
            anchor: InlineGapAnchor::EmptyParent(pair(20, 21)),
            boxes: vec![InlineBoxRef::NativePair { start: 22, end: 23 }],
        },
    ];
    real_source_order(&mut f, &[20, 22, 23, 21, 10]);
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = f.flow();
    let batch = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert!(batch.placed_text().is_empty());
    // Maintained nested-empty recipe at gap0: outerStart3, innerStart3,
    // innerEnd5, outerEnd5. The real atomic follows sixteen pixels of BP.
    for (node, x, width) in [
        (20, 0.0, 3.0),
        (22, 3.0, 3.0),
        (23, 6.0, 5.0),
        (21, 11.0, 5.0),
    ] {
        boundary_position(&batch, node, x, 0.0, width);
    }
    assert_eq!(output(&batch, 10).location, Point::new(n(16.0), S::ZERO));
    assert_eq!(output(&batch, 10).size, Size::splat(n(10.0)));
    let p = selected_empty(&batch, pair(20, 21));
    let c = selected_empty(&batch, pair(22, 23));
    empty_identity(p, pair(22, 23));
    empty_identity(c, pair(22, 23));
    assert_ne!(p.fragment(), c.fragment());
    assert_eq!(c.fragment().parent(), Some(p.fragment()));
    assert_eq!(
        p.fragment().parent().unwrap().owner(),
        GeneratedBoxRef::Node(0)
    );
    assert_eq!(
        p.fragment().nearest_common_ancestor(&c.fragment()).unwrap(),
        p.fragment()
    );
    assert_eq!(p.run_source_range(), 0..5);
    assert_eq!(c.selected_source_range(), 0..5);
    assert_eq!(
        actual_edges(p, pair(20, 21)),
        [
            (20, InlineBoundaryKind::Start, 0.0, 3.0),
            (21, InlineBoundaryKind::End, 11.0, 16.0),
        ]
    );
    assert_eq!(
        actual_edges(c, pair(22, 23)),
        [
            (22, InlineBoundaryKind::Start, 3.0, 6.0),
            (23, InlineBoundaryKind::End, 6.0, 11.0),
        ]
    );
}

#[test]
fn nested_ordering_empty_boxes_share_one_real_leaf_membership_and_ancestry() {
    nested_leaf::<f32>();
    nested_leaf::<f64>();
}

fn tie_group(boxes: Vec<InlineBoxRef<u32>>) -> InlineGapGroupInput<u32> {
    InlineGapGroupInput {
        anchor: InlineGapAnchor::Neighbor {
            node: 10,
            affinity: InlineGapAffinity::Before,
        },
        boxes,
    }
}
fn native_pair(start: u32, end: u32) -> InlineBoxRef<u32> {
    InlineBoxRef::NativePair { start, end }
}

fn refused<S: LayoutScalar>(
    f: &Fixture<S>,
    wrapping: WrappingInput<u32>,
    expected: WrappingScopeError,
) {
    let error = match LayoutFlowInput::try_new(&f.tree, 0, vec![], wrapping, Vec::new()) {
        Err(error) => error,
        Ok(flow) => compute_layout_with_flow(&flow, 0, request(n(100.0)))
            .expect_err("invalid real attachment authority must not publish"),
    };
    assert!(
        matches!(error.kind(), crate::LayoutErrorKindOf::InvalidInput(
        crate::LayoutInvalidInputOf::WrappingScope(actual)) if *actual == expected),
        "{error:?}"
    );
}

fn unrelated_ties<S: LayoutScalar>() {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    for (start, end) in [(20, 21), (22, 23), (24, 25)] {
        gap_attachments::append_empty_box(&mut f, start, end);
    }
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
            vec![
                child(pair(20, 21)),
                child(pair(22, 23)),
                child(pair(24, 25)),
                child(GeneratedBoxRef::Node(10)),
            ],
        ),
    );
    f.wrapping.gaps = vec![tie_group(vec![
        native_pair(22, 23),
        native_pair(20, 21),
        native_pair(24, 25),
    ])];
    f.finish();
    assert_eq!(f.nodes, [20, 21, 22, 23, 24, 25, 10]);
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = f.flow();
    let prior = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    // Three genuine empty siblings at one physical gap. Explicit order B,A,C
    // differs from source A,B,C; each executes its literal3+5 components once.
    for (node, x, width) in [
        (22, 0.0, 3.0),
        (23, 3.0, 5.0),
        (20, 8.0, 3.0),
        (21, 11.0, 5.0),
        (24, 16.0, 3.0),
        (25, 19.0, 5.0),
    ] {
        boundary_position(&prior, node, x, 0.0, width);
    }
    assert_eq!(output(&prior, 10).location, Point::new(n(24.0), S::ZERO));
    let mut common_root = None;
    for (start, end) in [(20, 21), (22, 23), (24, 25)] {
        let piece = selected_empty(&prior, pair(start, end));
        empty_identity(piece, pair(start, end));
        let root = piece.fragment().parent().unwrap();
        assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
        if let Some(first) = &common_root {
            assert_eq!(&root, first);
        } else {
            common_root = Some(root);
        }
        assert_eq!(piece.selected_source_range(), 0..7);
    }
    // Location is resolved, but separate singleton groups supply no relative
    // physical order for these unrelated siblings. Source order is no substitute.
    let mut missing_tie = f.wrapping.clone();
    missing_tie.gaps = vec![
        tie_group(vec![native_pair(20, 21)]),
        tie_group(vec![native_pair(22, 23)]),
        tie_group(vec![native_pair(24, 25)]),
    ];
    refused(&f, missing_tie, WrappingScopeError::AmbiguousGap);
    // The closed capability does NOT compose partial-order declarations. Even
    // this apparently cyclic AB/BC/CA input fails earlier at real-box uniqueness.
    let mut overlap = f.wrapping.clone();
    overlap.gaps = vec![
        tie_group(vec![native_pair(20, 21), native_pair(22, 23)]),
        tie_group(vec![native_pair(22, 23), native_pair(24, 25)]),
        tie_group(vec![native_pair(24, 25), native_pair(20, 21)]),
    ];
    refused(&f, overlap, WrappingScopeError::ConflictingGap);
    assert_eq!(output(&prior, 10).location, Point::new(n(24.0), S::ZERO));
    assert_eq!(
        selected_empty(&prior, pair(22, 23)).fragment().parent(),
        common_root
    );
}

#[test]
fn tied_empty_siblings_use_explicit_order_and_refuse_missing_or_duplicate_authority() {
    unrelated_ties::<f32>();
    unrelated_ties::<f64>();
}

fn zero_atomics<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    // The two FOREIGN odd operands remain genuine zero-size InlineBlocks with
    // their actual supplied levels and source identities. No hidden/OOF role.
    for node in [14, 15] {
        let mut input = crate::LayoutTree::node_input(&f.tree, node).clone();
        assert_eq!(input.display, crate::Display::InlineBlock);
        assert_eq!(
            input
                .atomic_inline_participation
                .unwrap()
                .bidi_level()
                .get(),
            1
        );
        input.size = Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO));
        f.tree.insert_input(node, LayoutInputOf::box_input(input));
        let item = f
            .participants
            .iter_mut()
            .find_map(|row| match row {
                MixedInlineParticipantOf::Atomic { item, .. }
                    if item.source_index == node as usize =>
                {
                    Some(item)
                }
                _ => None,
            })
            .unwrap();
        item.size = Size::ZERO;
        item.content_size = Size::ZERO;
        item.first_baseline = None; // Genuine empty-block bottom fallback0, no font baseline.
    }
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = f.flow();
    let batch = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert!(batch.placed_text().is_empty());
    for (node, x) in [(10, 0.0), (11, 10.0), (12, 23.0), (13, 33.0)] {
        assert_eq!(output(&batch, node).location, Point::new(n(x), S::ZERO));
        assert_eq!(output(&batch, node).size, Size::splat(n(10.0)));
    }
    for node in [14, 15] {
        assert_eq!(output(&batch, node).size, Size::ZERO);
        assert_eq!(output(&batch, node).location, Point::new(n(33.0), n(10.0)));
    }
    boundary_position(&batch, 20, 20.0, 0.0, 3.0);
    boundary_position(&batch, 21, 43.0, 0.0, 5.0);
    let c = box_piece(&batch, 12);
    let a = box_piece(&batch, 13);
    assert_eq!(member_ids(c), [(12, 3, 2)]);
    assert_eq!(member_ids(a), [(13, 4, 3)]);
    assert_ne!(
        c.fragment(),
        a.fragment(),
        "real foreign operands split C even with zero size"
    );
    let root = c.fragment().parent().unwrap();
    assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(a.fragment().parent(), Some(root.clone()));
    assert_eq!(
        c.fragment().nearest_common_ancestor(&a.fragment()).unwrap(),
        root.clone()
    );
    let roots = batch
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment() == root)
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 1);
    for (node, rank, ordinal) in [(14, 6, 4), (15, 7, 5)] {
        let members = roots[0]
            .selected_members()
            .filter(|member| member.node() == Some(node))
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].child_rank(), Some(rank));
        assert_eq!(members[0].operand_ordinal(), Some(ordinal));
        assert_eq!(
            members[0].empty_box(),
            None,
            "zero-size atomic is not an empty-inline placeholder"
        );
    }
}

#[test]
fn genuine_zero_size_atomics_remain_foreign_participating_component_separators() {
    zero_atomics::<f32>();
    zero_atomics::<f64>();
}
