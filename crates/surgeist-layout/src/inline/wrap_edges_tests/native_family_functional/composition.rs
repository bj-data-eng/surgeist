//! Native family bodies and ancestry follow real source roles, not fitting runs.
use super::*;

#[derive(Clone, Copy)]
enum Interruption {
    Float,
    VisibleBreak,
    Block,
}

fn interrupted_fixture<S: LayoutScalar>(kind: Interruption) -> Fixture<S> {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    // The actual source order is10,11,Start20,12,interruption30,13,End21,14,15.
    // A visible break has no generated box; the two actual box roles do.
    let input = match kind {
        Interruption::VisibleBreak => LayoutInputOf::line_break(
            crate::LineBreakInputOf::new()
                .with_metrics(metrics())
                .with_direction(Direction::Ltr),
        ),
        Interruption::Float => LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            float: crate::Float::Left,
            size: Size::new(PreferredSizeOf::px(n(8.0)), PreferredSizeOf::px(n(4.0))),
            ..NodeInputOf::default()
        }),
        Interruption::Block => LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(7.0))),
            ..NodeInputOf::default()
        }),
    };
    f.tree.insert_input(30, input);
    f.nodes.insert(4, 30);
    f.finish();
    let mut inside = vec![child(GeneratedBoxRef::Node(12))];
    match kind {
        Interruption::VisibleBreak => {}
        Interruption::Float => inside.push(GeneratedChildBoxInput {
            box_: GeneratedBoxRef::Node(30),
            participation: GeneratedBoxParticipation::OutOfFlow,
        }),
        Interruption::Block => inside.push(child(GeneratedBoxRef::Node(30))),
    }
    inside.push(child(GeneratedBoxRef::Node(13)));
    f.wrapping.child_order = vec![
        order(
            GeneratedBoxRef::Node(0),
            vec![
                child(GeneratedBoxRef::Node(10)),
                child(GeneratedBoxRef::Node(11)),
                child(pair(20, 21)),
                child(GeneratedBoxRef::Node(14)),
                child(GeneratedBoxRef::Node(15)),
            ],
        ),
        order(pair(20, 21), inside),
    ];
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    assert_eq!(f.nodes, [10, 11, 20, 12, 30, 13, 21, 14, 15]);
    f
}

fn one_family<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> crate::NativeInlineBoxFamily<u32> {
    let families: Vec<_> = batch.native_inline_box_families().collect();
    assert_eq!(families.len(), 1);
    assert_eq!(families[0].formatter(), 0);
    assert!(batch.placed_text().is_empty());
    families.into_iter().next().unwrap()
}

fn root_members<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<(u32, usize, usize)> {
    let mut members = Vec::new();
    for piece in batch
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == GeneratedBoxRef::Node(0))
    {
        members.extend(member_ids(piece));
    }
    members.sort_by_key(|member| member.0);
    members
}

fn float_split_runs<S: LayoutScalar>() {
    let f = interrupted_fixture::<S>(Interruption::Float);
    let actual = crate::LayoutTree::node_input(&f.tree, 30);
    assert_eq!(actual.float, crate::Float::Left);
    assert_eq!(actual.display, crate::Display::Block);
    assert_eq!(actual.atomic_inline_participation, None);
    let flow = f.flow();
    let batch = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    // The first real run consumes33: two borders20 + Start3 + border10.
    // The subsequent8x4 float begins at block10. Its exclusion shifts the
    // second ten-pixel line right8; the three odd operands reverse there.
    assert_eq!(
        atom_positions(&batch),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (23.0, 0.0),
            (28.0, 10.0),
            (18.0, 10.0),
            (8.0, 10.0)
        ]
    );
    let float = output(&batch, 30);
    assert_eq!(float.location, Point::new(S::ZERO, n(10.0)));
    assert_eq!(float.size, Size::new(n(8.0), n(4.0)));
    assert_eq!(float.source_index.get(), 4);
    boundary_position(&batch, 20, 20.0, 0.0, 3.0);
    boundary_position(&batch, 21, 38.0, 10.0, 5.0);
    assert_eq!(output(&batch, 0).size.height, n(20.0));
    let family = one_family(&batch);
    assert_eq!(
        family
            .fragments()
            .filter(|fragment| fragment.owner() == pair(20, 21))
            .count(),
        2
    );
    assert_eq!(
        family
            .fragments()
            .filter(|fragment| fragment.owner() == GeneratedBoxRef::Node(30))
            .count(),
        0
    );
    let c = box_piece(&batch, 12);
    let a = box_piece(&batch, 13);
    assert_eq!(member_ids(c), [(12, 3, 2)]);
    assert_eq!(member_ids(a), [(13, 5, 3)]);
    assert_eq!(
        root_members(&batch),
        [
            (10, 0, 0),
            (11, 1, 1),
            (12, 3, 2),
            (13, 5, 3),
            (14, 7, 4),
            (15, 8, 5)
        ]
    );
    assert_ne!(c.fragment(), a.fragment());
    let root = c.fragment().body_root();
    assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(root.parent(), None);
    assert_eq!(a.fragment().body_root(), root);
    assert_eq!(c.fragment().parent(), Some(root.clone()));
    assert_eq!(a.fragment().parent(), Some(root.clone()));
    assert_eq!(
        c.fragment().nearest_common_ancestor(&a.fragment()).unwrap(),
        root
    );
    assert_eq!(c.family(), &family);
    assert_eq!(a.family(), &family);
    // Both are local line0, but belong to DIFFERENT real fitting-run windows.
    // The float at raw rank4 is not borrowed into either selected window.
    assert_eq!((c.line_index(), a.line_index()), (0, 0));
    assert_eq!(c.run_source_range(), 0..4);
    assert_eq!(c.selected_source_range(), 0..4);
    assert_eq!(a.run_source_range(), 5..9);
    assert_eq!(a.selected_source_range(), 5..9);
    assert_eq!(edges(c), [(20, 2, InlineBoundaryKind::Start, 20.0, 23.0)]);
    assert_eq!(edges(a), [(21, 6, InlineBoundaryKind::End, 38.0, 43.0)]);
    let mut root_runs = Vec::new();
    for piece in batch.native_inline_box_pieces() {
        assert_eq!(piece.family(), &family);
        assert_eq!(piece.source_aliases().count(), 0);
        assert_eq!(
            piece.flow_axes(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
        );
        assert!(
            piece
                .selected_members()
                .all(|member| member.node() != Some(30))
        );
        if piece.fragment().owner() == GeneratedBoxRef::Node(0) {
            assert_eq!(piece.fragment(), root);
            root_runs.push(piece.run_source_range());
        }
    }
    root_runs.sort_by_key(|run| run.start);
    assert_eq!(root_runs, [0..4, 5..9]);
}

#[test]
fn real_float_splits_fitting_runs_without_splitting_native_body_f32() {
    float_split_runs::<f32>();
}
#[test]
fn real_float_splits_fitting_runs_without_splitting_native_body_f64() {
    float_split_runs::<f64>();
}

fn distinct_bodies<S: LayoutScalar>(kind: Interruption) {
    let f = interrupted_fixture::<S>(kind);
    let flow = f.flow();
    let batch = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    let second_top = match kind {
        Interruption::VisibleBreak => 10.0,
        Interruption::Block => 17.0,
        Interruption::Float => unreachable!("a float is not a body delimiter"),
    };
    // Before the actual delimiter, the three zero-level atomics stay ordered.
    // The next body has three odd atomics, whose real selected order is5,4,3.
    assert_eq!(
        atom_positions(&batch),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (23.0, 0.0),
            (20.0, second_top),
            (10.0, second_top),
            (0.0, second_top)
        ]
    );
    boundary_position(&batch, 20, 20.0, 0.0, 3.0);
    boundary_position(&batch, 21, 30.0, second_top, 5.0);
    assert_eq!(output(&batch, 0).size.height, n(second_top + 10.0));
    let separator = output(&batch, 30);
    assert_eq!(separator.source_index.get(), 4);
    match kind {
        Interruption::VisibleBreak => {
            // A genuine forced control has a baseline point and no border box.
            assert_eq!(separator.location, Point::new(n(33.0), n(10.0)));
            assert_eq!(separator.size, Size::ZERO);
        }
        Interruption::Block => {
            assert_eq!(separator.location, Point::new(S::ZERO, n(10.0)));
            assert_eq!(separator.size, Size::new(n(10.0), n(7.0)));
        }
        Interruption::Float => unreachable!(),
    }
    let family = one_family(&batch);
    assert_eq!(
        family
            .fragments()
            .filter(|fragment| fragment.owner() == pair(20, 21))
            .count(),
        2
    );
    let c = box_piece(&batch, 12);
    let a = box_piece(&batch, 13);
    assert_eq!(member_ids(c), [(12, 3, 2)]);
    assert_eq!(member_ids(a), [(13, 5, 3)]);
    assert_eq!(
        root_members(&batch),
        [
            (10, 0, 0),
            (11, 1, 1),
            (12, 3, 2),
            (13, 5, 3),
            (14, 7, 4),
            (15, 8, 5)
        ]
    );
    assert_eq!(c.family(), &family);
    assert_eq!(a.family(), &family);
    assert_ne!(c.fragment(), a.fragment());
    let first = c.fragment().body_root();
    let second = a.fragment().body_root();
    assert_eq!(first.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(second.owner(), GeneratedBoxRef::Node(0));
    assert_ne!(first, second);
    assert_eq!(first.parent(), None);
    assert_eq!(second.parent(), None);
    assert_eq!(c.fragment().parent(), Some(first.clone()));
    assert_eq!(a.fragment().parent(), Some(second.clone()));
    assert_eq!(
        c.fragment().nearest_common_ancestor(&a.fragment()),
        Err(WrappingScopeError::DifferentBody)
    );
    assert_eq!(
        first.nearest_common_ancestor(&second),
        Err(WrappingScopeError::DifferentBody)
    );
    assert_eq!(edges(c), [(20, 2, InlineBoundaryKind::Start, 20.0, 23.0)]);
    assert_eq!(edges(a), [(21, 6, InlineBoundaryKind::End, 30.0, 35.0)]);
    match kind {
        Interruption::VisibleBreak => {
            // One real fitting run contains both actual forced lines. The break
            // at raw rank4 is consumed separately; projected line0 units end
            // before it. The delimiter is not a supplied L2 operand.
            assert_eq!((c.line_index(), a.line_index()), (0, 1));
            assert_eq!(c.run_source_range(), 0..9);
            assert_eq!(a.run_source_range(), 0..9);
            assert_eq!(c.selected_source_range(), 0..4);
            assert_eq!(a.selected_source_range(), 5..9);
        }
        Interruption::Block => {
            // The actual block owns raw rank4 and7px of block flow, separating
            // the fitting runs as well as delimiting the global bodies.
            assert_eq!((c.line_index(), a.line_index()), (0, 0));
            assert_eq!(c.run_source_range(), 0..4);
            assert_eq!(c.selected_source_range(), 0..4);
            assert_eq!(a.run_source_range(), 5..9);
            assert_eq!(a.selected_source_range(), 5..9);
        }
        Interruption::Float => unreachable!(),
    }
}

#[test]
fn genuine_visible_break_delimits_distinct_native_body_roots_f32() {
    distinct_bodies::<f32>(Interruption::VisibleBreak);
}
#[test]
fn genuine_visible_break_delimits_distinct_native_body_roots_f64() {
    distinct_bodies::<f64>(Interruption::VisibleBreak);
}
#[test]
fn in_flow_block_delimits_distinct_native_body_roots_f32() {
    distinct_bodies::<f32>(Interruption::Block);
}
#[test]
fn in_flow_block_delimits_distinct_native_body_roots_f64() {
    distinct_bodies::<f64>(Interruption::Block);
}

fn nested_auto_ancestors<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], true);
    // P is a genuine undecorated Auto NativePair. Its exact zero Contribution
    // occurrences are not promoted to BoxEdges; C keeps its real3/5 operation.
    f.boundary(30, InlineBoundaryKind::Start, 0.0);
    f.boundary(31, InlineBoundaryKind::End, 0.0);
    f.nodes = vec![10, 11, 30, 20, 12, 13, 21, 31, 14, 15];
    f.finish();
    f.wrapping.scopes = vec![
        WrappingScopeInput::NativePair {
            start: 30,
            end: 31,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        },
        WrappingScopeInput::NativePair {
            start: 20,
            end: 21,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        },
    ];
    f.wrapping.child_order = vec![
        order(
            GeneratedBoxRef::Node(0),
            vec![
                child(GeneratedBoxRef::Node(10)),
                child(GeneratedBoxRef::Node(11)),
                child(pair(30, 31)),
                child(GeneratedBoxRef::Node(14)),
                child(GeneratedBoxRef::Node(15)),
            ],
        ),
        order(pair(30, 31), vec![child(pair(20, 21))]),
        order(
            pair(20, 21),
            vec![
                child(GeneratedBoxRef::Node(12)),
                child(GeneratedBoxRef::Node(13)),
            ],
        ),
    ];
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = f.flow();
    let wide = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert_eq!(
        atom_positions(&wide),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (23.0, 0.0),
            (53.0, 0.0),
            (43.0, 0.0),
            (33.0, 0.0)
        ]
    );
    boundary_position(&wide, 20, 20.0, 0.0, 3.0);
    boundary_position(&wide, 21, 63.0, 0.0, 5.0);
    let family = one_family(&wide);
    for owner in [pair(30, 31), pair(20, 21)] {
        assert_eq!(
            family
                .fragments()
                .filter(|fragment| fragment.owner() == owner)
                .count(),
            2
        );
    }
    let c = box_piece(&wide, 12);
    let a = box_piece(&wide, 13);
    assert_eq!(member_ids(c), [(12, 4, 2)]);
    assert_eq!(member_ids(a), [(13, 5, 3)]);
    let c_fragment = c.fragment();
    let a_fragment = a.fragment();
    let p_c = c_fragment.parent().unwrap();
    let p_a = a_fragment.parent().unwrap();
    assert_eq!(p_c.owner(), pair(30, 31));
    assert_eq!(p_a.owner(), pair(30, 31));
    assert_ne!(p_c, p_a);
    let root = p_c.parent().unwrap();
    assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(root.parent(), None);
    assert_eq!(p_a.parent(), Some(root.clone()));
    assert_eq!(c_fragment.nearest_common_ancestor(&p_c).unwrap(), p_c);
    assert_eq!(a_fragment.nearest_common_ancestor(&p_a).unwrap(), p_a);
    assert_eq!(
        c_fragment.nearest_common_ancestor(&a_fragment).unwrap(),
        root
    );
    assert_eq!(p_c.nearest_common_ancestor(&p_a).unwrap(), root);
    assert_eq!(c_fragment.body_root(), root);
    assert_eq!(a_fragment.body_root(), root);
    let mut p_aliases = Vec::new();
    for piece in wide
        .native_inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == pair(30, 31))
    {
        // P owns source aliases, never a fabricated strip operation. A real
        // descendant's operation, if exposed, must retain that descendant owner.
        assert!(
            piece
                .executed_edges()
                .all(|edge| edge.owner() != pair(30, 31))
        );
        for alias in piece
            .source_aliases()
            .filter(|alias| alias.owner() == pair(30, 31))
        {
            assert!(alias.is_source_alias());
            assert_eq!(alias.flow_interval(), None);
            assert_eq!(alias.strip_interval(), None);
            p_aliases.push((alias.node(), alias.child_rank(), alias.kind()));
        }
    }
    p_aliases.sort_by_key(|alias| alias.0);
    assert_eq!(
        p_aliases,
        [
            (30, 2, InlineBoundaryKind::Start),
            (31, 7, InlineBoundaryKind::End)
        ]
    );

    let narrow = compute_layout_with_flow(&flow, 0, request(n(30.0))).unwrap();
    assert_eq!(
        atom_positions(&narrow),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (3.0, 10.0),
            (13.0, 10.0),
            (10.0, 20.0),
            (0.0, 20.0)
        ]
    );
    boundary_position(&narrow, 20, 0.0, 10.0, 3.0);
    boundary_position(&narrow, 21, 23.0, 10.0, 5.0);
    assert_eq!(one_family(&narrow), family);
    let c_narrow = box_piece(&narrow, 12);
    let a_narrow = box_piece(&narrow, 13);
    assert_eq!(c_narrow.fragment(), c_fragment);
    assert_eq!(a_narrow.fragment(), a_fragment);
    assert_eq!(c_narrow.fragment().parent(), Some(p_c));
    assert_eq!(a_narrow.fragment().parent(), Some(p_a));
    assert_eq!((c_narrow.line_index(), a_narrow.line_index()), (1, 1));
    assert_eq!(c_narrow.selected_source_range(), 2..8);
    assert_eq!(a_narrow.selected_source_range(), 2..8);
    assert_eq!(c_narrow.run_source_range(), 0..10);
    assert_eq!(a_narrow.run_source_range(), 0..10);
    assert_eq!(
        c_narrow
            .fragment()
            .nearest_common_ancestor(&a_narrow.fragment())
            .unwrap(),
        root
    );
}

#[test]
fn undecorated_auto_ancestors_keep_real_split_parent_ncas_across_wrapping_f32() {
    nested_auto_ancestors::<f32>();
}
#[test]
fn undecorated_auto_ancestors_keep_real_split_parent_ncas_across_wrapping_f64() {
    nested_auto_ancestors::<f64>();
}
