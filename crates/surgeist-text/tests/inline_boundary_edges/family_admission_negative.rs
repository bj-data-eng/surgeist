//! Complete-body family admission and preparation identity.
use super::*;

fn admission_fixture(topology: t::capture::SourceProcessingTopology) -> t::prepared::PreparedText {
    let mut domain = block(style(t::style::WhiteSpace::Preserve), false, false);
    let t::capture::SourceProcessingScope::Block {
        topology: value, ..
    } = &mut domain
    else {
        unreachable!()
    };
    *value = topology;
    prepare_capture(
        t::source::Source::identified("A", t::source::Id::from_u64(1_646_301), 1),
        domain,
        &[(0, 1)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 1)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
        ],
        AHEM,
    )
}

fn local_request(p: &t::prepared::PreparedText) -> t::selection::LineSelectionRequest {
    assert_eq!(p.source_segments().len(), 1);
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    let owner = controls[0].captured_control().unwrap().scope();
    assert_eq!(owner.role(), t::capture::ScopeRole::Inline);
    assert_eq!(owner.entry_control(), controls[0].captured_control());
    assert_eq!(owner.exit_control(), controls[1].captured_control());
    assert_eq!(controls[1].captured_control().unwrap().scope(), owner);
    let parent = owner.parent().unwrap();
    assert_eq!(parent.role(), t::capture::ScopeRole::Domain);
    let context = p
        .inline_box_edge_context(
            t::inline_box::InlineBoxSource::captured(owner),
            t::inline_box::InlineBoxSource::captured(parent),
            t::finalized::LineEdgeSide::Start,
        )
        .unwrap();
    let segment = p.source_segments().next().unwrap();
    let mut input = request(p, segment.start(), segment.end_opportunity(), &[], 100.0);
    input.context.spacing = t::spacing::SpacingPolicy::None;
    assert!(input.inline_boxes.is_none());
    assert_eq!(input.controls.len(), 2);
    for row in &mut input.controls {
        let width = if row.occurrence == controls[0] {
            2.0
        } else {
            assert_eq!(row.occurrence, controls[1]);
            3.0
        };
        row.boundary = Some(t::allocation::InlineBoundaryAllocation::box_edge(
            context.clone(),
            t::inline_box::InlineBoxEdgeComponents::from_f64(0.0, width, 0.0).unwrap(),
        ));
    }
    assert!(p.boundary_context_demand(&input).unwrap().is_none());
    input
}

fn authentic_publication(p: &t::prepared::PreparedText, line: &t::finalized::FinalizedLine) {
    // Real maintained Ahem: UPEM1000/A advance1000 at size10. The two
    // independently supplied physical edges contribute2+3; no font/strut proxy.
    assert_eq!(
        p.analyzed_source().source().text(),
        &t::source::Text::from("A")
    );
    assert_eq!(ids(line), [(t::source::Range::new(0, 1), 35)]);
    assert_eq!(line.feedback().advance, 15.0);
    let mut glyphs = 0;
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        for glyph in run.glyphs() {
            assert_eq!(glyph.advance(), 10.0);
            glyphs += 1;
        }
    }
    assert_eq!(glyphs, 1);
    assert_eq!(line.control_slots().count(), 2);
}

fn exact_refusal(error: t::Error, expected: t::inline_box::InlineBoxFragmentationError) {
    assert_eq!(error.code, t::ErrorCode::InvalidTextProtocol);
    assert!(
        matches!(error.detail(),
            Some(t::ErrorDetail::InvalidTextProtocol {
                reason: t::prepared::ProtocolError::InlineBoxFragmentation(actual)
            }) if *actual == expected
        ),
        "unexpected family refusal: {error:?}"
    );
}

#[test]
fn unknown_topology_retains_local_edges_but_cannot_issue_global_family() {
    let unknown = admission_fixture(t::capture::SourceProcessingTopology::Unknown);
    let complete = admission_fixture(t::capture::SourceProcessingTopology::Complete);
    let unknown_request = local_request(&unknown);
    let complete_request = local_request(&complete);
    let old_unknown = finalize(&unknown, unknown_request.clone());
    let old_complete = finalize(&complete, complete_request.clone());
    authentic_publication(&unknown, &old_unknown);
    authentic_publication(&complete, &old_complete);
    assert_eq!(ids(&old_unknown), ids(&old_complete));
    assert_eq!(measures(&old_unknown), measures(&old_complete));

    // Domain0 is the actual captured root Block input ordinal, not a family ID.
    exact_refusal(
        unknown.inline_box_family(vec![], None).unwrap_err(),
        t::inline_box::InlineBoxFragmentationError::IncompleteTopology { domain: 0 },
    );
    assert!(unknown.scope_alignment_retention().is_none());
    assert!(complete.scope_alignment_retention().is_none());
    let family = complete.inline_box_family(vec![], None).unwrap();
    assert_eq!(family.prepared_text(), complete);
    let mut global_request = complete_request.clone();
    global_request.inline_boxes = Some(family.clone());
    let global = finalize(&complete, global_request);
    assert_eq!(global.inline_box_family(), Some(&family));
    authentic_publication(&complete, &global);

    // Refusal neither revokes old outputs nor poisons the ordinary local path.
    authentic_publication(&unknown, &old_unknown);
    authentic_publication(&complete, &old_complete);
    authentic_publication(&unknown, &finalize(&unknown, unknown_request));
    authentic_publication(&complete, &finalize(&complete, complete_request));
}

#[test]
fn independent_preparations_reject_foreign_family_without_revoking_prior_output() {
    let left = admission_fixture(t::capture::SourceProcessingTopology::Complete);
    let right = admission_fixture(t::capture::SourceProcessingTopology::Complete);
    assert_eq!(
        left.analyzed_source().source().text(),
        right.analyzed_source().source().text()
    );
    let left_domain = left.captured_owner().unwrap().domain();
    let right_domain = right.captured_owner().unwrap().domain();
    assert_ne!(
        left_domain, right_domain,
        "actual independent captures, not fabricated handles"
    );
    let left_family = left.inline_box_family(vec![], None).unwrap();
    let right_family = right.inline_box_family(vec![], None).unwrap();
    assert_ne!(left_family, right_family);

    for (receiver, own, foreign) in [
        (&left, &left_family, &right_family),
        (&right, &right_family, &left_family),
    ] {
        let mut valid = local_request(receiver);
        valid.inline_boxes = Some(own.clone());
        let prior = finalize(receiver, valid.clone());
        authentic_publication(receiver, &prior);
        assert_eq!(prior.inline_box_family(), Some(own));
        let mut mismatched = valid.clone();
        // Every other receiver-issued capability/allocation remains unchanged.
        mismatched.inline_boxes = Some(foreign.clone());
        exact_refusal(
            receiver.select(mismatched).unwrap_err(),
            t::inline_box::InlineBoxFragmentationError::ForeignFamily,
        );
        authentic_publication(receiver, &prior);
        let recovered = finalize(receiver, valid.clone());
        authentic_publication(receiver, &recovered);

        // Cloning the true issuer does not create a foreign preparation.
        let alias = receiver.clone();
        authentic_publication(&alias, &finalize(&alias, valid));
    }
}
