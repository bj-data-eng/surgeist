//! Native normal-box identity is independent of physical line wrapping.
use super::*;
use crate::NativeInlineBoxFamilyInput;

#[path = "native_family_functional/cache_completion.rs"]
mod cache_completion;

#[path = "native_family_functional/nested_ties_zero.rs"]
mod nested_ties_zero;

fn intrinsic_closing_cuts<S: LayoutScalar>() {
    let physical = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], true);
    let mut ordinary = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], true);
    contribution_edges(&mut ordinary, 3.0, 5.0);
    let mut transparent = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], true);
    contribution_edges(&mut transparent, 0.0, 0.0);
    transparent
        .wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    for (fixture, minimum, maximum) in [
        // Real closing BP5 belongs with BP3 and the two ten-pixel members.
        (&physical, 28.0, 68.0),
        // Ordinary End remains an operand: the final group is5+10+10.
        (&ordinary, 25.0, 68.0),
        // Proven zero aliases retain custody without adding physical advance.
        (&transparent, 20.0, 60.0),
    ] {
        let input = fixture.numeric();
        let supplied = project_inline_participants(&input.participants);
        let run = ProjectedInlineRun {
            participants: &input.participants,
            supplied: &supplied,
        };
        assert_eq!(
            inline_min_content(run, input.flow_axes, input.wrapping.as_ref()).unwrap(),
            n(minimum)
        );
        assert_eq!(
            inline_max_content(run, input.flow_axes, input.wrapping.as_ref()).unwrap(),
            n(maximum)
        );
    }
}

#[test]
fn native_family_intrinsic_cuts_preserve_closing_roles_f32() {
    intrinsic_closing_cuts::<f32>();
}

#[test]
fn native_family_intrinsic_cuts_preserve_closing_roles_f64() {
    intrinsic_closing_cuts::<f64>();
}

#[path = "native_family_functional/admission.rs"]
mod admission;
#[path = "native_family_functional/composition.rs"]
mod composition;
#[path = "native_family_functional/opposed_empty.rs"]
mod opposed_empty;

fn prepared_fixture<S: LayoutScalar>(levels: [u8; 6], soft_breaks: bool) -> Fixture<S> {
    let mut f = bidi_metadata::split_box_fixture::<S>();
    // The formatter must respond to each request's available width. Its actual
    // generated child inventory and the NativePair proof remain unchanged.
    let mut root = crate::LayoutTree::node_input(&f.tree, 0).clone();
    root.size.width = PreferredSizeOf::AUTO;
    f.tree.insert_input(0, LayoutInputOf::box_input(root));
    for (index, level) in levels.into_iter().enumerate() {
        let node = 10 + index as u32;
        let following = if soft_breaks && matches!(index, 1 | 3) {
            InlineBreakOpportunity::Allowed
        } else {
            InlineBreakOpportunity::Prohibited
        };
        let actual = AtomicInlineParticipation::new(BidiLevel::try_new(level).unwrap(), following);
        let mut input = crate::LayoutTree::node_input(&f.tree, node).clone();
        input.atomic_inline_participation = Some(actual);
        f.tree.insert_input(node, LayoutInputOf::box_input(input));
        let stored = f
            .participants
            .iter_mut()
            .find_map(|row| match row {
                MixedInlineParticipantOf::Atomic {
                    item,
                    participation,
                } if item.source_index == node as usize => Some(participation),
                _ => None,
            })
            .unwrap();
        *stored = actual;
    }
    assert_eq!(f.nodes, [10, 11, 20, 12, 13, 21, 14, 15]);
    f
}

fn contribution_edges<S: LayoutScalar>(f: &mut Fixture<S>, start: f64, end: f64) {
    for (node, kind, width) in [
        (20, InlineBoundaryKind::Start, start),
        (21, InlineBoundaryKind::End, end),
    ] {
        let boundary = InlineBoundaryInputOf::contribution(
            kind,
            metrics(),
            InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
        )
        .unwrap()
        .with_writing_mode(WritingMode::HorizontalTb)
        .with_direction(Direction::Ltr);
        // Zero advance does not erase the actual line metrics or manufacture a
        // physical BoxEdge operation and parent-axis capability.
        assert_eq!(boundary.metrics(), metrics());
        assert_eq!(boundary.parent_edge_axes(), None);
        f.tree
            .insert_input(node, LayoutInputOf::inline_boundary(boundary));
        let stored = f
            .participants
            .iter_mut()
            .find_map(|row| match row {
                MixedInlineParticipantOf::Boundary(control)
                    if control.source_index() == node as usize =>
                {
                    Some(control)
                }
                _ => None,
            })
            .unwrap();
        *stored = InlineBoundaryControlOf::new(
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
}

fn atom_positions<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> [(f64, f64); 6] {
    assert!(batch.placed_text().is_empty());
    std::array::from_fn(|index| {
        let actual = output(batch, 10 + index as u32);
        assert_eq!(actual.size, Size::new(n(10.0), n(10.0)));
        (actual.location.x.to_f64(), actual.location.y.to_f64())
    })
}

fn boundary_position<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    x: f64,
    line_top: f64,
    width: f64,
) {
    let actual = output(batch, node);
    // Baseline controls retain a zero-height reference point at the fixture's
    // ten-pixel baseline; atomic border boxes instead start at the line top.
    assert_eq!(actual.location, Point::new(n(x), n(line_top + 10.0)));
    assert_eq!(actual.size.width, n(width));
    assert_eq!(actual.size.height, S::ZERO);
}

fn box_piece<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> crate::NativeInlineBoxPieceView<'_, u32, S> {
    let pieces: Vec<_> = batch
        .native_inline_box_pieces()
        .filter(|piece| {
            piece.fragment().owner() == pair(20, 21)
                && piece
                    .selected_members()
                    .any(|member| member.node() == Some(node))
        })
        .collect();
    assert_eq!(pieces.len(), 1, "one selected piece owns atomic {node}");
    pieces[0]
}

fn member_ids<S: LayoutScalar>(
    piece: crate::NativeInlineBoxPieceView<'_, u32, S>,
) -> Vec<(u32, usize, usize)> {
    let mut members: Vec<_> = piece
        .selected_members()
        .map(|member| {
            assert_eq!(member.empty_box(), None);
            (
                member.node().unwrap(),
                member.child_rank().unwrap(),
                member.operand_ordinal().unwrap(),
            )
        })
        .collect();
    members.sort_by_key(|member| member.0);
    members
}

fn edges<S: LayoutScalar>(
    piece: crate::NativeInlineBoxPieceView<'_, u32, S>,
) -> Vec<(u32, usize, InlineBoundaryKind, f64, f64)> {
    piece
        .executed_edges()
        .map(|edge| {
            assert_eq!(edge.owner(), pair(20, 21));
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
        .collect()
}

fn demanded_wraps<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], true);
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    // All mutations precede this admission. Both requests use this SAME flow;
    // private numeric() would create a different admitted family.
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
    // Six ten-pixel borders plus the genuine 3/5 strips consume 68, not 60.
    assert_eq!(
        output(&wide, 21).location.x + output(&wide, 21).size.width,
        n(68.0)
    );
    let families: Vec<_> = wide.native_inline_box_families().collect();
    assert_eq!(families.len(), 1);
    let family = &families[0];
    assert_eq!(family.formatter(), 0);
    assert_eq!(
        family
            .fragments()
            .filter(|fragment| fragment.owner() == pair(20, 21))
            .count(),
        2
    );
    let c = box_piece(&wide, 12);
    let a = box_piece(&wide, 13);
    assert_eq!(member_ids(c), [(12, 3, 2)]);
    assert_eq!(member_ids(a), [(13, 4, 3)]);
    let c_fragment = c.fragment();
    let a_fragment = a.fragment();
    assert_ne!(c_fragment, a_fragment);
    let root = c_fragment.parent().unwrap();
    assert_eq!(root.owner(), GeneratedBoxRef::Node(0));
    assert_eq!(root.parent(), None);
    assert_eq!(a_fragment.parent(), Some(root.clone()));
    assert_eq!(c_fragment.body_root(), root);
    assert_eq!(a_fragment.body_root(), root);
    assert_eq!(
        c_fragment.nearest_common_ancestor(&a_fragment).unwrap(),
        root
    );
    assert_eq!(edges(c), [(20, 2, InlineBoundaryKind::Start, 20.0, 23.0)]);
    assert_eq!(edges(a), [(21, 5, InlineBoundaryKind::End, 63.0, 68.0)]);
    for piece in wide.native_inline_box_pieces() {
        assert_eq!(piece.family(), family);
        assert_eq!(piece.line_index(), 0);
        assert_eq!(piece.run_source_range(), 0..8);
        assert_eq!(piece.selected_source_range(), 0..8);
        assert_eq!(
            piece.flow_axes(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
        );
        assert_eq!(piece.source_aliases().count(), 0);
    }

    let narrow = compute_layout_with_flow(&flow, 0, request(n(30.0))).unwrap();
    // Actual permitted cuts follow atoms1 and3. The first/final lines consume
    // 20; the middle consumes 3 + 10 + 10 + 5 = 28, with two adjacent pieces.
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
    assert_eq!(
        output(&narrow, 11).location.x + output(&narrow, 11).size.width,
        n(20.0)
    );
    assert_eq!(
        output(&narrow, 21).location.x + output(&narrow, 21).size.width,
        n(28.0)
    );
    assert_eq!(
        output(&narrow, 14).location.x + output(&narrow, 14).size.width,
        n(20.0)
    );
    assert_eq!(
        narrow.native_inline_box_families().collect::<Vec<_>>(),
        families
    );
    let c_narrow = box_piece(&narrow, 12);
    let a_narrow = box_piece(&narrow, 13);
    assert_eq!(c_narrow.fragment(), c_fragment);
    assert_eq!(a_narrow.fragment(), a_fragment);
    assert_ne!(c_narrow.fragment(), a_narrow.fragment());
    assert_eq!(member_ids(c_narrow), [(12, 3, 2)]);
    assert_eq!(member_ids(a_narrow), [(13, 4, 3)]);
    assert_eq!(c_narrow.line_index(), 1);
    assert_eq!(a_narrow.line_index(), 1);
    assert_eq!(
        edges(c_narrow),
        [(20, 2, InlineBoundaryKind::Start, 0.0, 3.0)]
    );
    assert_eq!(
        edges(a_narrow),
        [(21, 5, InlineBoundaryKind::End, 23.0, 28.0)]
    );
    let sources = [0..2, 2..6, 6..8];
    let mut root_lines = Vec::new();
    for piece in narrow.native_inline_box_pieces() {
        assert_eq!(piece.family(), family);
        assert_eq!(piece.run_source_range(), 0..8);
        assert_eq!(piece.selected_source_range(), sources[piece.line_index()]);
        assert_eq!(
            piece.flow_axes(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
        );
        assert_eq!(piece.source_aliases().count(), 0);
        if piece.fragment().owner() == GeneratedBoxRef::Node(0) {
            assert_eq!(piece.fragment(), root);
            root_lines.push(piece.line_index());
        }
    }
    root_lines.sort_unstable();
    assert_eq!(root_lines, [0, 1, 2]);
    // The earlier immutable wide result retains its executed packet and IDs.
    assert_eq!(atom_positions(&wide)[3], (53.0, 0.0));
    assert_eq!(box_piece(&wide, 12).fragment(), c_fragment);
}

#[test]
fn requested_native_family_keeps_two_global_fragments_across_soft_wraps_f32() {
    demanded_wraps::<f32>();
}
#[test]
fn requested_native_family_keeps_two_global_fragments_across_soft_wraps_f64() {
    demanded_wraps::<f64>();
}

fn mixed_level_contiguous<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 0, 0], false);
    f.wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = f.flow();
    let batch = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    // Reversing one odd-level operand cannot separate its adjacent owner peer.
    assert_eq!(
        atom_positions(&batch),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (23.0, 0.0),
            (33.0, 0.0),
            (48.0, 0.0),
            (58.0, 0.0)
        ]
    );
    boundary_position(&batch, 20, 20.0, 0.0, 3.0);
    boundary_position(&batch, 21, 43.0, 0.0, 5.0);
    let families: Vec<_> = batch.native_inline_box_families().collect();
    assert_eq!(families.len(), 1);
    assert_eq!(
        families[0]
            .fragments()
            .filter(|fragment| fragment.owner() == pair(20, 21))
            .count(),
        1
    );
    let c = box_piece(&batch, 12);
    let a = box_piece(&batch, 13);
    assert_eq!(c.fragment(), a.fragment());
    assert_eq!(member_ids(c), [(12, 3, 2), (13, 4, 3)]);
    assert_eq!(
        c.fragment().parent().unwrap().owner(),
        GeneratedBoxRef::Node(0)
    );
}

#[test]
fn mixed_levels_with_contiguous_members_have_one_native_fragment_f32() {
    mixed_level_contiguous::<f32>();
}
#[test]
fn mixed_levels_with_contiguous_members_have_one_native_fragment_f64() {
    mixed_level_contiguous::<f64>();
}

fn zero_contribution_aliases<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    contribution_edges(&mut f, 0.0, 0.0);
    let ordinary = f.flow();
    let prior = compute_layout_with_flow(&ordinary, 0, request(n(100.0))).unwrap();
    // Ordinary contributions remain supplied operands: End separates the odd
    // atomic3 from the final odd pair. Merely being zero is not transparency.
    assert_eq!(
        atom_positions(&prior),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (30.0, 0.0),
            (50.0, 0.0),
            (40.0, 0.0)
        ]
    );
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_pieces().count(), 0);

    let mut demanded = f.wrapping.clone();
    demanded
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let flow = LayoutFlowInput::try_new(&f.tree, 0, vec![], demanded, Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert_eq!(
        atom_positions(&batch),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (50.0, 0.0),
            (40.0, 0.0),
            (30.0, 0.0)
        ]
    );
    boundary_position(&batch, 20, 20.0, 0.0, 0.0);
    boundary_position(&batch, 21, 50.0, 0.0, 0.0);
    let families: Vec<_> = batch.native_inline_box_families().collect();
    assert_eq!(families.len(), 1);
    assert_eq!(
        families[0]
            .fragments()
            .filter(|fragment| fragment.owner() == pair(20, 21))
            .count(),
        2
    );
    let c = box_piece(&batch, 12);
    let a = box_piece(&batch, 13);
    assert_ne!(c.fragment(), a.fragment());
    assert_eq!(member_ids(c), [(12, 3, 2)]);
    assert_eq!(member_ids(a), [(13, 4, 3)]);
    for piece in batch.native_inline_box_pieces() {
        assert_eq!(piece.executed_edges().count(), 0);
    }
    let mut aliases = Vec::new();
    for piece in [c, a] {
        for edge in piece.source_aliases() {
            assert_eq!(edge.owner(), pair(20, 21));
            assert!(edge.is_source_alias());
            assert_eq!(edge.flow_interval(), None);
            assert_eq!(edge.strip_interval(), None);
            assert_eq!(
                edge.flow_axes(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
            );
            aliases.push((
                edge.node(),
                edge.child_rank(),
                edge.kind(),
                edge.source_alias_position().unwrap().to_f64(),
            ));
        }
    }
    aliases.sort_by_key(|alias| alias.0);
    assert_eq!(
        aliases,
        [
            (20, 2, InlineBoundaryKind::Start, 20.0),
            (21, 5, InlineBoundaryKind::End, 50.0)
        ]
    );
    // The source End alias is the logical edge of the reversed atomic at50,
    // not the physical right edge of the owner's last ten-pixel border at60.
    assert_eq!(
        output(&batch, 13).location.x + output(&batch, 13).size.width,
        n(60.0)
    );
    assert_eq!(atom_positions(&prior)[3], (30.0, 0.0));
}

#[test]
fn requested_zero_contributions_are_source_aliases_not_physical_edges_f32() {
    zero_contribution_aliases::<f32>();
}
#[test]
fn requested_zero_contributions_are_source_aliases_not_physical_edges_f64() {
    zero_contribution_aliases::<f64>();
}

fn nonzero_contribution_refusal<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    contribution_edges(&mut f, 3.0, 5.0);
    let ordinary = f.flow();
    let prior = compute_layout_with_flow(&ordinary, 0, request(n(100.0))).unwrap();
    assert_eq!(
        atom_positions(&prior),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (23.0, 0.0),
            (33.0, 0.0),
            (58.0, 0.0),
            (48.0, 0.0)
        ]
    );
    boundary_position(&prior, 20, 20.0, 0.0, 3.0);
    boundary_position(&prior, 21, 43.0, 0.0, 5.0);
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_pieces().count(), 0);
    let mut demanded = f.wrapping.clone();
    demanded
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let error = match LayoutFlowInput::try_new(&f.tree, 0, vec![], demanded, Vec::new()) {
        Ok(_) => panic!("a nonzero Contribution cannot be promoted into a normal-box family"),
        Err(error) => error,
    };
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), crate::LayoutOperation::InlineWrapping);
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::IncompatibleContribution
            ))
        ),
        "unexpected refusal: {error:?}"
    );
    // Failed demand neither changes the actual inputs nor poisons prior output
    // or the still-valid ordinary operation.
    assert_eq!(atom_positions(&prior)[3], (33.0, 0.0));
    let retry = compute_layout_with_flow(&ordinary, 0, request(n(100.0))).unwrap();
    assert_eq!(atom_positions(&retry), atom_positions(&prior));
}

#[test]
fn nonzero_contributions_refuse_family_demand_without_promotion_f32() {
    nonzero_contribution_refusal::<f32>();
}
#[test]
fn nonzero_contributions_refuse_family_demand_without_promotion_f64() {
    nonzero_contribution_refusal::<f64>();
}
