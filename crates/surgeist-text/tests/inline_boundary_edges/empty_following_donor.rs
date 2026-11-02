//! A genuine empty sibling can interpose between mixed-level box members.
use super::*;

struct DonorFixture {
    prepared: t::PreparedText,
    content: t::CapturedScope,
    empty: Option<t::CapturedScope>,
}

fn donor_fixture(with_empty: bool) -> DonorFixture {
    let mut controls = vec![(K::RightToLeftOverride, 0)];
    let mut pairs = Vec::new();
    let mut body = vec![control(0)];
    let empty_start = if with_empty {
        controls.extend([(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 0)]);
        pairs.push((1, 2));
        body.extend([E::EnterScope(1), control(1), control(2), E::ExitScope(1)]);
        Some(1)
    } else {
        None
    };
    let content_scope = pairs.len() + 1;
    let opening = controls.len();
    controls.extend([
        (K::InlineBoundaryStart, 0),
        (K::PopDirectionalFormat, 1),
        (K::RightToLeftOverride, 2),
        (K::InlineBoundaryEnd, 3),
        (K::PopDirectionalFormat, 3),
    ]);
    pairs.push((opening, opening + 3));
    body.extend([
        E::EnterScope(content_scope),
        control(opening),
        origin(0),
        E::Connection(0),
        control(opening + 1),
        origin(1),
        E::Connection(1),
        control(opening + 2),
        origin(2),
        control(opening + 3),
        E::ExitScope(content_scope),
        control(opening + 4),
    ]);
    let p = prepare_capture(
        t::Source::identified("aAb", t::Id::from_u64(1_646_402), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1), (1, 2), (2, 3)],
        &controls,
        &pairs,
        body,
        AHEM,
    );
    let issued = p
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let content = issued[opening].captured_control().unwrap().scope();
    assert_eq!(content.role(), t::ScopeRole::Inline);
    assert_eq!(content.entry_control(), issued[opening].captured_control());
    assert_eq!(
        content.exit_control(),
        issued[opening + 3].captured_control()
    );
    let domain = content.parent().unwrap();
    let outer_rlo = issued[0].captured_control().unwrap();
    assert_eq!(outer_rlo.kind(), &K::RightToLeftOverride);
    assert_eq!(outer_rlo.scope(), domain);
    assert!(outer_rlo.event_position() < content.enter_position());
    let empty = empty_start.map(|start| {
        let left = issued[start].captured_control().unwrap();
        let right = issued[start + 1].captured_control().unwrap();
        let scope = left.scope();
        assert_eq!(scope.role(), t::ScopeRole::Inline);
        assert_eq!(scope.parent(), Some(domain.clone()));
        assert_eq!(scope.entry_control(), Some(left.clone()));
        assert_eq!(scope.exit_control(), Some(right.clone()));
        assert_eq!(right.scope(), scope);
        assert_ne!(scope, content);
        assert_eq!(left.source_position().get(), 0);
        assert_eq!(right.source_position().get(), 0);
        assert!(outer_rlo.event_position() < scope.enter_position());
        assert!(scope.enter_position() < left.event_position());
        assert!(left.event_position() < right.event_position());
        assert!(right.event_position() < scope.exit_position());
        assert!(scope.exit_position() < content.enter_position());
        scope
    });
    let actual_origins = p.origin_views().collect::<Vec<_>>();
    assert_eq!(
        actual_origins
            .iter()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [
            t::Range::new(0, 1),
            t::Range::new(1, 2),
            t::Range::new(2, 3)
        ]
    );
    assert!(content.enter_position() < actual_origins[0].capture_position().unwrap());
    assert!(
        actual_origins[0].capture_position().unwrap()
            < issued[opening + 1]
                .captured_control()
                .unwrap()
                .event_position()
    );
    assert!(
        issued[opening + 1]
            .captured_control()
            .unwrap()
            .event_position()
            < actual_origins[1].capture_position().unwrap()
    );
    assert!(
        actual_origins[1].capture_position().unwrap()
            < issued[opening + 2]
                .captured_control()
                .unwrap()
                .event_position()
    );
    assert!(
        issued[opening + 2]
            .captured_control()
            .unwrap()
            .event_position()
            < actual_origins[2].capture_position().unwrap()
    );
    assert!(
        content.exit_position()
            < issued[opening + 4]
                .captured_control()
                .unwrap()
                .event_position()
    );
    DonorFixture {
        prepared: p,
        content,
        empty,
    }
}

fn donor_local_line(p: &t::PreparedText) -> t::FinalizedLine {
    let segment = p.source_segments().next().unwrap();
    assert_eq!(p.source_segments().len(), 1);
    let mut input = request(p, segment.start(), paragraph_end(p), &[], 100.0);
    input.context.spacing = t::SpacingPolicy::None;
    for row in &mut input.controls {
        match row.occurrence.view().kind() {
            K::InlineBoundaryStart | K::InlineBoundaryEnd => {
                let owner = row.occurrence.captured_control().unwrap().scope();
                let context = p
                    .inline_box_edge_context(
                        t::InlineBoxSource::captured(owner.clone()),
                        t::InlineBoxSource::captured(owner.parent().unwrap()),
                        t::LineEdgeSide::Start,
                    )
                    .unwrap();
                row.boundary = Some(t::InlineBoundaryAllocation::box_edge(
                    context,
                    t::InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap(),
                ));
            }
            _ => assert!(row.boundary.is_none()),
        }
    }
    assert!(p.boundary_context_demand(&input).unwrap().is_none());
    finalize(p, input)
}

fn nonempty_text_members(piece: &t::FinalizedInlineBoxPiece) -> Vec<t::PreparedUnit> {
    piece
        .members()
        .filter_map(|member| match member.participant() {
            Some(t::PreparedParticipant::Text(unit)) if !unit.view().source_range().is_empty() => {
                assert_eq!(member.unit_source_range(), unit.view().source_range());
                assert_eq!(member.presentation_range().len(), 1);
                Some(unit.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn empty_inline_uses_following_level_and_splits_only_interposed_global_members() {
    // No-empty control first: level changes alone are not foreign content.
    for with_empty in [false, true] {
        let f = donor_fixture(with_empty);
        let p = &f.prepared;
        assert_eq!(p.analyzed_source().source().text(), &t::Text::from("aAb"));
        assert!(p.analyzed_source().combined_cells().next().is_none());
        let a = unit(p, t::Range::new(0, 1));
        let upper_a = unit(p, t::Range::new(1, 2));
        let b = unit(p, t::Range::new(2, 3));
        // Real X4/PDF context: a1, A0, b1. Formatting controls remain actual
        // Control participants; no placeholder/ORC or fabricated scalar donor.
        assert_eq!(
            [
                a.view().level().get(),
                upper_a.view().level().get(),
                b.view().level().get()
            ],
            [1, 0, 1]
        );
        assert_eq!(
            p.participants()
                .filter_map(|part| match part {
                    t::PreparedParticipant::Text(unit)
                        if !unit.view().source_range().is_empty() =>
                        Some(unit),
                    t::PreparedParticipant::Atomic(_) | t::PreparedParticipant::Cell(_) =>
                        panic!("no invented atomic or Cell"),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [a.clone(), upper_a.clone(), b.clone()]
        );
        let local = donor_local_line(p);
        let expected = [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 35, 10.0),
            (t::Range::new(2, 3), 68, 20.0),
        ];
        assert_eq!(packet(&local), expected);
        assert_eq!(local.feedback().advance, 30.0);
        let family = p.inline_box_family(vec![], None).unwrap();
        let owner = t::InlineBoxSource::captured(f.content.clone());
        let output = reselect(p, &family, &owner, &local);
        assert_eq!(packet(&output), expected);
        assert_eq!(output.feedback().advance, 30.0);
        let content = owned(&output, &owner);
        if let Some(empty) = f.empty {
            assert_eq!(content.len(), 2);
            let lefts = content
                .iter()
                .filter(|piece| nonempty_text_members(piece).contains(&a))
                .collect::<Vec<_>>();
            let rights = content
                .iter()
                .filter(|piece| nonempty_text_members(piece).contains(&upper_a))
                .collect::<Vec<_>>();
            assert_eq!(lefts.len(), 1);
            assert_eq!(rights.len(), 1);
            let left = lefts[0];
            let right = rights[0];
            assert_eq!(nonempty_text_members(left), std::slice::from_ref(&a));
            assert_eq!(nonempty_text_members(right), [upper_a.clone(), b.clone()]);
            assert_ne!(left.fragment(), right.fragment());
            let root = t::InlineBoxSource::captured(f.content.parent().unwrap());
            let nca = left
                .fragment()
                .nearest_common_ancestor(&right.fragment())
                .unwrap();
            assert_eq!(nca.owner(), root);
            assert_eq!(left.fragment().parent(), Some(nca.clone()));
            assert_eq!(right.fragment().parent(), Some(nca.clone()));
            let empty_owner = t::InlineBoxSource::captured(empty);
            let empty_pieces = owned(&output, &empty_owner);
            assert_eq!(empty_pieces.len(), 1);
            assert_eq!(empty_pieces[0].fragment().parent(), Some(nca));
            assert!(nonempty_text_members(&empty_pieces[0]).is_empty());
        } else {
            assert_eq!(content.len(), 1);
            assert_eq!(nonempty_text_members(&content[0]), [a, upper_a, b]);
        }
        assert_eq!(
            packet(&local),
            expected,
            "prior local publication remains intact"
        );
    }
}
