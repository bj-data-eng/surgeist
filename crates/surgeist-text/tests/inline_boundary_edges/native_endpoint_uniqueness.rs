//! Complete native families retain singular actual endpoint custody.
use super::*;

fn native_controls() -> (t::PreparedText, Vec<t::CapturedControl>) {
    let p = prepare_capture(
        t::Source::identified("A", t::Id::from_u64(1_646_701), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryEnd, 1),
            (K::InlineBoundaryEnd, 1),
        ],
        &[],
        vec![control(0), control(1), origin(0), control(2), control(3)],
        AHEM,
    );
    assert_eq!(p.source_segments().len(), 1);
    assert_eq!(
        p.unit_views().map(|u| u.source_range()).collect::<Vec<_>>(),
        [t::Range::new(0, 1)]
    );
    let views = p.control_views().collect::<Vec<_>>();
    assert_eq!(views.len(), 4);
    let actual = views
        .iter()
        .map(|v| v.handle().captured_control().unwrap())
        .collect::<Vec<_>>();
    let root = actual[0].scope();
    assert_eq!(root.role(), t::ScopeRole::Domain);
    assert_eq!(
        root.domain_topology(),
        t::SourceProcessingTopology::Complete
    );
    assert!(actual.iter().all(|c| c.scope() == root));
    assert!(root.entry_control().is_none() && root.exit_control().is_none());
    assert_eq!(
        views
            .iter()
            .map(|v| v.source_position().get())
            .collect::<Vec<_>>(),
        [0, 0, 1, 1]
    );
    assert_ne!(actual[0], actual[1]);
    assert_ne!(actual[2], actual[3]);
    assert!(
        actual
            .windows(2)
            .all(|pair| pair[0].event_position() < pair[1].event_position())
    );
    (p, actual)
}
fn invalid_pair(error: &t::Error) {
    assert!(
        matches!(
            error.detail(),
            Some(t::ErrorDetail::InvalidTextProtocol {
                reason: t::ProtocolError::InlineBoxEdge(t::InlineBoxEdgeError::InvalidPair),
            })
        ),
        "actual closed-family custody refusal: {error:?}"
    );
}
#[test]
fn complete_native_family_rejects_two_boxes_sharing_one_actual_start() {
    let (p, c) = native_controls();
    let a = p.native_inline_box(c[0].clone(), c[2].clone()).unwrap();
    let b = p.native_inline_box(c[0].clone(), c[3].clone()).unwrap();
    assert_ne!(a, b);
    invalid_pair(&p.inline_box_family(vec![a, b], None).unwrap_err());
}
#[test]
fn complete_native_family_rejects_two_boxes_sharing_one_actual_end() {
    let (p, c) = native_controls();
    let a = p.native_inline_box(c[0].clone(), c[3].clone()).unwrap();
    let b = p.native_inline_box(c[1].clone(), c[3].clone()).unwrap();
    assert_ne!(a, b);
    invalid_pair(&p.inline_box_family(vec![a, b], None).unwrap_err());
}
#[test]
fn native_family_canonicalizes_identical_pairs_but_keeps_distinct_equal_cut_controls() {
    let (p, c) = native_controls();
    let outer = p.native_inline_box(c[0].clone(), c[3].clone()).unwrap();
    let inner = p.native_inline_box(c[1].clone(), c[2].clone()).unwrap();
    assert_ne!(outer, inner);
    assert!(outer.captured_scope().is_none() && inner.captured_scope().is_none());
    let one = p.inline_box_family(vec![outer.clone()], None).unwrap();
    let repeated = p
        .inline_box_family(vec![outer.clone(), outer.clone()], None)
        .unwrap();
    assert_eq!(
        one, repeated,
        "semantic key canonicalizes the same actual pair"
    );
    let nested = p
        .inline_box_family(vec![outer.clone(), inner.clone()], None)
        .unwrap();
    let reverse = p.inline_box_family(vec![inner, outer], None).unwrap();
    assert_eq!(
        nested, reverse,
        "authentic declarations canonicalize independently of argument order"
    );
    assert_ne!(
        one, nested,
        "equal numerical cuts do not erase the distinct inner box"
    );
}
