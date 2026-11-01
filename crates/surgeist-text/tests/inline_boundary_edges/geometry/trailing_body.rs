//! Closing endpoint custody in a genuine trailing control-only body.
//! A real source ForcedBreak starts a new body before the authentic closing edge.
use super::*;

fn trailing_request(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    opening: &t::PreparedControl,
    closing: &t::PreparedControl,
    context: &t::InlineBoxEdgeContext,
) -> t::LineSelectionRequest {
    let edge = |control: &t::PreparedControl| {
        let width = if control == opening {
            0.0
        } else {
            assert_eq!(
                control, closing,
                "only the genuine paired endpoints have box geometry"
            );
            5.0
        };
        t::InlineBoundaryAllocation::box_edge(
            context.clone(),
            t::InlineBoxEdgeComponents::from_f64(0.0, width, 0.0).unwrap(),
        )
    };
    let mut input = request(p, start, end, &[], 100.0);
    input.context.spacing = t::SpacingPolicy::None;
    assert!(input.inline_boxes.is_none());
    for row in &mut input.controls {
        if &row.occurrence == opening || &row.occurrence == closing {
            row.boundary = Some(edge(&row.occurrence));
        } else {
            assert_eq!(row.occurrence.view().kind(), &K::ForcedBreak);
            assert!(row.boundary.is_none());
        }
    }
    while let Some(occurrence) = p.boundary_context_demand(&input).unwrap() {
        let boundary = edge(&occurrence);
        input
            .boundary_context
            .push(t::InlineBoundaryContextAllocation {
                occurrence,
                boundary,
            });
    }
    input
}

#[test]
fn closing_endpoint_after_forced_break_retains_its_own_trailing_body_piece() {
    // Same real mixed-event chronology as the maintained
    // forced_break_chronology_does_not_relocate_a_later_closing_endpoint case,
    // with no later glyph. No newline/source scalar or empty origin is invented.
    let p = prepare_capture(
        t::Source::identified("A", t::Id::from_u64(1_646_404), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::ForcedBreak, 1),
            (K::InlineBoundaryEnd, 1),
        ],
        &[(0, 2)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            control(2),
            E::ExitScope(1),
        ],
        AHEM,
    );
    assert_eq!(p.analyzed_source().source().text(), &t::Text::from("A"));
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::Text::from("A")
    );
    assert_eq!(p.source_segments().len(), 1);
    let originals = p
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 1);
    assert_eq!(originals[0].view().aggregate_range(), t::Range::new(0, 1));
    assert_eq!(originals[0].view().source().text(), &t::Text::from("A"));
    let controls = p
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 3);
    let opening = &controls[0];
    let force = &controls[1];
    let closing = &controls[2];
    let scope = opening.captured_control().unwrap().scope();
    assert_eq!(scope.role(), t::ScopeRole::Inline);
    assert_eq!(scope.entry_control(), opening.captured_control());
    assert_eq!(scope.exit_control(), closing.captured_control());
    assert_eq!(force.captured_control().unwrap().scope(), scope);
    assert_eq!(closing.captured_control().unwrap().scope(), scope);
    assert_eq!(force.view().kind(), &K::ForcedBreak);
    let original_event = originals[0].view().capture_position().unwrap();
    assert!(scope.enter_position() < opening.captured_control().unwrap().event_position());
    assert!(opening.captured_control().unwrap().event_position() < original_event);
    assert!(original_event < force.captured_control().unwrap().event_position());
    assert!(
        force.captured_control().unwrap().event_position()
            < closing.captured_control().unwrap().event_position()
    );
    assert!(closing.captured_control().unwrap().event_position() < scope.exit_position());
    let domain = scope.parent().unwrap();
    assert_eq!(domain.role(), t::ScopeRole::Domain);
    let owner = t::InlineBoxSource::captured(scope.clone());
    let root = t::InlineBoxSource::captured(domain);
    let context = p
        .inline_box_edge_context(owner.clone(), root.clone(), t::LineEdgeSide::Start)
        .unwrap();

    let mut forced = p
        .opportunity_views()
        .filter(|view| view.kind() == t::PreparedBreakKind::Forced)
        .map(|view| view.handle());
    let first_end = forced
        .next()
        .expect("one actual mandatory break from the captured ForcedBreak");
    assert!(forced.next().is_none());
    let second_start = first_end.view().boundary();
    let paragraph = paragraph_end(&p);
    let final_end = paragraph.view().boundary();
    assert!(second_start.view().participant_offset() < final_end.view().participant_offset());
    assert!(p.boundary_consumes_control(&second_start, force).unwrap());
    assert!(!p.boundary_consumes_control(&second_start, closing).unwrap());
    assert!(p.boundary_consumes_control(&final_end, closing).unwrap());
    let segment = p.source_segments().next().unwrap();
    assert!(
        p.boundary_inside_captured_scope(&segment, &second_start, &scope)
            .unwrap()
    );

    let first_input = trailing_request(&p, segment.start(), first_end, opening, closing, &context);
    let second_input = trailing_request(&p, second_start, paragraph, opening, closing, &context);
    assert!(
        second_input
            .participants
            .contains(&t::PreparedParticipant::Control(closing.clone()))
    );
    assert!(
        !second_input
            .participants
            .contains(&t::PreparedParticipant::Control(force.clone()))
    );
    assert!(
        second_input.participants.iter().all(|part| match part {
            t::PreparedParticipant::Control(control) => control == closing,
            t::PreparedParticipant::Text(unit) =>
                unit.view().source_range().is_empty() && unit.view().processed_range().is_empty(),
            _ => false,
        }),
        "the genuine trailing body has only its actual closing endpoint/zero markers"
    );

    // Establish local admission and actual EndBP5 before requesting the family.
    let local_first = finalize(&p, first_input.clone());
    let local_second = finalize(&p, second_input.clone());
    assert!(local_first.inline_box_pieces().is_none());
    assert!(local_second.inline_box_pieces().is_none());
    assert_eq!(ids(&local_first), [(t::Range::new(0, 1), 35)]);
    assert_eq!(local_first.feedback().advance, 10.0);
    assert_eq!(local_second.feedback().advance, 5.0);
    assert!(ids(&local_second).is_empty());
    assert!(
        local_first
            .control_slots()
            .all(|slot| slot.control() != *closing)
    );
    let local_end = local_second
        .control_slots()
        .find(|slot| slot.control() == *closing)
        .expect("the authentic closing endpoint executes in the trailing body");
    let (from, to) = local_end.flow_interval();
    assert_eq!(to - from, 5.0);

    let family = p.inline_box_family(vec![], None).unwrap();
    let mut first_input = first_input;
    let mut second_input = second_input;
    first_input.inline_boxes = Some(family.clone());
    second_input.inline_boxes = Some(family.clone());
    let first = finalize(&p, first_input);
    let second = finalize(&p, second_input);
    assert_eq!(ids(&first), [(t::Range::new(0, 1), 35)]);
    assert_eq!(first.feedback().advance, 10.0);
    assert!(ids(&second).is_empty());
    assert_eq!(second.feedback().advance, 5.0);
    let first_pieces = first
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    let second_pieces = second
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(first_pieces.len(), 1);
    assert_eq!(
        second_pieces.len(),
        1,
        "a prior body's glyph cannot erase the trailing body's genuine box"
    );
    assert_eq!(first_pieces[0].start_control(), Some(opening.clone()));
    assert_eq!(first_pieces[0].end_control(), None);
    assert_eq!(second_pieces[0].start_control(), None);
    assert_eq!(second_pieces[0].end_control(), Some(closing.clone()));
    let edge = second_pieces[0]
        .end_edge()
        .expect("actual retained EndBP5 execution");
    assert_eq!(edge.control(), *closing);
    let (from, to) = edge.flow_interval();
    assert_eq!(to - from, 5.0);
    let first_id = first_pieces[0].fragment();
    let second_id = second_pieces[0].fragment();
    assert_ne!(first_id, second_id, "separate actual forced-break bodies");
    let first_root = first_id.parent().unwrap();
    let second_root = second_id.parent().unwrap();
    assert_eq!(first_root.owner(), root);
    assert_eq!(second_root.owner(), root);
    assert_ne!(first_root, second_root);
    assert!(matches!(
        first_id
            .nearest_common_ancestor(&second_id)
            .unwrap_err()
            .detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::InlineBoxFragmentation(
                t::InlineBoxFragmentationError::DifferentBody
            )
        })
    ));
    // Old publications and source capability aliases coexist unchanged.
    assert_eq!(local_first.feedback().advance, 10.0);
    assert_eq!(local_second.feedback().advance, 5.0);
    assert_eq!(originals[0].view().source().text(), &t::Text::from("A"));
}
