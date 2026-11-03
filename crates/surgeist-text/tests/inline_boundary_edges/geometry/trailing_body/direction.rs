//! Canonical direction facts keep post-break controls in their own body.
use super::*;

#[test]
fn closing_only_body_cannot_inherit_pre_break_strong_direction() {
    let p = prepare_capture(
        t::Source::identified("A", t::Id::from_u64(1_646_407), 1),
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
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 3);
    assert_eq!(controls[1].view().kind(), &K::ForcedBreak);
    assert_eq!(controls[2].view().kind(), &K::InlineBoundaryEnd);
    assert!(
        controls[1].captured_control().unwrap().event_position()
            < controls[2].captured_control().unwrap().event_position()
    );
    let segment = p.source_segments().next().unwrap();
    let forced = p
        .opportunity_views()
        .find(|v| v.kind() == t::PreparedBreakKind::Forced)
        .expect("actual captured mandatory break")
        .handle();
    let tail_start = forced.view().boundary();
    let end = paragraph_end(&p).view().boundary();
    assert!(
        p.boundary_consumes_control(&tail_start, &controls[1])
            .unwrap()
    );
    assert!(
        !p.boundary_consumes_control(&tail_start, &controls[2])
            .unwrap()
    );
    assert!(p.boundary_consumes_control(&end, &controls[2]).unwrap());
    let first = p
        .line_direction_facts(&segment, &segment.start(), &tail_start)
        .unwrap();
    assert_eq!(
        first.paragraph_strong_direction(),
        Some(t::ParagraphBaseDirection::LeftToRight)
    );
    assert!(first.has_eligible_strong());
    let tail = p.line_direction_facts(&segment, &tail_start, &end).unwrap();
    assert!(!tail.has_eligible_strong());
    assert_eq!(
        tail.paragraph_strong_direction(),
        None,
        "the real closing-only body contains no P2/P3-eligible strong character"
    );
    assert!(matches!(
        p.line_direction_facts(&segment, &segment.start(), &end)
            .unwrap_err()
            .detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::InvalidLineDirectionQuery
        })
    ));
}
