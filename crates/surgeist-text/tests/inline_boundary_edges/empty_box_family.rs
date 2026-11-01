//! Genuine empty box identity and source custody across wrapping.
use super::*;

fn with_empty_sibling() -> t::PreparedText {
    prepare_capture(
        t::Source::identified("abcABC", t::Id::from_u64(1_646_101), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2), (2, 3), (3, 4), (4, 6)],
        &[
            (K::InlineBoundaryStart, 2),
            (K::RightToLeftOverride, 3),
            (K::InlineBoundaryEnd, 4),
            (K::InlineBoundaryStart, 4),
            (K::InlineBoundaryEnd, 4),
            (
                K::OptionalBreak {
                    wrap: t::Wrap::Word,
                },
                4,
            ),
            (K::PopDirectionalFormat, 6),
        ],
        &[(0, 2), (3, 4)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            E::Connection(1),
            control(1),
            origin(2),
            control(2),
            E::ExitScope(1),
            E::EnterScope(2),
            control(3),
            control(4),
            E::ExitScope(2),
            control(5),
            E::Connection(2),
            origin(3),
            control(6),
        ],
        AHEM,
    )
}

fn zero_line(
    p: &t::PreparedText,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    width: f64,
    family: Option<&t::PreparedInlineBoxFamily>,
) -> t::FinalizedLine {
    let mut input = request(p, start, end, &[], width);
    input.context.spacing = t::SpacingPolicy::None;
    input.inline_boxes = family.cloned();
    let zero_boundary = |control: &t::PreparedControl| {
        let scope = control.captured_control().unwrap().scope();
        let parent = scope.parent().unwrap();
        let context = p
            .inline_box_edge_context(
                t::InlineBoxSource::captured(scope),
                t::InlineBoxSource::captured(parent),
                t::LineEdgeSide::Start,
            )
            .unwrap();
        t::InlineBoundaryAllocation::box_edge(
            context,
            t::InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap(),
        )
    };
    for row in &mut input.controls {
        match row.occurrence.view().kind() {
            K::InlineBoundaryStart | K::InlineBoundaryEnd => {
                row.boundary = Some(zero_boundary(&row.occurrence));
            }
            _ => assert!(row.boundary.is_none()),
        }
    }
    while let Some(control) = p.boundary_context_demand(&input).unwrap() {
        let boundary = zero_boundary(&control);
        input
            .boundary_context
            .push(t::InlineBoundaryContextAllocation {
                occurrence: control,
                boundary,
            });
    }
    finalize(p, input)
}

fn one_owned_piece(
    line: &t::FinalizedLine,
    owner: &t::InlineBoxSource,
) -> t::FinalizedInlineBoxPiece {
    let mut pieces = line
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| &piece.owner() == owner);
    let piece = pieces
        .next()
        .expect("the genuine empty Inline retains its own piece");
    assert!(pieces.next().is_none());
    piece
}

#[test]
fn empty_captured_inline_retains_family_identity_when_following_text_wraps() {
    let p = with_empty_sibling();
    assert_eq!(
        p.analyzed_source().source().text(),
        &t::Text::from("abcABC")
    );
    assert_eq!(p.source_segments().len(), 1);
    assert_eq!(
        p.origin_views()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [
            t::Range::new(0, 2),
            t::Range::new(2, 3),
            t::Range::new(3, 4),
            t::Range::new(4, 6),
        ]
    );
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 7);
    let empty_start = controls[3].captured_control().unwrap();
    let empty_end = controls[4].captured_control().unwrap();
    let empty = empty_start.scope();
    assert_eq!(empty.role(), t::ScopeRole::Inline);
    assert_eq!(empty.entry_control(), Some(empty_start.clone()));
    assert_eq!(empty.exit_control(), Some(empty_end.clone()));
    assert_eq!(empty_end.scope(), empty);
    let occupied = controls[0].captured_control().unwrap().scope();
    let domain = empty.parent().unwrap();
    assert_eq!(occupied.parent(), Some(domain.clone()));
    assert_ne!(empty, occupied);
    assert_eq!(domain.role(), t::ScopeRole::Domain);
    assert_eq!(empty_start.source_position().get(), 4);
    assert_eq!(empty_end.source_position().get(), 4);

    // Coincident source offsets establish no custody. These are real ordered
    // captured box events after cA closes and before the actual OptionalBreak.
    let enter = empty.enter_position();
    let exit = empty.exit_position();
    assert!(controls[2].captured_control().unwrap().event_position() < enter);
    assert!(enter < empty_start.event_position());
    assert!(empty_start.event_position() < empty_end.event_position());
    assert!(empty_end.event_position() < exit);
    assert!(exit < controls[5].captured_control().unwrap().event_position());
    assert!(
        p.origin_views().all(|view| {
            let at = view.capture_position().unwrap();
            !(enter < at && at < exit)
        }),
        "the real empty box contains no original occurrence"
    );

    let optional = controls[5].clone();
    assert_eq!(
        optional.view().kind(),
        &K::OptionalBreak {
            wrap: t::Wrap::Word
        }
    );
    let mut cuts = soft_at(&p, 4).into_iter().filter(|offer| {
        let count = offer.view().boundary().view().participant_offset();
        matches!(p.participants().take(count).last(),
            Some(t::PreparedParticipant::Control(previous)) if previous == optional)
    });
    let cut = cuts
        .next()
        .expect("actual allowed cut after this source OptionalBreak");
    assert!(cuts.next().is_none());
    assert_eq!(cut.view().kind(), t::PreparedBreakKind::Allowed);
    let continuation = cut.view().boundary();
    let segment = p.source_segments().next().unwrap();

    // Pre-family public receipts establish real candidate markers. Nonempty
    // originals alone do not prove that preparation issued no zero marker.
    let authentic_zero_units = p
        .unit_views()
        .filter(|unit| {
            unit.source_range() == t::Range::new(4, 4) && unit.processed_range().is_empty()
        })
        .map(|unit| unit.handle())
        .collect::<Vec<_>>();

    // Local zero-edge controls are valid independently of the new family front.
    let local = zero_line(&p, segment.start(), paragraph_end(&p), 100.0, None);
    assert!(local.inline_box_pieces().is_none());
    assert_eq!(local.feedback().advance, 60.0);
    let expected_wide = [
        (t::Range::new(0, 1), 67, 0.0),
        (t::Range::new(1, 2), 68, 10.0),
        (t::Range::new(2, 3), 69, 20.0),
        (t::Range::new(5, 6), 37, 30.0),
        (t::Range::new(4, 5), 36, 40.0),
        (t::Range::new(3, 4), 35, 50.0),
    ];
    assert_eq!(packet(&local), expected_wide);

    let family = p.inline_box_family(vec![], None).unwrap();
    let wide = zero_line(&p, segment.start(), paragraph_end(&p), 100.0, Some(&family));
    let first = zero_line(&p, segment.start(), cut, 40.0, Some(&family));
    let second = zero_line(&p, continuation, paragraph_end(&p), 40.0, Some(&family));
    assert_eq!(packet(&wide), expected_wide);
    assert_eq!(wide.feedback().advance, 60.0);
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
            (t::Range::new(3, 4), 35, 30.0),
        ]
    );
    assert_eq!(
        packet(&second),
        [
            (t::Range::new(5, 6), 37, 0.0),
            (t::Range::new(4, 5), 36, 10.0),
        ]
    );

    let empty_owner = t::InlineBoxSource::captured(empty);
    let root_owner = t::InlineBoxSource::captured(domain);
    let wide_piece = one_owned_piece(&wide, &empty_owner);
    let first_piece = one_owned_piece(&first, &empty_owner);
    assert_eq!(
        wide_piece.fragment(),
        first_piece.fragment(),
        "one global family identity survives wrapping of the following donors"
    );
    for (line, finalized_piece) in [(&wide, &wide_piece), (&first, &first_piece)] {
        let selected = line.selected_line();
        let pieces = selected
            .inline_box_pieces()
            .unwrap()
            .filter(|piece| piece.owner() == empty_owner)
            .collect::<Vec<_>>();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].fragment(), finalized_piece.fragment());
    }
    for piece in [&wide_piece, &first_piece] {
        assert_eq!(piece.family(), &family);
        let parent = piece.fragment().parent().expect("actual domain ancestor");
        assert_eq!(parent.owner(), root_owner);
        assert!(parent.parent().is_none());
        assert_eq!(piece.start_control(), Some(controls[3].clone()));
        assert_eq!(piece.end_control(), Some(controls[4].clone()));
        assert!(
            piece.members().all(|member| match member.participant() {
                Some(t::PreparedParticipant::Text(unit)) => {
                    // An already-issued zero marker is honest incidence, not
                    // borrowed neighboring text or a newly manufactured scalar.
                    assert!(authentic_zero_units.contains(unit));
                    assert_eq!(unit.view().source_range(), t::Range::new(4, 4));
                    assert_eq!(unit.view().processed_range().len(), 0);
                    assert_eq!(member.unit_source_range(), t::Range::new(4, 4));
                    assert_eq!(member.presentation_range().len(), 0);
                    true
                }
                Some(t::PreparedParticipant::Atomic(_)) => false,
                _ => true,
            }),
            "identity cannot borrow nonempty text or invent atomic content"
        );
    }
    assert!(
        second
            .inline_box_pieces()
            .unwrap()
            .all(|piece| piece.owner() != empty_owner),
        "no receiving-line donation of an already selected real empty box"
    );
    for line in [&wide, &first] {
        for handle in [&controls[3], &controls[4]] {
            let mut slots = line
                .control_slots()
                .filter(|slot| slot.control() == *handle);
            let slot = slots
                .next()
                .expect("each actual empty endpoint executes once");
            assert!(slots.next().is_none());
            let (from, to) = slot.flow_interval();
            assert_eq!(from, to, "affirmative zero edge, no positivity predicate");
        }
    }
    let placed = place(&p, &[first, second]);
    assert_eq!(
        placed_packet(&placed),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 20.0, 8.0),
            (35, 30.0, 8.0),
            (37, 0.0, 18.0),
            (36, 10.0, 18.0),
        ]
    );
    let placed_empty = placed
        .inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == empty_owner)
        .collect::<Vec<_>>();
    assert_eq!(placed_empty.len(), 1);
    assert_eq!(placed_empty[0].line_index(), 0);
    assert_eq!(placed_empty[0].fragment(), wide_piece.fragment());
    // Prior no-family publication still owns the same six source glyphs.
    assert_eq!(packet(&local), expected_wide);
}
