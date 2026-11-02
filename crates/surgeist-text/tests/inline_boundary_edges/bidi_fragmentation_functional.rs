//! Global box fragments keep their source identity when wrapping changes only
//! the adjacency of their selected pieces.
use super::*;

fn sources(p: &t::PreparedText) -> (t::InlineBoxSource, t::InlineBoxSource) {
    let scope = endpoint(p, K::InlineBoundaryStart)
        .captured_control()
        .unwrap()
        .scope();
    assert_eq!(
        scope.exit_control(),
        endpoint(p, K::InlineBoundaryEnd).captured_control()
    );
    let parent = scope.parent().unwrap();
    assert_eq!(parent.role(), t::ScopeRole::Domain);
    (
        t::InlineBoxSource::captured(scope),
        t::InlineBoxSource::captured(parent),
    )
}

fn unit(p: &t::PreparedText, range: t::Range) -> t::PreparedUnit {
    let mut units = p
        .unit_views()
        .filter(|u| u.source_range() == range)
        .map(|u| u.handle());
    let unit = units
        .next()
        .expect("one actual ASCII TCU at this original interval");
    assert!(units.next().is_none());
    unit
}

fn reselect(
    p: &t::PreparedText,
    family: &t::PreparedInlineBoxFamily,
    owner: &t::InlineBoxSource,
    basis: &t::FinalizedLine,
) -> t::FinalizedLine {
    assert!(
        basis.inline_box_pieces().is_none(),
        "the existing no-family operation makes no global claim"
    );
    let mut request = basis.selected_line().request().clone();
    assert!(request.inline_boxes.is_none());
    request.inline_boxes = Some(family.clone());
    let selected = p.select(request).unwrap();
    assert_eq!(selected.inline_box_family(), Some(family));
    let before = selected
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| &piece.owner() == owner)
        .map(|piece| piece.fragment())
        .collect::<Vec<_>>();
    let output = selected
        .finalize(t::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: t::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(output.inline_box_family(), Some(family));
    let after = owned(&output, owner)
        .into_iter()
        .map(|piece| piece.fragment())
        .collect::<Vec<_>>();
    assert_eq!(
        before, after,
        "allocation finalization retains the actual selected identities"
    );
    output
}

fn owned(line: &t::FinalizedLine, owner: &t::InlineBoxSource) -> Vec<t::FinalizedInlineBoxPiece> {
    line.inline_box_pieces()
        .expect("a demanded family retains its actual projection")
        .filter(|piece| &piece.owner() == owner)
        .collect()
}

fn texts(piece: &t::FinalizedInlineBoxPiece) -> Vec<t::PreparedUnit> {
    piece
        .members()
        .filter_map(|member| match member.participant() {
            Some(t::PreparedParticipant::Text(unit)) => {
                // This is the whole real containing TCU, not a claim that a member's
                // range describes exclusive box ink or a general shared packet.
                assert_eq!(member.unit_source_range(), unit.view().source_range());
                let presentation = member.presentation_range();
                assert_eq!(
                    presentation.len(),
                    1,
                    "this fixture has genuine single-ASCII units"
                );
                Some(unit.clone())
            }
            _ => None,
        })
        .collect()
}

fn carrying(
    pieces: &[t::FinalizedInlineBoxPiece],
    member: &t::PreparedUnit,
) -> t::FinalizedInlineBoxPiece {
    let mut found = pieces.iter().filter(|piece| texts(piece).contains(member));
    let piece = found
        .next()
        .expect("an actual owned source member has its projected piece")
        .clone();
    assert!(
        found.next().is_none(),
        "this unshared Ahem TCU has one owned piece"
    );
    piece
}

fn ancestry(c: &t::InlineBoxFragment, a: &t::InlineBoxFragment, domain: &t::InlineBoxSource) {
    assert_ne!(
        c, a,
        "outside C/B separates the two unwrapped source memberships"
    );
    let parent = c
        .parent()
        .expect("the actual containing domain has a body fragment");
    assert_eq!(parent.owner(), *domain);
    assert_eq!(a.parent(), Some(parent.clone()));
    assert!(parent.parent().is_none());
    assert_eq!(c.nearest_common_ancestor(a).unwrap(), parent);
    assert_eq!(a.nearest_common_ancestor(c).unwrap(), parent);
    assert_eq!(c.nearest_common_ancestor(c).unwrap(), *c);
}

fn separated(narrow: bool) {
    // This genuine capture supplies abcABC, RLO(ABC), box(cA),
    // source/control levels and a genuine OptionalBreak after A.
    let p = captured_override();
    let (owner, domain) = sources(&p);
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    let c_unit = unit(&p, t::Range::new(2, 3));
    let a_unit = unit(&p, t::Range::new(3, 4));
    let opening = endpoint(&p, K::InlineBoundaryStart);
    let closing = endpoint(&p, K::InlineBoundaryEnd);
    let basis = line(
        &p,
        narrow,
        p.source_segments().next().unwrap().start(),
        paragraph_end(&p),
        100.0,
    );
    let wide = reselect(&p, &family, &owner, &basis);
    assert_eq!(wide.feedback().advance, 68.0);
    assert_eq!(
        packet(&wide),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(5, 6), 37, 33.0),
            (t::Range::new(4, 5), 36, 43.0),
            (t::Range::new(3, 4), 35, 53.0),
        ]
    );
    strip(&wide, &p, K::InlineBoundaryStart, (20.0, 23.0));
    strip(&wide, &p, K::InlineBoundaryEnd, (63.0, 68.0));
    let pieces = owned(&wide, &owner);
    assert_eq!(
        pieces.len(),
        2,
        "two global memberships, not six glyph-derived fragments"
    );
    let c = carrying(&pieces, &c_unit);
    let a = carrying(&pieces, &a_unit);
    assert_eq!(texts(&c), std::slice::from_ref(&c_unit));
    assert_eq!(texts(&a), std::slice::from_ref(&a_unit));
    let c_id = c.fragment();
    let a_id = a.fragment();
    assert_eq!(c_id.owner(), owner);
    assert_eq!(a_id.owner(), owner);
    ancestry(&c_id, &a_id, &domain);
    assert_eq!(c.start_control(), Some(opening.clone()));
    assert_eq!(c.end_control(), None);
    assert_eq!(a.start_control(), None);
    assert_eq!(a.end_control(), Some(closing.clone()));
    assert_eq!(c.start_edge().unwrap().strip_interval(), (20.0, 23.0));
    assert_eq!(a.end_edge().unwrap().strip_interval(), (63.0, 68.0));
    assert!(c.end_edge().is_none() && a.start_edge().is_none());
    for piece in [&c, &a] {
        assert_eq!(piece.family(), &family);
        assert_eq!(piece.finalized_line(), &wide);
    }

    let cut = actual_optional_break(&p);
    let continuation = cut.view().boundary();
    let first_basis = line(
        &p,
        narrow,
        p.source_segments().next().unwrap().start(),
        cut,
        48.0,
    );
    let second_basis = line(&p, narrow, continuation, paragraph_end(&p), 48.0);
    let first = reselect(&p, &family, &owner, &first_basis);
    let second = reselect(&p, &family, &owner, &second_basis);
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (48.0, 20.0)
    );
    assert_eq!(
        packet(&first),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(3, 4), 35, 33.0),
        ]
    );
    assert_eq!(
        packet(&second),
        [
            (t::Range::new(5, 6), 37, 0.0),
            (t::Range::new(4, 5), 36, 10.0)
        ]
    );
    strip(&first, &p, K::InlineBoundaryStart, (20.0, 23.0));
    strip(&first, &p, K::InlineBoundaryEnd, (43.0, 48.0));
    assert!(second.control_slots().all(|s| !matches!(
        s.control().view().kind(),
        K::InlineBoundaryStart | K::InlineBoundaryEnd
    )));
    let adjacent = owned(&first, &owner);
    assert_eq!(
        adjacent.len(),
        2,
        "wrapping adjacency does not merge distinct global IDs"
    );
    let c_wrapped = carrying(&adjacent, &c_unit);
    let a_wrapped = carrying(&adjacent, &a_unit);
    assert_eq!(texts(&c_wrapped), [c_unit]);
    assert_eq!(texts(&a_wrapped), [a_unit]);
    assert_eq!(c_wrapped.fragment(), c_id);
    assert_eq!(a_wrapped.fragment(), a_id);
    ancestry(&c_wrapped.fragment(), &a_wrapped.fragment(), &domain);
    assert_eq!(c_wrapped.start_control(), Some(opening));
    assert_eq!(a_wrapped.end_control(), Some(closing));
    assert!(c_wrapped.end_control().is_none() && a_wrapped.start_control().is_none());
    assert_eq!(
        c_wrapped.start_edge().unwrap().strip_interval(),
        (20.0, 23.0)
    );
    assert_eq!(a_wrapped.end_edge().unwrap().strip_interval(), (43.0, 48.0));
    assert!(
        owned(&second, &owner).is_empty(),
        "outside C/B carries no cA piece or edge"
    );

    let wide_layout = place(&p, &[wide]);
    assert_eq!(
        placed_packet(&wide_layout),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 23.0, 8.0),
            (37, 33.0, 8.0),
            (36, 43.0, 8.0),
            (35, 53.0, 8.0)
        ]
    );
    let wrapped_layout = place(&p, &[first, second]);
    assert_eq!(
        placed_packet(&wrapped_layout),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 23.0, 8.0),
            (35, 33.0, 8.0),
            (37, 0.0, 18.0),
            (36, 10.0, 18.0)
        ]
    );
    for layout in [&wide_layout, &wrapped_layout] {
        let pieces = layout
            .inline_box_pieces()
            .filter(|v| v.finalized_piece().owner() == owner)
            .collect::<Vec<_>>();
        assert_eq!(pieces.len(), 2);
        for piece in pieces {
            assert_eq!(piece.family(), &family);
            assert_eq!(piece.line_index(), 0);
            assert_eq!(piece.anchor(), t::Point::new(0.0, 8.0));
            assert_eq!(piece.orientation(), t::LineOrientation::Horizontal);
            assert!(piece.fragment() == c_id || piece.fragment() == a_id);
        }
    }
}

fn captured_whole_box() -> t::PreparedText {
    // Same genuine source, RLO/PDF and optional break, with the whole mixed
    // source inside the single nominated box. No foreign row can split it.
    let p = prepare_capture(
        t::Source::identified("abcABC", t::Id::from_u64(1_646_101), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2), (2, 3), (3, 4), (4, 6)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::RightToLeftOverride, 3),
            (
                K::OptionalBreak {
                    wrap: t::Wrap::Word,
                },
                4,
            ),
            (K::PopDirectionalFormat, 6),
            (K::InlineBoundaryEnd, 6),
        ],
        &[(0, 4)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            E::Connection(0),
            origin(1),
            E::Connection(1),
            control(1),
            origin(2),
            control(2),
            E::Connection(2),
            origin(3),
            control(3),
            control(4),
            E::ExitScope(1),
        ],
        AHEM,
    );
    assert_eq!(p.source_segments().len(), 1);
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
    assert!(
        p.control_views()
            .all(|c| c.capture_position().is_some() && c.handle().captured_control().is_some())
    );
    p
}

fn together(narrow: bool) {
    let p = captured_whole_box();
    let (owner, domain) = sources(&p);
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    let basis = line(
        &p,
        narrow,
        p.source_segments().next().unwrap().start(),
        paragraph_end(&p),
        100.0,
    );
    let wide = reselect(&p, &family, &owner, &basis);
    assert_eq!(wide.feedback().advance, 68.0);
    assert_eq!(
        packet(&wide),
        [
            (t::Range::new(0, 1), 67, 3.0),
            (t::Range::new(1, 2), 68, 13.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(5, 6), 37, 33.0),
            (t::Range::new(4, 5), 36, 43.0),
            (t::Range::new(3, 4), 35, 53.0),
        ]
    );
    strip(&wide, &p, K::InlineBoundaryStart, (0.0, 3.0));
    strip(&wide, &p, K::InlineBoundaryEnd, (63.0, 68.0));
    let pieces = owned(&wide, &owner);
    assert_eq!(
        pieces.len(),
        1,
        "mixed levels alone do not split an actual box"
    );
    let id = pieces[0].fragment();
    let parent = id.parent().unwrap();
    assert_eq!(parent.owner(), domain);
    assert!(parent.parent().is_none());
    let cut = actual_optional_break(&p);
    let continuation = cut.view().boundary();
    let first_basis = line(
        &p,
        narrow,
        p.source_segments().next().unwrap().start(),
        cut,
        43.0,
    );
    let second_basis = line(&p, narrow, continuation, paragraph_end(&p), 43.0);
    let first = reselect(&p, &family, &owner, &first_basis);
    let second = reselect(&p, &family, &owner, &second_basis);
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (43.0, 25.0)
    );
    assert_eq!(
        packet(&first),
        [
            (t::Range::new(0, 1), 67, 3.0),
            (t::Range::new(1, 2), 68, 13.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(3, 4), 35, 33.0)
        ]
    );
    assert_eq!(
        packet(&second),
        [
            (t::Range::new(5, 6), 37, 0.0),
            (t::Range::new(4, 5), 36, 10.0)
        ]
    );
    let a = owned(&first, &owner);
    let b = owned(&second, &owner);
    assert_eq!((a.len(), b.len()), (1, 1));
    assert_eq!(a[0].fragment(), id);
    assert_eq!(b[0].fragment(), id);
    assert_eq!(a[0].fragment().parent(), Some(parent.clone()));
    assert_eq!(b[0].fragment().parent(), Some(parent));
    assert_eq!(
        a[0].fragment()
            .nearest_common_ancestor(&b[0].fragment())
            .unwrap(),
        id
    );
    let mut first_members = texts(&a[0])
        .into_iter()
        .map(|u| u.view().source_range())
        .collect::<Vec<_>>();
    let mut second_members = texts(&b[0])
        .into_iter()
        .map(|u| u.view().source_range())
        .collect::<Vec<_>>();
    first_members.sort_by_key(|r| r.start);
    second_members.sort_by_key(|r| r.start);
    assert_eq!(
        first_members,
        [
            t::Range::new(0, 1),
            t::Range::new(1, 2),
            t::Range::new(2, 3),
            t::Range::new(3, 4)
        ]
    );
    assert_eq!(second_members, [t::Range::new(4, 5), t::Range::new(5, 6)]);
    assert_eq!(
        a[0].start_control(),
        Some(endpoint(&p, K::InlineBoundaryStart))
    );
    assert!(a[0].end_control().is_none() && a[0].end_edge().is_none());
    assert!(b[0].start_control().is_none() && b[0].start_edge().is_none());
    assert_eq!(b[0].end_control(), Some(endpoint(&p, K::InlineBoundaryEnd)));
    assert_eq!(a[0].start_edge().unwrap().strip_interval(), (0.0, 3.0));
    assert_eq!(b[0].end_edge().unwrap().strip_interval(), (20.0, 25.0));
    let placed = place(&p, &[first, second]);
    assert_eq!(
        placed_packet(&placed),
        [
            (67, 3.0, 8.0),
            (68, 13.0, 8.0),
            (69, 23.0, 8.0),
            (35, 33.0, 8.0),
            (37, 0.0, 18.0),
            (36, 10.0, 18.0)
        ]
    );
    let pieces = placed
        .inline_box_pieces()
        .filter(|v| v.finalized_piece().owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(
        pieces.len(),
        2,
        "one global fragment has one actual piece on each line"
    );
    let mut lines = pieces.iter().map(|v| v.line_index()).collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(lines, [0, 1]);
    for piece in pieces {
        assert_eq!(piece.fragment(), id);
        assert_eq!(piece.family(), &family);
    }
}

#[test]
fn f32_wrapping_adjacency_preserves_two_source_bound_global_fragments() {
    separated(true);
}
#[test]
fn f64_wrapping_adjacency_preserves_two_source_bound_global_fragments() {
    separated(false);
}
#[test]
fn f32_mixed_content_inside_one_box_keeps_one_fragment_across_the_break() {
    together(true);
}
#[test]
fn f64_mixed_content_inside_one_box_keeps_one_fragment_across_the_break() {
    together(false);
}

#[path = "bidi_fragmentation_nested.rs"]
mod bidi_fragmentation_nested;

#[path = "empty_following_donor.rs"]
mod empty_following_donor;

#[path = "bidi_fragmentation_isolate.rs"]
mod bidi_fragmentation_isolate;
