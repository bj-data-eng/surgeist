//! Literal LF and zero-advance continuations retain authentic body custody.
use super::*;

fn zero_tail_request(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    opening: &t::PreparedControl,
    closing: &t::PreparedControl,
    context: &t::InlineBoxEdgeContext,
) -> t::LineSelectionRequest {
    let edge = || {
        t::InlineBoundaryAllocation::box_edge(
            context.clone(),
            t::InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap(),
        )
    };
    let mut input = request(p, start, end, &[], 100.0);
    input.context.spacing = t::SpacingPolicy::None;
    for row in &mut input.controls {
        if &row.occurrence == opening || &row.occurrence == closing {
            row.boundary = Some(edge());
        } else {
            assert_eq!(row.occurrence.view().kind(), &K::ForcedBreak);
            assert!(row.boundary.is_none());
        }
    }
    while let Some(occurrence) = p.boundary_context_demand(&input).unwrap() {
        assert!(&occurrence == opening || &occurrence == closing);
        input
            .boundary_context
            .push(t::InlineBoundaryContextAllocation {
                occurrence,
                boundary: edge(),
            });
    }
    input
}

fn actual_parts(
    p: &t::PreparedText,
) -> (
    t::PreparedControl,
    t::PreparedControl,
    t::CapturedScope,
    t::InlineBoxEdgeContext,
) {
    let controls = p
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let opening = controls.first().unwrap().clone();
    let closing = controls.last().unwrap().clone();
    assert_eq!(opening.view().kind(), &K::InlineBoundaryStart);
    assert_eq!(closing.view().kind(), &K::InlineBoundaryEnd);
    let scope = opening.captured_control().unwrap().scope();
    assert_eq!(scope.role(), t::ScopeRole::Inline);
    assert_eq!(scope.entry_control(), opening.captured_control());
    assert_eq!(scope.exit_control(), closing.captured_control());
    assert_eq!(closing.captured_control().unwrap().scope(), scope);
    let domain = scope.parent().unwrap();
    assert_eq!(domain.role(), t::ScopeRole::Domain);
    let context = p
        .inline_box_edge_context(
            t::InlineBoxSource::captured(scope.clone()),
            t::InlineBoxSource::captured(domain),
            t::LineEdgeSide::Start,
        )
        .unwrap();
    (opening, closing, scope, context)
}

fn mandatory(
    p: &t::PreparedText,
    scope: &t::CapturedScope,
    closing: &t::PreparedControl,
) -> t::PreparedOpportunity {
    let mut offers = p
        .opportunity_views()
        .filter(|view| view.kind() == t::PreparedBreakKind::Forced)
        .map(|view| view.handle());
    let offer = offers.next().expect("one actual mandatory break");
    assert!(offers.next().is_none());
    let cut = offer.view().boundary();
    let segment = p.source_segments().next().unwrap();
    assert_eq!(p.source_segments().len(), 1);
    assert!(!p.boundary_consumes_control(&cut, closing).unwrap());
    assert!(
        p.boundary_inside_captured_scope(&segment, &cut, scope)
            .unwrap()
    );
    assert!(
        cut.view().participant_offset()
            < paragraph_end(p)
                .view()
                .boundary()
                .view()
                .participant_offset()
    );
    offer
}

fn two_body_custody(
    p: &t::PreparedText,
    scope: &t::CapturedScope,
    closing: &t::PreparedControl,
    mut first_input: t::LineSelectionRequest,
    mut tail_input: t::LineSelectionRequest,
    closing_width: f64,
) {
    assert!(first_input.inline_boxes.is_none() && tail_input.inline_boxes.is_none());
    assert!(
        tail_input
            .participants
            .contains(&t::PreparedParticipant::Control(closing.clone()))
    );
    // These real no-family premises must pass before capability construction.
    let prior_first = finalize(p, first_input.clone());
    let prior_tail = finalize(p, tail_input.clone());
    assert_eq!(ids(&prior_first), [(t::Range::new(0, 1), 35)]);
    assert_eq!(prior_first.feedback().advance, 10.0);
    assert!(ids(&prior_tail).is_empty());
    assert_eq!(prior_tail.feedback().advance, closing_width);
    assert!(
        prior_first
            .control_slots()
            .all(|slot| slot.control() != *closing)
    );
    let actual_end = prior_tail
        .control_slots()
        .find(|slot| slot.control() == *closing)
        .unwrap();
    let (from, to) = actual_end.flow_interval();
    assert_eq!(to - from, closing_width);
    for run in prior_first.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        for glyph in run.glyphs() {
            assert_eq!(glyph.advance(), 10.0);
        }
    }
    let family = p.inline_box_family(vec![], None).unwrap();
    first_input.inline_boxes = Some(family.clone());
    tail_input.inline_boxes = Some(family.clone());
    let first = finalize(p, first_input);
    let tail = finalize(p, tail_input);
    assert_eq!(ids(&first), [(t::Range::new(0, 1), 35)]);
    assert_eq!(first.feedback().advance, 10.0);
    assert!(ids(&tail).is_empty());
    assert_eq!(tail.feedback().advance, closing_width);
    let owner = t::InlineBoxSource::captured(scope.clone());
    let root_owner = t::InlineBoxSource::captured(scope.parent().unwrap());
    let first_pieces = first
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    let tail_pieces = tail
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(first_pieces.len(), 1);
    assert_eq!(
        tail_pieces.len(),
        1,
        "a genuine trailing body survives independently of paint/advance"
    );
    assert_eq!(first_pieces[0].end_control(), None);
    assert_eq!(tail_pieces[0].end_control(), Some(closing.clone()));
    let edge = tail_pieces[0]
        .end_edge()
        .expect("actual selected closing edge, even when zero");
    assert_eq!(edge.control(), *closing);
    let (from, to) = edge.flow_interval();
    assert_eq!(to - from, closing_width);
    let a = first_pieces[0].fragment();
    let b = tail_pieces[0].fragment();
    assert_ne!(a, b);
    let ap = a.parent().unwrap();
    let bp = b.parent().unwrap();
    assert_eq!(ap.owner(), root_owner);
    assert_eq!(bp.owner(), root_owner);
    assert_ne!(ap, bp);
    assert!(matches!(
        a.nearest_common_ancestor(&b).unwrap_err().detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::InlineBoxFragmentation(
                t::InlineBoxFragmentationError::DifferentBody
            )
        })
    ));
    assert_eq!(prior_first.feedback().advance, 10.0);
    assert_eq!(prior_tail.feedback().advance, closing_width);
}

#[test]
fn preserved_newline_in_one_original_retains_its_following_closing_piece() {
    let p = prepare_capture(
        t::Source::identified("A\n", t::Id::from_u64(1_646_405), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
        ],
        AHEM,
    );
    assert_eq!(p.analyzed_source().source().text(), &t::Text::from("A\n"));
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::Text::from("A\n")
    );
    let originals = p
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 1);
    assert_eq!(originals[0].view().local_range(), t::Range::new(0, 2));
    assert_eq!(originals[0].view().aggregate_range(), t::Range::new(0, 2));
    assert!(
        p.control_views().all(|view| *view.kind() != K::ForcedBreak),
        "the actual preserved LF is not replaced by a fabricated captured control"
    );
    let (opening, closing, scope, context) = actual_parts(&p);
    assert!(scope.enter_position() < originals[0].view().capture_position().unwrap());
    assert!(
        originals[0].view().capture_position().unwrap()
            < closing.captured_control().unwrap().event_position()
    );
    assert!(closing.captured_control().unwrap().event_position() < scope.exit_position());
    let end = mandatory(&p, &scope, &closing);
    let tail_start = end.view().boundary();
    assert_eq!(tail_start.view().source_after().get(), 2);
    let segment = p.source_segments().next().unwrap();
    let first = trailing_request(&p, segment.start(), end, &opening, &closing, &context);
    let tail = trailing_request(
        &p,
        tail_start,
        paragraph_end(&p),
        &opening,
        &closing,
        &context,
    );
    two_body_custody(&p, &scope, &closing, first, tail, 5.0);
    assert_eq!(originals[0].view().source().text(), &t::Text::from("A\n"));
}

#[test]
fn zero_closing_edge_after_control_break_keeps_real_body_piece_identity() {
    let p = prepare_capture(
        t::Source::identified("A", t::Id::from_u64(1_646_406), 1),
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
    let (opening, closing, scope, context) = actual_parts(&p);
    let forces = p
        .control_views()
        .filter(|view| *view.kind() == K::ForcedBreak)
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(forces.len(), 1);
    let captured = forces[0].captured_control().unwrap();
    assert_eq!(captured.scope(), scope);
    assert!(captured.event_position() < closing.captured_control().unwrap().event_position());
    let end = mandatory(&p, &scope, &closing);
    assert!(
        p.boundary_consumes_control(&end.view().boundary(), &forces[0])
            .unwrap()
    );
    let tail_start = end.view().boundary();
    let segment = p.source_segments().next().unwrap();
    let first = zero_tail_request(&p, segment.start(), end, &opening, &closing, &context);
    let tail = zero_tail_request(
        &p,
        tail_start,
        paragraph_end(&p),
        &opening,
        &closing,
        &context,
    );
    two_body_custody(&p, &scope, &closing, first, tail, 0.0);
}
