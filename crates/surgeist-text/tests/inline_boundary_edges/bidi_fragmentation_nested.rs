//! Nested captured owners can split into corresponding parent fragments;
//! wrapping adjacency does not turn the original source parent into their NCA.
use super::*;

fn nested_capture() -> t::PreparedText {
    let p = prepare_capture(
        t::Source::identified("abcABC", t::Id::from_u64(1_646_101), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2), (2, 3), (3, 4), (4, 6)],
        &[
            (K::InlineBoundaryStart, 2),
            (K::InlineBoundaryStart, 2),
            (K::RightToLeftOverride, 3),
            (K::InlineBoundaryEnd, 4),
            (K::InlineBoundaryEnd, 4),
            (
                K::OptionalBreak {
                    wrap: t::Wrap::Word,
                },
                4,
            ),
            (K::PopDirectionalFormat, 6),
        ],
        &[(0, 4), (1, 3)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            E::EnterScope(2),
            control(1),
            origin(1),
            E::Connection(1),
            control(2),
            origin(2),
            control(3),
            E::ExitScope(2),
            control(4),
            E::ExitScope(1),
            control(5),
            E::Connection(2),
            origin(3),
            control(6),
        ],
        AHEM,
    );
    assert_eq!(p.source_segments().len(), 1);
    assert!(
        p.control_views()
            .all(|c| c.capture_position().is_some() && c.handle().captured_control().is_some())
    );
    assert_eq!(
        p.unit_views()
            .map(|u| (u.source_range(), u.level().get()))
            .collect::<Vec<_>>(),
        [
            (t::Range::new(0, 1), 0),
            (t::Range::new(1, 2), 0),
            (t::Range::new(2, 3), 0),
            (t::Range::new(3, 4), 1),
            (t::Range::new(4, 5), 1),
            (t::Range::new(5, 6), 1),
        ]
    );
    p
}

type Contexts = Vec<(t::PreparedControl, t::InlineBoxEdgeContext)>;

fn declarations(
    p: &t::PreparedText,
) -> (
    t::InlineBoxSource,
    t::InlineBoxSource,
    t::InlineBoxSource,
    Contexts,
) {
    let mut outer = p
        .control_views()
        .filter(|v| {
            *v.kind() == K::InlineBoundaryStart
                && v.handle()
                    .captured_control()
                    .unwrap()
                    .scope()
                    .parent()
                    .unwrap()
                    .role()
                    == t::ScopeRole::Domain
        })
        .map(|v| v.handle());
    let ps = outer.next().expect("the genuine outer Inline start");
    assert!(outer.next().is_none());
    let p_scope = ps.captured_control().unwrap().scope();
    let domain = p_scope.parent().unwrap();
    let mut inner = p
        .control_views()
        .filter(|v| {
            *v.kind() == K::InlineBoundaryStart
                && v.handle().captured_control().unwrap().scope().parent() == Some(p_scope.clone())
        })
        .map(|v| v.handle());
    let cs = inner.next().expect("the genuine captured child start");
    assert!(inner.next().is_none());
    let c_scope = cs.captured_control().unwrap().scope();
    assert_eq!(c_scope.parent(), Some(p_scope.clone()));
    assert_ne!(p_scope, c_scope);
    let ending = |scope: &t::CapturedScope| {
        let mut rows = p
            .control_views()
            .filter(|v| {
                *v.kind() == K::InlineBoundaryEnd
                    && v.handle().captured_control().unwrap().scope() == *scope
            })
            .map(|v| v.handle());
        let control = rows.next().unwrap();
        assert!(rows.next().is_none());
        control
    };
    let pe = ending(&p_scope);
    let ce = ending(&c_scope);
    assert_eq!(p_scope.entry_control(), ps.captured_control());
    assert_eq!(p_scope.exit_control(), pe.captured_control());
    assert_eq!(c_scope.entry_control(), cs.captured_control());
    assert_eq!(c_scope.exit_control(), ce.captured_control());
    assert_ne!(ps, cs);
    assert_ne!(pe, ce);
    // Equal source cuts are a control, not evidence of parentage: the actual
    // Enter/Exit/nominations above establish P -> C -> original c/A membership.
    assert_eq!(
        (
            ps.view().source_position().get(),
            cs.view().source_position().get()
        ),
        (2, 2)
    );
    assert_eq!(
        (
            pe.view().source_position().get(),
            ce.view().source_position().get()
        ),
        (4, 4)
    );
    let p_owner = t::InlineBoxSource::captured(p_scope.clone());
    let c_owner = t::InlineBoxSource::captured(c_scope);
    let root = t::InlineBoxSource::captured(domain);
    // Both actual parent used directions remain inherited LTR. RLO changes
    // source text levels, not these explicitly supplied physical parent sides.
    let p_context = p
        .inline_box_edge_context(p_owner.clone(), root.clone(), t::LineEdgeSide::Start)
        .unwrap();
    let c_context = p
        .inline_box_edge_context(c_owner.clone(), p_owner.clone(), t::LineEdgeSide::Start)
        .unwrap();
    (
        p_owner,
        c_owner,
        root,
        vec![
            (ps, p_context.clone()),
            (cs, c_context.clone()),
            (ce, c_context),
            (pe, p_context),
        ],
    )
}

fn nested_line(
    p: &t::PreparedText,
    family: &t::PreparedInlineBoxFamily,
    contexts: &Contexts,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    available: f64,
) -> t::FinalizedLine {
    let zero = t::InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap();
    let boundary = |control: &t::PreparedControl| {
        let context = &contexts
            .iter()
            .find(|(c, _)| c == control)
            .expect("actual nominated endpoint")
            .1;
        t::InlineBoundaryAllocation::box_edge(context.clone(), zero)
    };
    let mut input = request(p, start, end, &[], available);
    input.context.spacing = t::SpacingPolicy::None;
    input.inline_boxes = Some(family.clone());
    for row in &mut input.controls {
        if contexts.iter().any(|(c, _)| c == &row.occurrence) {
            row.boundary = Some(boundary(&row.occurrence));
        } else {
            assert!(!matches!(
                row.occurrence.view().kind(),
                K::InlineBoundaryStart | K::InlineBoundaryEnd
            ));
        }
    }
    while let Some(control) = p.boundary_context_demand(&input).unwrap() {
        let edge = boundary(&control);
        input
            .boundary_context
            .push(t::InlineBoundaryContextAllocation {
                occurrence: control,
                boundary: edge,
            });
    }
    let selected = p.select(input).unwrap();
    assert_eq!(selected.inline_box_family(), Some(family));
    let before = selected
        .inline_box_pieces()
        .unwrap()
        .map(|v| (v.owner(), v.fragment()))
        .collect::<Vec<_>>();
    let output = selected
        .finalize(t::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: t::ScopeAllocations::None,
        })
        .unwrap();
    let after = output
        .inline_box_pieces()
        .unwrap()
        .map(|v| (v.owner(), v.fragment()))
        .collect::<Vec<_>>();
    assert_eq!(before.len(), after.len());
    assert!(before.iter().all(|piece| after.contains(piece)));
    output
}

fn corresponding(
    line: &t::FinalizedLine,
    p_owner: &t::InlineBoxSource,
    c_owner: &t::InlineBoxSource,
    root: &t::InlineBoxSource,
    c: &t::PreparedUnit,
    a: &t::PreparedUnit,
) -> [t::InlineBoxFragment; 4] {
    let p = owned(line, p_owner);
    let child = owned(line, c_owner);
    assert_eq!(
        (p.len(), child.len()),
        (2, 2),
        "both actual owners have two global memberships"
    );
    let pc = carrying(&p, c);
    let pa = carrying(&p, a);
    let cc = carrying(&child, c);
    let ca = carrying(&child, a);
    assert_eq!(texts(&pc), std::slice::from_ref(c));
    assert_eq!(texts(&cc), std::slice::from_ref(c));
    assert_eq!(texts(&pa), std::slice::from_ref(a));
    assert_eq!(texts(&ca), std::slice::from_ref(a));
    // Parent and child share the real executed payload; inclusive membership
    // is not permission to duplicate or slice its provider glyphs.
    assert!(pc.shares_packet_with(&cc));
    assert!(pa.shares_packet_with(&ca));
    let ids = [pc.fragment(), pa.fragment(), cc.fragment(), ca.fragment()];
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[2], ids[3]);
    assert_eq!(ids[2].parent(), Some(ids[0].clone()));
    assert_eq!(ids[3].parent(), Some(ids[1].clone()));
    let domain = ids[0].parent().unwrap();
    assert_eq!(domain.owner(), *root);
    assert!(domain.parent().is_none());
    assert_eq!(ids[1].parent(), Some(domain.clone()));
    assert_eq!(ids[2].nearest_common_ancestor(&ids[3]).unwrap(), domain);
    assert_eq!(ids[3].nearest_common_ancestor(&ids[2]).unwrap(), domain);
    assert_eq!(ids[2].nearest_common_ancestor(&ids[0]).unwrap(), ids[0]);
    assert_eq!(ids[3].nearest_common_ancestor(&ids[1]).unwrap(), ids[1]);
    assert_eq!(ids[2].nearest_common_ancestor(&ids[1]).unwrap(), domain);
    assert_ne!(
        domain.owner(),
        *p_owner,
        "the NCA lies above the original source P"
    );
    ids
}

#[test]
fn nested_split_fragments_have_corresponding_parents_and_a_domain_nca_after_wrapping() {
    let p = nested_capture();
    let (p_owner, c_owner, root, contexts) = declarations(&p);
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    let c = unit(&p, t::Range::new(2, 3));
    let a = unit(&p, t::Range::new(3, 4));
    let wide = nested_line(
        &p,
        &family,
        &contexts,
        p.source_segments().next().unwrap().start(),
        paragraph_end(&p),
        100.0,
    );
    assert_eq!(wide.feedback().advance, 60.0);
    assert_eq!(
        packet(&wide),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 20.0),
            (t::Range::new(5, 6), 37, 30.0),
            (t::Range::new(4, 5), 36, 40.0),
            (t::Range::new(3, 4), 35, 50.0)
        ]
    );
    let ids = corresponding(&wide, &p_owner, &c_owner, &root, &c, &a);
    let cut = actual_optional_break(&p);
    let continuation = cut.view().boundary();
    let first = nested_line(
        &p,
        &family,
        &contexts,
        p.source_segments().next().unwrap().start(),
        cut,
        40.0,
    );
    let second = nested_line(
        &p,
        &family,
        &contexts,
        continuation,
        paragraph_end(&p),
        40.0,
    );
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (40.0, 20.0)
    );
    assert_eq!(
        packet(&first),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 20.0),
            (t::Range::new(3, 4), 35, 30.0)
        ]
    );
    assert_eq!(
        packet(&second),
        [
            (t::Range::new(5, 6), 37, 0.0),
            (t::Range::new(4, 5), 36, 10.0)
        ]
    );
    assert_eq!(
        corresponding(&first, &p_owner, &c_owner, &root, &c, &a),
        ids
    );
    assert!(owned(&second, &p_owner).is_empty() && owned(&second, &c_owner).is_empty());
    let placed = place(&p, &[first, second]);
    assert_eq!(
        placed_packet(&placed),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 20.0, 8.0),
            (35, 30.0, 8.0),
            (37, 0.0, 18.0),
            (36, 10.0, 18.0)
        ]
    );
    for owner in [&p_owner, &c_owner] {
        let views = placed
            .inline_box_pieces()
            .filter(|v| v.finalized_piece().owner() == *owner)
            .collect::<Vec<_>>();
        assert_eq!(views.len(), 2);
        for view in views {
            assert_eq!(view.family(), &family);
            assert_eq!(view.line_index(), 0);
            assert!(ids.contains(&view.fragment()));
        }
    }
}
