//! A genuine deleted-only box retains source aliases independently of its family.
use super::*;

#[test]
fn positive_source_deleted_only_inline_retains_family_and_original_aliases() {
    // Same actual producer/recipe as scope_incidence's removed-and-empty case:
    // Discard truly eliminates the positive SPACE, unlike collapsed-zero SPACE.
    let p = prepare_capture(
        t::Source::identified(" A", t::Id::from_u64(1_646_403), 1),
        block(style(t::WhiteSpace::Discard), false, true),
        &[(0, 1), (1, 2)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryEnd, 1),
            (
                K::OptionalBreak {
                    wrap: t::Wrap::Word,
                },
                1,
            ),
        ],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            control(2),
            E::Connection(0),
            origin(1),
        ],
        AHEM,
    );
    assert_eq!(p.analyzed_source().source().text(), &t::Text::from(" A"));
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::Text::from("A")
    );
    let originals = p
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 2);
    assert_eq!(originals[0].view().aggregate_range(), t::Range::new(0, 1));
    assert_eq!(originals[0].view().local_range(), t::Range::new(0, 1));
    assert_eq!(originals[0].view().source().text(), &t::Text::from(" A"));
    let source_alias = originals[0].clone();
    let removed = p
        .unit_views()
        .find(|unit| unit.source_range() == t::Range::new(0, 1))
        .expect("actual positive-source Deleted unit")
        .handle();
    assert!(removed.view().processed_range().is_empty());
    assert!(
        removed.view().empty_origin().is_none(),
        "this is not an empty original"
    );
    let removed_alias = removed.clone();
    let alias_facts = || {
        removed_alias
            .view()
            .origins()
            .map(|part| {
                (
                    part.origin(),
                    part.local_range(),
                    part.aggregate_range(),
                    part.processed_range().is_empty(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        alias_facts(),
        [(
            source_alias.clone(),
            t::Range::new(0, 1),
            t::Range::new(0, 1),
            true
        )]
    );

    let controls = p
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let scope = controls[0].captured_control().unwrap().scope();
    assert_eq!(scope.role(), t::ScopeRole::Inline);
    assert_eq!(scope.entry_control(), controls[0].captured_control());
    assert_eq!(scope.exit_control(), controls[1].captured_control());
    assert_eq!(controls[1].captured_control().unwrap().scope(), scope);
    let domain = scope.parent().unwrap();
    assert_eq!(domain.role(), t::ScopeRole::Domain);
    let original_event = source_alias.view().capture_position().unwrap();
    assert!(scope.enter_position() < controls[0].captured_control().unwrap().event_position());
    assert!(controls[0].captured_control().unwrap().event_position() < original_event);
    assert!(original_event < controls[1].captured_control().unwrap().event_position());
    assert!(controls[1].captured_control().unwrap().event_position() < scope.exit_position());
    assert!(scope.exit_position() < controls[2].captured_control().unwrap().event_position());
    assert!(scope.exit_position() < originals[1].view().capture_position().unwrap());
    let segment = p.source_segments().next().unwrap();
    assert_eq!(p.source_segments().len(), 1);
    let mut cuts = soft_at(&p, 1).into_iter().filter(|offer| {
        let count = offer.view().boundary().view().participant_offset();
        matches!(p.participants().take(count).last(),
            Some(t::PreparedParticipant::Control(actual)) if actual == controls[2])
    });
    let after = cuts
        .next()
        .expect("real optional cut after the deleted box closes")
        .view()
        .boundary();
    assert!(cuts.next().is_none());
    assert_eq!(
        segment.start().view().processed_position(),
        after.view().processed_position()
    );
    assert!(p.boundary_consumes_control(&after, &controls[1]).unwrap());
    assert!(
        !p.boundary_inside_captured_scope(&segment, &after, &scope)
            .unwrap()
    );
    assert!(
        p.participants()
            .take(after.view().participant_offset())
            .any(|part| part == t::PreparedParticipant::Text(removed.clone()))
    );

    let owner = t::InlineBoxSource::captured(scope.clone());
    let parent = t::InlineBoxSource::captured(domain);
    let context = p
        .inline_box_edge_context(owner.clone(), parent.clone(), t::LineEdgeSide::Start)
        .unwrap();
    let mut input = request(&p, segment.start(), paragraph_end(&p), &[], 100.0);
    input.context.spacing = t::SpacingPolicy::None;
    for row in &mut input.controls {
        if row.occurrence == controls[0] || row.occurrence == controls[1] {
            row.boundary = Some(t::InlineBoundaryAllocation::box_edge(
                context.clone(),
                t::InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap(),
            ));
        } else {
            assert_eq!(row.occurrence, controls[2]);
            assert!(row.boundary.is_none());
        }
    }
    assert!(p.boundary_context_demand(&input).unwrap().is_none());
    let prior = finalize(&p, input.clone());
    assert!(prior.inline_box_pieces().is_none());
    let authentic_units = p.unit_views().map(|view| view.handle()).collect::<Vec<_>>();
    let family = p.inline_box_family(vec![], None).unwrap();
    input.inline_boxes = Some(family.clone());
    let selected = p.select(input).unwrap();
    let selected_ids = selected
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .map(|piece| piece.fragment())
        .collect::<Vec<_>>();
    assert_eq!(selected_ids.len(), 1);
    let output = selected
        .finalize(t::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: t::ScopeAllocations::None,
        })
        .unwrap();
    let pieces = output
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].fragment(), selected_ids[0]);
    assert_eq!(pieces[0].fragment().parent().unwrap().owner(), parent);
    assert_eq!(pieces[0].start_control(), Some(controls[0].clone()));
    assert_eq!(pieces[0].end_control(), Some(controls[1].clone()));
    for member in pieces[0].members() {
        match member.participant() {
            Some(t::PreparedParticipant::Text(unit)) => assert!(authentic_units.contains(unit)),
            Some(t::PreparedParticipant::Atomic(_)) | Some(t::PreparedParticipant::Cell(_)) => {
                panic!("this source contains neither atomic nor Cell content")
            }
            _ => {}
        }
    }
    // Source aliases are independent of exclusive member/packet ownership.
    // Do not demand that the deleted-only piece exclusively owns this unit.
    for line in [&prior, &output] {
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(ids(line), [(t::Range::new(1, 2), 35)]);
        let packet = line
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), AHEM);
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance(), run.origin().x + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(packet, [(35, 10.0, 0.0)]);
        let deleted_mapping = line
            .mapping_views()
            .filter(|view| {
                view.kind() == t::PresentationMappingKind::Deleted
                    && view.source_range() == t::Range::new(0, 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(deleted_mapping.len(), 1);
        assert!(deleted_mapping[0].processed_range().is_empty());
        assert_eq!(deleted_mapping[0].presentation_range().len(), 0);
        assert!(
            line.selected_line()
                .request()
                .participants
                .contains(&t::PreparedParticipant::Text(removed_alias.clone()))
        );
        for handle in [&controls[0], &controls[1]] {
            let mut slots = line
                .control_slots()
                .filter(|slot| slot.control() == *handle);
            let slot = slots.next().unwrap();
            assert!(slots.next().is_none());
            let (from, to) = slot.flow_interval();
            assert_eq!(from, to);
        }
    }
    assert_eq!(source_alias.view().source().text(), &t::Text::from(" A"));
    assert_eq!(
        alias_facts(),
        [(source_alias, t::Range::new(0, 1), t::Range::new(0, 1), true)]
    );
}
