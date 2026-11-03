//! Real source members remain distinct while their actual provider packet is shared.
use super::*;

fn captured_siblings(mark: bool) -> (t::PreparedText, Vec<u8>) {
    let bytes = font::ligature();
    let p = prepare_capture(
        t::Source::identified(
            if mark { "A\u{301}B" } else { "AAB" },
            t::Id::from_u64(if mark { 1_646_101 } else { 1_646_102 }),
            1,
        ),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1), (1, 2), (2, 3)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryEnd, 1),
            (K::InlineBoundaryStart, 1),
            (K::InlineBoundaryEnd, 2),
        ],
        &[(0, 1), (2, 3)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            E::Connection(0),
            E::EnterScope(2),
            control(2),
            origin(1),
            control(3),
            E::ExitScope(2),
            E::Connection(1),
            origin(2),
        ],
        &bytes,
    );
    (p, bytes)
}

fn unit_at(p: &t::PreparedText, source: t::Range) -> t::PreparedUnit {
    let mut found = p.unit_views().filter(|unit| unit.source_range() == source);
    let unit = found.next().expect("actual authored source unit").handle();
    assert!(found.next().is_none());
    unit
}

fn owner_at(p: &t::PreparedText, start_index: usize, end_index: usize) -> t::InlineBoxSource {
    let start = p
        .control_views()
        .nth(start_index)
        .unwrap()
        .handle()
        .captured_control()
        .unwrap();
    let end = p
        .control_views()
        .nth(end_index)
        .unwrap()
        .handle()
        .captured_control()
        .unwrap();
    let scope = start.scope();
    assert_eq!(scope.role(), t::ScopeRole::Inline);
    assert_eq!(scope.entry_control(), Some(start));
    assert_eq!(scope.exit_control(), Some(end));
    assert_eq!(
        scope.domain_topology(),
        t::SourceProcessingTopology::Complete
    );
    assert_eq!(scope.parent().unwrap().role(), t::ScopeRole::Domain);
    t::InlineBoxSource::captured(scope)
}

fn packet(line: &t::FinalizedLine, bytes: &[u8]) -> Vec<(u32, f32, f64, f32, t::Range)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            let x = run.origin().x;
            run.glyphs()
                .map(move |glyph| {
                    (
                        glyph.id(),
                        glyph.advance(),
                        x + glyph.pen() + f64::from(glyph.x()),
                        glyph.y(),
                        glyph.range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn owned(line: &t::FinalizedLine, owner: &t::InlineBoxSource) -> t::FinalizedInlineBoxPiece {
    let mut found = line
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| &piece.owner() == owner);
    let piece = found
        .next()
        .expect("one LTR fragment for this genuine sibling");
    assert!(found.next().is_none());
    piece
}

fn text_members(piece: &t::FinalizedInlineBoxPiece) -> Vec<t::PreparedUnit> {
    piece
        .members()
        .filter_map(|member| match member.participant() {
            Some(t::PreparedParticipant::Text(unit)) => {
                assert_eq!(member.unit_source_range(), unit.view().source_range());
                Some(unit.clone())
            }
            _ => None,
        })
        .collect()
}

fn shared_packet(mark: bool) {
    let (p, bytes) = captured_siblings(mark);
    // The maintained font::ligature supplies cmap A35/acute35/B36 and actual
    // DFLT/ccmp lookup4: glyph35 + glyph35 -> X58. This is not optional liga,
    // fake provider coverage, a mark-to-base attachment or a synthetic TCU.
    let word = |table: &[u8], at: usize| u16::from_be_bytes(table[at..at + 2].try_into().unwrap());
    assert_eq!(word(font::table(&bytes, *b"head"), 18), 1000);
    for glyph in [35, 36, 58] {
        assert_eq!(word(font::table(&bytes, *b"hmtx"), glyph * 4), 1000);
    }
    let analysis = p.analyzed_source();
    assert_eq!(
        analysis.source().text().code_units(),
        if mark {
            vec![65, 769, 66]
        } else {
            vec![65, 65, 66]
        }
    );
    assert_eq!(
        analysis.graphemes().collect::<Vec<_>>(),
        if mark {
            vec![t::Range::new(0, 2), t::Range::new(2, 3)]
        } else {
            vec![
                t::Range::new(0, 1),
                t::Range::new(1, 2),
                t::Range::new(2, 3),
            ]
        }
    );
    assert!(p.unit_views().all(|unit| unit.level().get() == 0));
    let units = [
        unit_at(&p, t::Range::new(0, 1)),
        unit_at(&p, t::Range::new(1, 2)),
        unit_at(&p, t::Range::new(2, 3)),
    ];
    assert_eq!(
        p.origin_views()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [
            t::Range::new(0, 1),
            t::Range::new(1, 2),
            t::Range::new(2, 3)
        ]
    );
    let left = owner_at(&p, 0, 1);
    let right = owner_at(&p, 2, 3);
    assert_ne!(left, right);
    let facts = [(0, 0.0), (1, 0.0), (2, 0.0), (3, 0.0)];
    let mut basis_request = request(
        &p,
        p.boundary_views().next().unwrap().handle(),
        paragraph_end(&p),
        &facts,
        100.0,
    );
    answer(&p, &mut basis_request, &facts);
    assert!(basis_request.inline_boxes.is_none());
    let baseline = finalize(&p, basis_request);
    let expected = [
        (58, 10.0, 0.0, 0.0, t::Range::new(0, 2)),
        (36, 10.0, 10.0, 0.0, t::Range::new(2, 3)),
    ];
    assert_eq!(packet(&baseline, &bytes), expected);
    assert_eq!(baseline.feedback().advance, 20.0);
    assert!(baseline.inline_box_pieces().is_none());

    // Complete topology comes from the actual root Block; all three originals
    // belong to its one real raw paragraph, so this exact view covers its body.
    // Captured siblings are automatic inventory, not a fabricated NativePair.
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    assert_eq!(family.prepared_text(), p);
    let mut with_family = baseline.selected_line().request().clone();
    with_family.inline_boxes = Some(family.clone());
    let selected = p.select(with_family).unwrap();
    assert_eq!(selected.inline_box_family(), Some(&family));
    let mut selected_fragments = Vec::new();
    for (owner, unit) in [(&left, &units[0]), (&right, &units[1])] {
        let mut pieces = selected
            .inline_box_pieces()
            .unwrap()
            .filter(|piece| &piece.owner() == owner);
        let piece = pieces.next().unwrap();
        assert!(pieces.next().is_none());
        assert_eq!(
            piece
                .members()
                .filter_map(|m| match m.participant() {
                    Some(t::PreparedParticipant::Text(u)) => Some(u.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            std::slice::from_ref(unit)
        );
        selected_fragments.push(piece.fragment());
    }
    let line = selected
        .finalize(t::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: t::ScopeAllocations::None,
        })
        .unwrap();
    let a = owned(&line, &left);
    let b = owned(&line, &right);
    assert_eq!(text_members(&a), [units[0].clone()]);
    assert_eq!(text_members(&b), [units[1].clone()]);
    assert_eq!(a.fragment(), selected_fragments[0]);
    assert_eq!(b.fragment(), selected_fragments[1]);
    assert_ne!(
        a.fragment(),
        b.fragment(),
        "shared ink does not merge source boxes"
    );
    assert!(a.shares_packet_with(&b));
    assert!(b.shares_packet_with(&a));
    assert_eq!(a.family(), &family);
    assert_eq!(b.family(), &family);
    assert_eq!(a.finalized_line(), &line);
    assert_eq!(b.finalized_line(), &line);
    let parent = a.fragment().parent().unwrap();
    assert_eq!(b.fragment().parent(), Some(parent.clone()));
    assert_eq!(
        a.fragment().nearest_common_ancestor(&b.fragment()).unwrap(),
        parent
    );
    // Whole range0..2 stays on ONE real output glyph even when it contains two
    // TCUs. Each sibling retains only its own real original member; neither
    // receives the other source unit or the following B by packet closure.
    assert_eq!(packet(&line, &bytes), expected);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(line.inline_box_family(), Some(&family));
    assert_eq!(
        line.control_slots()
            .map(|c| (c.control(), c.flow_interval(), c.strip_interval()))
            .collect::<Vec<_>>(),
        baseline
            .control_slots()
            .map(|c| (c.control(), c.flow_interval(), c.strip_interval()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        packet(&baseline, &bytes),
        expected,
        "the prior no-family output remains usable"
    );
    assert!(baseline.inline_box_pieces().is_none());
}

#[test]
fn sibling_base_and_acute_scopes_share_one_real_ccmp_packet_without_lending_source_members() {
    shared_packet(true);
}

#[test]
fn sibling_egcs_retain_distinct_members_and_fragments_inside_one_real_ccmp_packet() {
    shared_packet(false);
}
