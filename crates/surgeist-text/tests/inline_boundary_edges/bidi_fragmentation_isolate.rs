//! An authentic RLI/PDI group keeps its cA box as one global fragment;
//! isolate controls affect levels and custody without becoming source glyphs.
use super::*;

fn captured_isolate() -> t::PreparedText {
    let p = prepare_capture(
        t::Source::identified("abcABC", t::Id::from_u64(1_646_101), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2), (2, 3), (3, 4), (4, 6)],
        &[
            (K::InlineBoundaryStart, 2),
            (K::RightToLeftIsolate, 2),
            (K::PopDirectionalIsolate, 4),
            (K::InlineBoundaryEnd, 4),
        ],
        &[(0, 3)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            control(1),
            origin(1),
            E::Connection(1),
            origin(2),
            control(2),
            control(3),
            E::ExitScope(1),
            E::Connection(2),
            origin(3),
        ],
        AHEM,
    );
    assert_eq!(
        p.analyzed_source().source().text(),
        &t::Text::from("abcABC")
    );
    assert_eq!(p.analyzed_source().direction(), t::Direction::LeftToRight);
    assert_eq!(
        p.unit_views()
            .map(|u| (u.source_range(), u.level().get()))
            .collect::<Vec<_>>(),
        [
            (t::Range::new(0, 1), 0),
            (t::Range::new(1, 2), 0),
            (t::Range::new(2, 3), 2),
            (t::Range::new(3, 4), 2),
            (t::Range::new(4, 5), 0),
            (t::Range::new(5, 6), 0),
        ]
    );
    // X5a starts the genuine isolate at odd level1. Its two Latin L units get
    // level2 under I2; RLI/PDI themselves remain in their surrounding level0.
    // The separate After-affinity box Start inherits actual c level2.
    assert_eq!(
        p.control_views()
            .map(|v| (v.kind().clone(), v.source_position().get(), v.level().get()))
            .collect::<Vec<_>>(),
        [
            (K::InlineBoundaryStart, 2, 2),
            (K::RightToLeftIsolate, 2, 0),
            (K::PopDirectionalIsolate, 4, 0),
            (K::InlineBoundaryEnd, 4, 0),
        ]
    );
    p
}

fn qualify_authority(p: &t::PreparedText) {
    let owner = p.captured_owner().expect("real source output authority");
    assert!(owner.joint_flow().is_none());
    let members = owner.members().collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    let raw = &members[0];
    assert_eq!(owner.facts_owner(), *raw);
    let domain = owner.domain();
    assert_eq!(raw.domain(), domain);
    assert_eq!(domain.role(), t::ScopeRole::Domain);
    assert_eq!(
        domain.domain_topology(),
        t::SourceProcessingTopology::Complete
    );
    // Complete actual domain paragraph inventory agrees with this preparation's
    // owning raw member; no exterior paragraph is assumed from a source hull.
    let paragraphs = domain
        .flow_events()
        .unwrap()
        .filter_map(|e| match e {
            t::CapturedFlowEvent::ParagraphStart(paragraph) => Some(paragraph),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(paragraphs, members);
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
    let mut segments = p.source_segments();
    assert_eq!(segments.len(), 1);
    let segment = segments.next().unwrap();
    assert_eq!(segment.start().view().source_after().get(), 0);
    assert_eq!(segment.end().view().source_before().get(), 6);
    let start = endpoint(p, K::InlineBoundaryStart)
        .captured_control()
        .unwrap();
    let rli = endpoint(p, K::RightToLeftIsolate)
        .captured_control()
        .unwrap();
    let pdi = endpoint(p, K::PopDirectionalIsolate)
        .captured_control()
        .unwrap();
    let end = endpoint(p, K::InlineBoundaryEnd)
        .captured_control()
        .unwrap();
    assert_eq!(rli.kind(), &K::RightToLeftIsolate);
    assert_eq!(pdi.kind(), &K::PopDirectionalIsolate);
    assert_ne!(rli, pdi);
    let scope = start.scope();
    assert_eq!(scope.entry_control(), Some(start.clone()));
    assert_eq!(scope.exit_control(), Some(end.clone()));
    assert_eq!(scope.parent(), Some(domain));
    for control in [&start, &rli, &pdi, &end] {
        assert_eq!(control.scope(), scope);
        assert_eq!(control.paragraph(), *raw);
        assert!(
            raw.enter_position() < control.event_position()
                && control.event_position() < raw.exit_position()
        );
    }
    let c = p
        .origin_views()
        .find(|v| v.aggregate_range() == t::Range::new(2, 3))
        .unwrap()
        .capture_position()
        .unwrap();
    let a = p
        .origin_views()
        .find(|v| v.aggregate_range() == t::Range::new(3, 4))
        .unwrap()
        .capture_position()
        .unwrap();
    assert!(start.event_position() < rli.event_position());
    assert!(rli.event_position() < c && c < a && a < pdi.event_position());
    assert!(pdi.event_position() < end.event_position());
    assert!(
        p.control_views().all(|v| v.capture_position()
            == Some(v.handle().captured_control().unwrap().event_position()))
    );
}

#[test]
fn a_genuine_rli_group_keeps_one_captured_box_fragment_and_no_pseudo_source_glyphs() {
    let p = captured_isolate();
    qualify_authority(&p); // Before construction of any fragment family.
    let (owner, domain) = sources(&p);
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    let basis = line(
        &p,
        false,
        p.source_segments().next().unwrap().start(),
        paragraph_end(&p),
        100.0,
    );
    let output = reselect(&p, &family, &owner, &basis);
    assert_eq!(output.feedback().advance, 68.0);
    assert_eq!(
        packet(&output),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(3, 4), 35, 33.0),
            (t::Range::new(4, 5), 36, 48.0),
            (t::Range::new(5, 6), 37, 58.0),
        ]
    );
    strip(&output, &p, K::InlineBoundaryStart, (20.0, 23.0));
    strip(&output, &p, K::InlineBoundaryEnd, (43.0, 48.0));
    let rli = endpoint(&p, K::RightToLeftIsolate);
    let pdi = endpoint(&p, K::PopDirectionalIsolate);
    for control in [rli, pdi] {
        let mut slots = output.control_slots().filter(|v| v.control() == control);
        assert!(slots.next().is_some());
        assert!(slots.next().is_none());
    }
    let pieces = owned(&output, &owner);
    assert_eq!(
        pieces.len(),
        1,
        "the actual isolate prevents foreign content from interposing"
    );
    let fragment = pieces[0].fragment();
    assert_eq!(fragment.owner(), owner);
    let parent = fragment.parent().unwrap();
    assert_eq!(parent.owner(), domain);
    assert!(parent.parent().is_none());
    assert_eq!(
        fragment.nearest_common_ancestor(&fragment).unwrap(),
        fragment
    );
    let c = unit(&p, t::Range::new(2, 3));
    let a = unit(&p, t::Range::new(3, 4));
    let mut members = texts(&pieces[0]);
    members.sort_by_key(|u| u.view().source_range().start);
    assert_eq!(members, [c, a]);
    assert_eq!(
        pieces[0].start_control(),
        Some(endpoint(&p, K::InlineBoundaryStart))
    );
    assert_eq!(
        pieces[0].end_control(),
        Some(endpoint(&p, K::InlineBoundaryEnd))
    );
    assert_eq!(
        pieces[0].start_edge().unwrap().strip_interval(),
        (20.0, 23.0)
    );
    assert_eq!(pieces[0].end_edge().unwrap().strip_interval(), (43.0, 48.0));
    let replay = output.with_spacing(t::SpacingAllocations::None).unwrap();
    assert_eq!(
        packet(&replay),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(3, 4), 35, 33.0),
            (t::Range::new(4, 5), 36, 48.0),
            (t::Range::new(5, 6), 37, 58.0),
        ]
    );
    let replay_pieces = owned(&replay, &owner);
    assert_eq!(replay_pieces.len(), 1);
    assert_eq!(replay_pieces[0].fragment(), fragment);
    let placed = place(&p, &[replay]);
    assert_eq!(
        placed_packet(&placed),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 23.0, 8.0),
            (35, 33.0, 8.0),
            (36, 48.0, 8.0),
            (37, 58.0, 8.0)
        ]
    );
    let views = placed
        .inline_box_pieces()
        .filter(|v| v.finalized_piece().owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].fragment(), fragment);
    assert_eq!(views[0].family(), &family);
    assert_eq!(views[0].line_index(), 0);
    assert_eq!(views[0].anchor(), t::Point::new(0.0, 8.0));
    assert_eq!(views[0].orientation(), t::LineOrientation::Horizontal);
}
