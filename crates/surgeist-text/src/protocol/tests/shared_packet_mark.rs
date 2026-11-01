//! Family metadata preserves the maintained real cross-origin GPOS attachment.
use super::*;

#[test]
fn demanded_family_keeps_attached_mark_packet_and_the_childs_actual_source_member() {
    let p = capture(true, false);
    // Existing owner qualifies actual A35/mark100/B36, anchor(400,600) minus
    // mark(100,200), UPEM1000/size10, cross-origin EGC0..2 and exact events.
    premises(&p, false);
    assert_event_custody(&p);
    let prior = finish(&p, request(&p), [0.0; 2]);
    let expected = [
        (35, 10.0, 0.0, 0.0, Range::new(0, 2)),
        (100, 0.0, 3.0, -4.0, Range::new(0, 2)),
        (36, 10.0, 10.0, 0.0, Range::new(2, 3)),
    ];
    assert_eq!(glyphs(&prior), expected);
    assert!(prior.inline_box_pieces().is_none());
    let control = p
        .control_views()
        .next()
        .unwrap()
        .handle()
        .captured_control()
        .unwrap();
    let scope = control.scope();
    assert_eq!(
        scope.domain_topology(),
        crate::SourceProcessingTopology::Complete
    );
    let owner = crate::InlineBoxSource::captured(scope);
    let base = p
        .unit_views()
        .find(|u| u.source_range() == Range::new(0, 1))
        .unwrap()
        .handle();
    let mark = p
        .unit_views()
        .find(|u| u.source_range() == Range::new(1, 2))
        .unwrap()
        .handle();
    let family = p.inline_box_family(Vec::new(), None).unwrap();
    let mut input = prior.selected_line().request().clone();
    input.inline_boxes = Some(family.clone());
    let selected = p.select(input).unwrap();
    let chosen = selected
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .map(|piece| piece.fragment())
        .collect::<Vec<_>>();
    assert_eq!(chosen.len(), 1);
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let mut pieces = line
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner);
    let piece = pieces.next().unwrap();
    assert!(pieces.next().is_none());
    let members = piece
        .members()
        .filter_map(|member| match member.participant() {
            Some(PreparedParticipant::Text(unit)) => Some(unit.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(members, [base]);
    assert!(
        !members.contains(&mark),
        "attachment closure is not ownership of the outside mark original"
    );
    assert_eq!(piece.fragment(), chosen[0]);
    let parent = piece.fragment().parent().unwrap();
    assert_eq!(
        parent.owner(),
        crate::InlineBoxSource::captured(p.captured_owner().unwrap().domain())
    );
    assert_eq!(line.inline_box_family(), Some(&family));
    assert_eq!(
        glyphs(&line),
        expected,
        "the provider packet is emitted once without mark detachment"
    );
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        line.control_slots()
            .map(|c| (c.control(), c.flow_interval(), c.strip_interval()))
            .collect::<Vec<_>>(),
        prior
            .control_slots()
            .map(|c| (c.control(), c.flow_interval(), c.strip_interval()))
            .collect::<Vec<_>>()
    );
    assert_eq!(glyphs(&prior), expected);
    assert!(prior.inline_box_pieces().is_none());
}
