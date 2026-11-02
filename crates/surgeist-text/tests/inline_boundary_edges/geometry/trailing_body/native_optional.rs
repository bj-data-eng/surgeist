//! A real optional cut preserves Native identity within the same canonical body.
//! Family identity does not promote ordinary Contributions into BoxEdge authority.
use super::*;

#[test]
fn native_optional_cut_retains_the_same_global_fragment_on_its_closing_suffix() {
    let p = prepare_capture(
        t::Source::identified("A", t::Id::from_u64(1_646_410), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1)],
        &[
            (K::InlineBoundaryStart, 0),
            (
                K::OptionalBreak {
                    wrap: t::Wrap::Word,
                },
                1,
            ),
            (K::InlineBoundaryEnd, 1),
        ],
        &[], // No nominated captured Inline and no fabricated source box.
        vec![control(0), origin(0), control(1), control(2)],
        AHEM,
    );
    assert_eq!(p.analyzed_source().source().text(), &t::Text::from("A"));
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::Text::from("A")
    );
    assert_eq!(p.source_segments().len(), 1);
    let controls = p
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 3);
    let opening = &controls[0];
    let optional = &controls[1];
    let closing = &controls[2];
    assert_eq!(opening.view().kind(), &K::InlineBoundaryStart);
    assert_eq!(
        optional.view().kind(),
        &K::OptionalBreak {
            wrap: t::Wrap::Word
        }
    );
    assert_eq!(closing.view().kind(), &K::InlineBoundaryEnd);
    let start_source = opening.captured_control().unwrap();
    let optional_source = optional.captured_control().unwrap();
    let end_source = closing.captured_control().unwrap();
    let domain = start_source.scope();
    assert_eq!(domain.role(), t::ScopeRole::Domain);
    assert_eq!(
        domain.domain_topology(),
        t::SourceProcessingTopology::Complete
    );
    assert_eq!(optional_source.scope(), domain);
    assert_eq!(end_source.scope(), domain);
    assert!(domain.entry_control().is_none() && domain.exit_control().is_none());
    let captured = p.captured_owner().unwrap();
    assert_eq!(captured.domain(), domain);
    assert!(captured.members().all(|member| {
        member
            .scope_intersections()
            .all(|intersection| intersection.scope().role() == t::ScopeRole::Domain)
    }));
    let originals = p
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 1);
    assert_eq!(originals[0].view().aggregate_range(), t::Range::new(0, 1));
    let original_event = originals[0].view().capture_position().unwrap();
    assert!(start_source.event_position() < original_event);
    assert!(original_event < optional_source.event_position());
    assert!(optional_source.event_position() < end_source.event_position());
    assert!(end_source.event_position() < domain.exit_position());
    let native = p
        .native_inline_box(start_source.clone(), end_source.clone())
        .unwrap();
    assert!(
        native.captured_scope().is_none(),
        "actual NativePair, not canonical CapturedInline"
    );
    let root_owner = t::InlineBoxSource::captured(domain.clone());
    assert_ne!(native, root_owner);
    assert_eq!(
        native,
        p.native_inline_box(start_source, end_source).unwrap()
    );

    assert!(
        p.opportunity_views()
            .all(|v| v.kind() != t::PreparedBreakKind::Forced)
    );
    let mut offers = soft_at(&p, 1).into_iter().filter(|offer| {
        let cut = offer.view().boundary();
        let count = cut.view().participant_offset();
        matches!(p.participants().take(count).last(),
            Some(t::PreparedParticipant::Control(actual)) if actual == *optional)
            && !p.boundary_consumes_control(&cut, closing).unwrap()
    });
    let optional_cut = offers
        .next()
        .expect("genuine native optional cut before End");
    assert!(offers.next().is_none());
    assert_eq!(optional_cut.view().kind(), t::PreparedBreakKind::Allowed);
    let tail_start = optional_cut.view().boundary();
    assert!(p.boundary_consumes_control(&tail_start, optional).unwrap());
    assert!(!p.boundary_consumes_control(&tail_start, closing).unwrap());
    let paragraph = paragraph_end(&p);
    assert!(
        p.boundary_consumes_control(&paragraph.view().boundary(), closing)
            .unwrap()
    );
    assert!(
        tail_start.view().participant_offset()
            < paragraph.view().boundary().view().participant_offset()
    );
    let segment = p.source_segments().next().unwrap();
    // Existing request/answer helpers construct only ordinary Contributions.
    // No inline_box_edge_context or BoxEdge allocation is requested anywhere.
    let facts = [(0, 0.0), (2, 5.0)];
    let mut first_input = request(&p, segment.start(), optional_cut, &facts, 100.0);
    let mut tail_input = request(&p, tail_start, paragraph, &facts, 100.0);
    for input in [&mut first_input, &mut tail_input] {
        input.context.spacing = t::SpacingPolicy::None;
        answer(&p, input, &facts);
        assert!(input.inline_boxes.is_none());
    }
    assert!(
        tail_input
            .participants
            .contains(&t::PreparedParticipant::Control(closing.clone()))
    );
    assert!(
        !tail_input
            .participants
            .contains(&t::PreparedParticipant::Control(optional.clone()))
    );
    assert!(tail_input.participants.iter().all(|part| match part {
        t::PreparedParticipant::Control(actual) => actual == closing,
        t::PreparedParticipant::Text(unit) =>
            unit.view().source_range().is_empty() && unit.view().processed_range().is_empty(),
        _ => false,
    }));

    // Establish actual local behavior before demanding a complete native family.
    let prior_first = finalize(&p, first_input.clone());
    let prior_tail = finalize(&p, tail_input.clone());
    assert_eq!(ids(&prior_first), [(t::Range::new(0, 1), 35)]);
    assert_eq!(prior_first.feedback().advance, 10.0);
    assert!(ids(&prior_tail).is_empty());
    assert_eq!(prior_tail.feedback().advance, 5.0);
    assert!(
        prior_first
            .control_slots()
            .all(|slot| slot.control() != *closing)
    );
    let ordinary_end = prior_tail
        .control_slots()
        .find(|slot| slot.control() == *closing)
        .unwrap();
    let (from, to) = ordinary_end.flow_interval();
    assert_eq!(to - from, 5.0);
    for run in prior_first.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        for glyph in run.glyphs() {
            assert_eq!(glyph.advance(), 10.0);
        }
    }

    let family = p.inline_box_family(vec![native.clone()], None).unwrap();
    assert_eq!(family.prepared_text(), p);
    first_input.inline_boxes = Some(family.clone());
    tail_input.inline_boxes = Some(family.clone());
    let first = finalize(&p, first_input);
    let tail = finalize(&p, tail_input);
    assert_eq!(ids(&first), [(t::Range::new(0, 1), 35)]);
    assert_eq!(first.feedback().advance, 10.0);
    assert!(ids(&tail).is_empty());
    assert_eq!(tail.feedback().advance, 5.0);
    let actual_end = tail
        .control_slots()
        .find(|slot| slot.control() == *closing)
        .unwrap();
    assert_eq!(actual_end.flow_interval(), ordinary_end.flow_interval());
    let first_pieces = first
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == native)
        .collect::<Vec<_>>();
    let tail_pieces = tail
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == native)
        .collect::<Vec<_>>();
    assert_eq!(first_pieces.len(), 1);
    assert_eq!(
        tail_pieces.len(),
        1,
        "declared NativePair identity survives its same-body End-only suffix"
    );
    // A complete identity capability grants no BoxEdge geometry operation.
    for piece in [&first_pieces[0], &tail_pieces[0]] {
        assert!(piece.start_control().is_none() && piece.end_control().is_none());
        assert!(piece.start_edge().is_none() && piece.end_edge().is_none());
    }
    let a = first_pieces[0].fragment();
    let b = tail_pieces[0].fragment();
    assert_eq!(
        a, b,
        "a real optional cut does not create a new global fragment"
    );
    assert_eq!(a.parent().unwrap().owner(), root_owner);
    assert_eq!(a.parent(), b.parent());
    assert_eq!(prior_first.feedback().advance, 10.0);
    assert_eq!(prior_tail.feedback().advance, 5.0);
    assert_eq!(originals[0].view().source().text(), &t::Text::from("A"));
}
